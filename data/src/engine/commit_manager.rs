use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::operation::{GroupId, OperationId};
use exocore_common::crypto::signature::Signature;
use exocore_common::node::NodeId;
use exocore_common::protos::data_chain_capnp::chain_operation;
use exocore_common::time::{
    consistent_timestamp_from_duration, consistent_timestamp_to_duration, Clock,
    ConsistentTimestamp,
};

use crate::block::{
    Block, BlockHeight, BlockOffset, BlockOperations, BlockOwned, BlockSignature, BlockSignatures,
};
use crate::chain;
use crate::engine::{pending_sync, Event, SyncContext};
use crate::operation;
use crate::operation::{Operation, OperationType};
use crate::pending;
use std::str::FromStr;

use super::Error;
use crate::pending::CommitStatus;
use exocore_common::cell::{Cell, CellNodes, CellNodesRead};
use std::time::Duration;

///
/// CommitManager's configuration
///
#[derive(Copy, Clone, Debug)]
pub struct CommitManagerConfig {
    /// How deep a block need to be before we cleanup its operations from pending store
    pub operations_cleanup_after_block_depth: BlockHeight,

    /// After how many new operations in pending store do we force a commit, even if we aren't
    /// past the commit interval
    pub commit_maximum_pending_store_count: usize,

    /// Interval at which commits are made, unless we hit `commit_maximum_pending_count`
    pub commit_maximum_interval: Duration,

    /// For how long a block proposal is considered valid after its creation
    /// This is used to prevent
    pub block_proposal_timeout: Duration,
}

impl Default for CommitManagerConfig {
    fn default() -> Self {
        CommitManagerConfig {
            operations_cleanup_after_block_depth: 6,
            commit_maximum_pending_store_count: 10,
            commit_maximum_interval: Duration::from_secs(3),
            block_proposal_timeout: Duration::from_secs(7),
        }
    }
}

///
/// Manages commit of pending store's operations to the chain. It does that by monitoring the pending store for incoming
/// block proposal, signing/refusing them or proposing new blocks.
///
/// It also manages cleanup of the pending store, by deleting old operations that were committed to the chain and that are
/// in block with sufficient height.
///
pub(super) struct CommitManager<PS: pending::PendingStore, CS: chain::ChainStore> {
    config: CommitManagerConfig,
    cell: Cell,
    clock: Clock,
    phantom: std::marker::PhantomData<(PS, CS)>,
}

impl<PS: pending::PendingStore, CS: chain::ChainStore> CommitManager<PS, CS> {
    pub fn new(config: CommitManagerConfig, cell: Cell, clock: Clock) -> CommitManager<PS, CS> {
        CommitManager {
            config,
            cell,
            clock,
            phantom: std::marker::PhantomData,
        }
    }

    ///
    /// Tick is called by the Engine at interval to make progress on proposing blocks, signing / refusing
    /// proposed blocks, and committing them to the chain. We also cleanup the pending store once operations
    /// have passed a certain depth in the chain, which guarantees their persistence.
    ///
    pub fn tick(
        &mut self,
        sync_context: &mut SyncContext,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        chain_store: &mut CS,
    ) -> Result<(), Error> {
        // find all blocks (proposed, committed, refused, etc.) in pending store
        let mut pending_blocks = PendingBlocks::new(
            &self.config,
            &self.clock,
            &self.cell,
            pending_store,
            chain_store,
        )?;

        // get all potential next blocks sorted by most probable to less probable, and select the best next block
        let potential_next_blocks = pending_blocks.potential_next_blocks();
        let best_potential_next_block = potential_next_blocks.first().map(|b| b.group_id);
        debug!(
            "{}: Tick begins. potential_next_blocks={:?} best_next_block={:?}",
            self.cell.local_node().id(),
            potential_next_blocks,
            best_potential_next_block
        );

        // if we have a next block, we check if we can sign it and commit it
        if let Some(next_block_id) = best_potential_next_block {
            let (has_my_signature, has_my_refusal) = {
                let next_block = pending_blocks.get_block(&next_block_id);
                (next_block.has_my_signature, next_block.has_my_refusal)
            };

            if !has_my_signature && !has_my_refusal {
                if let Ok(should_sign) = self.check_should_sign_block(
                    next_block_id,
                    &pending_blocks,
                    chain_store,
                    pending_store,
                ) {
                    let mut_next_block = pending_blocks.get_block_mut(&next_block_id);
                    if should_sign {
                        self.sign_block(
                            sync_context,
                            pending_synchronizer,
                            pending_store,
                            mut_next_block,
                        )?;
                    } else {
                        self.refuse_block(
                            sync_context,
                            pending_synchronizer,
                            pending_store,
                            mut_next_block,
                        )?;
                    }
                }
            }

            let next_block = pending_blocks.get_block(&next_block_id);
            let valid_signatures = next_block
                .signatures
                .iter()
                .filter(|sig| next_block.validate_signature(&self.cell, sig));

            let nodes = self.cell.nodes();
            if next_block.has_my_signature && nodes.is_quorum(valid_signatures.count()) {
                debug!(
                    "{}: Block has enough signatures, we should commit",
                    self.cell.local_node().id(),
                );
                self.commit_block(sync_context, next_block, pending_store, chain_store)?;
            }
        } else if self.should_propose_block(chain_store, &pending_blocks)? {
            debug!(
                "{}: No current block, and we can propose one",
                self.cell.local_node().id(),
            );
            self.propose_block(
                sync_context,
                &pending_blocks,
                pending_synchronizer,
                pending_store,
                chain_store,
            )?;
        }

        self.maybe_cleanup_pending_store(
            sync_context,
            &pending_blocks,
            pending_store,
            chain_store,
        )?;

        Ok(())
    }

    ///
    /// Checks if we should sign a block that was previously proposed. We need to make sure
    /// all operations are valid and not already in the chain and then validate the hash of
    /// the block with local version of the operations.
    ///
    fn check_should_sign_block(
        &self,
        block_id: OperationId,
        pending_blocks: &PendingBlocks,
        chain_store: &CS,
        pending_store: &PS,
    ) -> Result<bool, Error> {
        let block = pending_blocks.get_block(&block_id);
        let block_frame = block.proposal.get_block()?;

        // make sure we don't have operations that are already committed
        for operation_id in &block.operations {
            for block_id in pending_blocks
                .operations_blocks
                .get(operation_id)
                .expect("Operation was not in map")
            {
                let op_block = pending_blocks
                    .blocks_status
                    .get(block_id)
                    .expect("Couldn't find block");
                if *op_block == BlockStatus::PastCommitted {
                    info!("{}: Refusing block {:?} because there is already a committed block at this offset", self.cell.local_node().id(), block);
                    return Ok(false);
                }

                let operation_in_chain = chain_store
                    .get_block_by_operation_id(*operation_id)?
                    .is_some();
                if operation_in_chain {
                    info!("{}: Refusing block {:?} because it contains operation_id={} already in chain", self.cell.local_node().id(), block, operation_id);
                    return Ok(false);
                }
            }
        }

        // validate hash of operations of block
        let block_operations = Self::get_block_operations(block, pending_store)?.map(|op| op.frame);
        let operations_hash = BlockOperations::hash_operations(block_operations)?;
        let block_header_reader = block_frame.get_reader()?;
        if operations_hash.as_bytes() != block_header_reader.get_operations_hash()? {
            info!(
            "{}: Refusing block {:?} because entries hash didn't match our local hash for block",
            self.cell.local_node().id(),
            block
        );
            return Ok(false);
        }

        Ok(true)
    }

    ///
    /// Adds our signature to a given block proposal.
    ///
    fn sign_block(
        &self,
        sync_context: &mut SyncContext,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        next_block: &mut PendingBlock,
    ) -> Result<(), Error> {
        let local_node = self.cell.local_node();

        let operation_id = self.clock.consistent_time(&local_node);
        let signature_frame_builder = operation::OperationBuilder::new_signature_for_block(
            next_block.group_id,
            operation_id,
            local_node.id(),
            &next_block.proposal.get_block()?,
        )?;

        let signature_operation = signature_frame_builder.sign_and_build(&local_node)?;

        let signature_reader = signature_operation.get_operation_reader()?;
        let pending_signature = PendingBlockSignature::from_operation(signature_reader)?;
        debug!(
            "{}: Signing block {:?}",
            self.cell.local_node().id(),
            next_block,
        );
        next_block.add_my_signature(pending_signature);

        pending_synchronizer.handle_new_operation(
            sync_context,
            pending_store,
            signature_operation,
        )?;

        Ok(())
    }

    ///
    /// Adds our refusal to a given block proposal (ex: it's not valid)
    ///
    fn refuse_block(
        &self,
        sync_context: &mut SyncContext,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        next_block: &mut PendingBlock,
    ) -> Result<(), Error> {
        let local_node = self.cell.local_node();

        let operation_id = self.clock.consistent_time(&local_node);

        let refusal_builder = operation::OperationBuilder::new_refusal(
            next_block.group_id,
            operation_id,
            local_node.id(),
        )?;
        let refusal_operation = refusal_builder.sign_and_build(&local_node)?;

        let refusal_reader = refusal_operation.get_operation_reader()?;
        let pending_refusal = PendingBlockRefusal::from_operation(refusal_reader)?;

        next_block.add_my_refusal(pending_refusal);

        pending_synchronizer.handle_new_operation(
            sync_context,
            pending_store,
            refusal_operation,
        )?;

        Ok(())
    }

    ///
    /// Checks if we need to propose a new block, based on when the last block was created and
    /// how many operations are in the store.
    ///
    fn should_propose_block(
        &self,
        chain_store: &CS,
        pending_blocks: &PendingBlocks,
    ) -> Result<bool, Error> {
        let local_node = self.cell.local_node();
        if !local_node.has_full_access() {
            return Ok(false);
        }

        let nodes = self.cell.nodes();
        let now = self.clock.consistent_time(local_node);
        if is_node_commit_turn(&nodes, local_node.id(), now, &self.config)? {
            // number of operations in store minus number of operations in blocks ~= non-committed
            let approx_non_committed_operations = pending_blocks
                .entries_operations_count
                .checked_sub(pending_blocks.operations_blocks.len())
                .unwrap_or(0);

            if approx_non_committed_operations >= self.config.commit_maximum_pending_store_count {
                debug!(
                    "{}: Enough operations ({} >= {}) to commit & it's my turn. Proposing one.",
                    local_node.id(),
                    approx_non_committed_operations,
                    self.config.commit_maximum_pending_store_count
                );
                return Ok(true);
            } else {
                let previous_block = chain_store
                    .get_last_block()?
                    .ok_or(Error::UninitializedChain)?;
                let previous_block_elapsed = now
                    .checked_sub(previous_block.get_proposed_operation_id()?)
                    .unwrap_or(now);
                let maximum_interval =
                    consistent_timestamp_from_duration(self.config.commit_maximum_interval);

                if previous_block_elapsed >= maximum_interval {
                    debug!(
                        "{}: Enough operations to commit & it's my turn. Will propose a block.",
                        local_node.id()
                    );
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
        }

        Ok(false)
    }

    ///
    /// Creates a new block proposal with operations currently in the store.
    ///
    fn propose_block(
        &self,
        sync_context: &mut SyncContext,
        pending_blocks: &PendingBlocks,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        chain_store: &mut CS,
    ) -> Result<(), Error> {
        let local_node = self.cell.local_node();
        let previous_block = chain_store
            .get_last_block()?
            .ok_or(Error::UninitializedChain)?;

        let block_operations = pending_store
            .operations_iter(..)?
            .filter(|operation| operation.operation_type == OperationType::Entry)
            .filter(|operation| {
                // check if operation was committed to any previous block
                let operation_is_committed = pending_blocks
                    .operations_blocks
                    .get(&operation.operation_id)
                    .map_or(false, |blocks| {
                        blocks.iter().any(|block| {
                            let block_status = pending_blocks
                                .blocks_status
                                .get(block)
                                .expect("Couldn't find status of a current block");
                            *block_status == BlockStatus::PastCommitted
                        })
                    });

                let operation_in_chain = chain_store
                    .get_block_by_operation_id(operation.operation_id)
                    .ok()
                    .and_then(|operation| operation)
                    .is_some();

                !operation_is_committed && !operation_in_chain
            })
            .sorted_by_key(|operation| operation.operation_id)
            .map(|operation| operation.frame);

        let block_operations = BlockOperations::from_operations(block_operations)?;
        let block_operation_id = self.clock.consistent_time(&local_node);
        let block = BlockOwned::new_with_prev_block(
            &self.cell,
            &previous_block,
            block_operation_id,
            block_operations,
        )?;
        if block.operations_iter()?.next().is_none() {
            debug!("No operations need to be committed, so won't be proposing any block");
            return Ok(());
        }

        let block_proposal_frame_builder = operation::OperationBuilder::new_block_proposal(
            block_operation_id,
            local_node.id(),
            &block,
        )?;
        let block_proposal_operation = block_proposal_frame_builder.sign_and_build(&local_node)?;

        debug!(
            "{}: Proposed block at offset={} operation_id={}",
            self.cell.local_node().id(),
            previous_block.next_offset(),
            block_operation_id,
        );
        pending_synchronizer.handle_new_operation(
            sync_context,
            pending_store,
            block_proposal_operation,
        )?;

        Ok(())
    }

    ///
    /// Commits (write) the given block to the chain.
    ///
    fn commit_block(
        &self,
        sync_context: &mut SyncContext,
        next_block: &PendingBlock,
        pending_store: &mut PS,
        chain_store: &mut CS,
    ) -> Result<(), Error> {
        let block_frame = next_block.proposal.get_block()?;
        let block_header_reader = block_frame.get_reader()?;

        let block_offset = next_block.proposal.offset;
        let block_height = block_header_reader.get_height();

        // fetch block's operations from the pending store
        let block_operations =
            Self::get_block_operations(next_block, pending_store)?.map(|operation| operation.frame);

        // make sure that the hash of operations is same as defined by the block
        // this should never happen since we wouldn't have signed the block if hash didn't match
        let block_operations = BlockOperations::from_operations(block_operations)?;
        if block_operations.multihash_bytes() != block_header_reader.get_operations_hash()? {
            return Err(Error::Fatal(
                "Block hash for local entries didn't match block hash, but was previously signed"
                    .to_string(),
            ));
        }

        // build signatures frame
        let signatures = next_block
            .signatures
            .iter()
            .filter_map(|pending_signature| {
                if next_block.validate_signature(&self.cell, pending_signature) {
                    Some(BlockSignature::new(
                        pending_signature.node_id.clone(),
                        pending_signature.signature.clone(),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let block_signatures = BlockSignatures::new_from_signatures(signatures);
        let signatures_frame =
            block_signatures.to_frame_for_existing_block(&block_header_reader)?;

        // finally build the frame
        let block = BlockOwned::new(
            block_offset,
            block_frame.to_owned(),
            block_operations.data().to_vec(),
            signatures_frame,
        );

        debug!(
            "{}: Writing new block to chain: {:?}",
            self.cell.local_node().id(),
            next_block
        );
        chain_store.write_block(&block)?;
        for operation_id in block_operations.operations_id() {
            pending_store.update_operation_commit_status(
                operation_id,
                CommitStatus::Committed(block_offset, block_height),
            )?;
        }
        sync_context.push_event(Event::NewChainBlock(next_block.proposal.offset));

        Ok(())
    }

    ///
    /// Retrieves from the pending store all operations that are in the given block
    ///
    fn get_block_operations(
        next_block: &PendingBlock,
        pending_store: &PS,
    ) -> Result<impl Iterator<Item = pending::StoredOperation>, Error> {
        let operations = next_block
            .operations
            .iter()
            .map(|operation| {
                pending_store
                    .get_operation(*operation)
                    .map_err(Into::into)
                    .and_then(|op| {
                        op.ok_or_else(|| CommitManagerError::MissingOperation(*operation).into())
                    })
            })
            .collect::<Result<Vec<_>, Error>>()? // collect automatically flatten result into Result<Vec<_>>
            .into_iter()
            .sorted_by_key(|operation| operation.operation_id);

        Ok(operations)
    }

    ///
    /// Cleanups all operations that have been committed to the chain and that are deep enough
    /// to be considered impossible to be removed (i.e. there are no plausible fork)
    ///
    fn maybe_cleanup_pending_store(
        &self,
        sync_context: &mut SyncContext,
        pending_blocks: &PendingBlocks,
        pending_store: &mut PS,
        chain_store: &CS,
    ) -> Result<(), Error> {
        let last_stored_block = chain_store
            .get_last_block()?
            .ok_or(Error::UninitializedChain)?;
        let last_stored_block_height = last_stored_block.get_height()?;

        // cleanup all blocks and their operations that are committed or refused with enough depth
        for (group_id, block) in &pending_blocks.blocks {
            if block.status == BlockStatus::PastCommitted
                || block.status == BlockStatus::PastRefused
            {
                let block_frame = block.proposal.get_block()?;
                let block_header_reader = block_frame.get_reader()?;

                let block_offset = block_header_reader.get_offset();
                let block_height = block_header_reader.get_height();

                let height_diff = last_stored_block_height - block_height;
                if height_diff >= self.config.operations_cleanup_after_block_depth {
                    debug!(
                        "Block {:?} can be cleaned up (last_stored_block_height={})",
                        block, last_stored_block_height
                    );

                    // delete the block & related operations (sigs, refusals, etc.)
                    pending_store.delete_operation(*group_id)?;

                    // delete operations of the block if they were committed, but not refused
                    if block.status == BlockStatus::PastCommitted {
                        for operation_id in &block.operations {
                            pending_store.delete_operation(*operation_id)?;
                        }
                    }

                    // update the sync state so that the `PendingSynchronizer` knows what was last block to get cleaned
                    sync_context.sync_state.pending_last_cleanup_block =
                        Some((block_offset, block_height));
                }
            }
        }

        // get approx number of operations that are not associated with block
        let approx_non_committed_operations = pending_blocks
            .entries_operations_count
            .checked_sub(pending_blocks.operations_blocks.len())
            .unwrap_or(0);

        // check for dangling operations, which are operations that are already in the chain but not in
        // any blocks that are in pending store. They probably got re-added to the pending store by a node
        // that was out of sync
        if approx_non_committed_operations > 0 {
            let mut operations_to_delete = Vec::new();
            for operation in pending_store.operations_iter(..)? {
                let is_in_block = pending_blocks
                    .operations_blocks
                    .contains_key(&operation.operation_id);
                if !is_in_block {
                    if let Some(block) =
                        chain_store.get_block_by_operation_id(operation.operation_id)?
                    {
                        let block_height = block.get_height()?;
                        let block_depth = last_stored_block_height - block_height;
                        if block_depth >= self.config.operations_cleanup_after_block_depth {
                            operations_to_delete.push(operation.operation_id);
                        }
                    }
                }
            }

            if !operations_to_delete.is_empty() {
                debug!(
                    "Deleting {} dangling operations from pending store",
                    operations_to_delete.len()
                );
                for operation_id in operations_to_delete {
                    pending_store.delete_operation(operation_id)?;
                }
            }
        }

        Ok(())
    }
}

///
/// In order to prevent nodes to commit new blocks all the same time resulting in splitting
/// the vote, we make nodes propose blocks in turns.
///
/// Turns are calculated by sorting nodes by their node ids, and then finding out who's turn
/// it is based on current time.
///
fn is_node_commit_turn(
    nodes: &CellNodesRead,
    my_node_id: &NodeId,
    now: u64,
    config: &CommitManagerConfig,
) -> Result<bool, Error> {
    let nodes_iter = nodes.iter();
    let sorted_nodes = nodes_iter
        .all()
        .sorted_by_key(|node| node.id().to_str())
        .collect_vec();
    let my_node_position = sorted_nodes
        .iter()
        .position(|node| node.id() == my_node_id)
        .ok_or(Error::MyNodeNotFound)? as u64;

    let commit_interval = consistent_timestamp_from_duration(config.commit_maximum_interval);
    let epoch = (now as f64 / commit_interval as f64).floor() as u64;
    let node_turn = epoch % (sorted_nodes.len() as u64);
    Ok(node_turn == my_node_position)
}

///
/// Structure that contains information on the pending store and blocks in it.
/// It is used by the commit manager to know if it needs to propose, sign, commit blocks
///
struct PendingBlocks {
    blocks: HashMap<GroupId, PendingBlock>,
    blocks_status: HashMap<GroupId, BlockStatus>,
    operations_blocks: HashMap<OperationId, HashSet<GroupId>>,
    entries_operations_count: usize,
}

impl PendingBlocks {
    fn new<PS: pending::PendingStore, CS: chain::ChainStore>(
        config: &CommitManagerConfig,
        clock: &Clock,
        cell: &Cell,
        pending_store: &PS,
        chain_store: &CS,
    ) -> Result<PendingBlocks, Error> {
        let local_node = cell.local_node();
        let now = clock.consistent_time(local_node.node());

        let last_stored_block = chain_store
            .get_last_block()?
            .ok_or(Error::UninitializedChain)?;
        let next_offset = last_stored_block.next_offset();

        // first pass to fetch all groups proposal
        let mut groups_id = Vec::new();
        let mut entries_operations_count = 0;
        for pending_op in pending_store.operations_iter(..)? {
            match pending_op.operation_type {
                OperationType::BlockPropose => {
                    groups_id.push(pending_op.operation_id);
                }
                OperationType::Entry => {
                    entries_operations_count += 1;
                }
                _ => {}
            }
        }

        // then we get all operations for each block proposal
        let mut blocks = HashMap::<OperationId, PendingBlock>::new();
        for group_id in groups_id.iter_mut() {
            let group_operations = if let Some(group_operations) =
                pending_store.get_group_operations(*group_id)?
            {
                group_operations
            } else {
                warn!("Didn't have any operations for block proposal with group_id={}, which shouldn't be possible", group_id);
                continue;
            };

            let mut operations = Vec::new();
            let mut proposal: Option<PendingBlockProposal> = None;
            let mut signatures = Vec::new();
            let mut refusals = Vec::new();

            for operation in group_operations.operations {
                let operation_reader = operation.frame.get_reader()?;

                match operation_reader.get_operation().which()? {
                    chain_operation::operation::Which::BlockPropose(reader) => {
                        let block_frame = crate::block::read_header_frame(reader?.get_block()?)?;
                        let block_header_reader = block_frame.get_reader()?;
                        for operation_header in block_header_reader.get_operations_header()? {
                            operations.push(operation_header.get_operation_id());
                        }

                        proposal = Some(PendingBlockProposal {
                            offset: block_header_reader.get_offset(),
                            operation,
                        })
                    }
                    chain_operation::operation::Which::BlockSign(_reader) => {
                        signatures.push(PendingBlockSignature::from_operation(operation_reader)?);
                    }
                    chain_operation::operation::Which::BlockRefuse(_reader) => {
                        refusals.push(PendingBlockRefusal::from_operation(operation_reader)?);
                    }
                    chain_operation::operation::Which::Entry(_) => {
                        warn!("Found a non-block related operation in block group, which shouldn't be possible (group_id={})", group_id);
                    }
                };
            }

            let proposal: PendingBlockProposal =
                proposal.expect("Couldn't find proposal operation within a group of the proposal");
            let has_my_refusal = refusals.iter().any(|sig| sig.node_id == *local_node.id());
            let has_my_signature = signatures.iter().any(|sig| sig.node_id == *local_node.id());
            let has_expired = proposal.has_expired(config, now);

            let status = match chain_store.get_block(proposal.offset).ok() {
                Some(block) => {
                    if block.get_proposed_operation_id()? == *group_id {
                        BlockStatus::PastCommitted
                    } else {
                        BlockStatus::PastRefused
                    }
                }
                None => {
                    let nodes = cell.nodes();
                    if proposal.offset < next_offset {
                        // means it was a proposed block for a diverged chain
                        BlockStatus::PastRefused
                    } else if nodes.is_quorum(refusals.len()) || has_my_refusal {
                        BlockStatus::NextRefused
                    } else if has_expired {
                        BlockStatus::NextExpired
                    } else {
                        BlockStatus::NextPotential
                    }
                }
            };

            info!(
                "{}: Found pending store's block: offset={} group_id={} status={:?}",
                cell.local_node().id(),
                proposal.offset,
                group_id,
                status
            );
            let pending_block = PendingBlock {
                group_id: *group_id,
                status,

                proposal,
                refusals,
                signatures,

                has_my_refusal,
                has_my_signature,

                operations,
            };
            blocks.insert(*group_id, pending_block);
        }

        let operations_blocks = Self::map_operations_blocks(&blocks);
        let blocks_status = Self::map_blocks_status(&blocks);

        Ok(PendingBlocks {
            blocks,
            blocks_status,
            operations_blocks,
            entries_operations_count,
        })
    }

    fn get_block(&self, block_op_id: &OperationId) -> &PendingBlock {
        self.blocks
            .get(block_op_id)
            .expect("Couldn't find block in map")
    }

    fn get_block_mut(&mut self, block_op_id: &OperationId) -> &mut PendingBlock {
        self.blocks
            .get_mut(block_op_id)
            .expect("Couldn't find block in map")
    }

    fn map_operations_blocks(
        pending_blocks: &HashMap<OperationId, PendingBlock>,
    ) -> HashMap<OperationId, HashSet<OperationId>> {
        let mut operations_blocks: HashMap<OperationId, HashSet<OperationId>> = HashMap::new();
        for block in pending_blocks.values() {
            for operation_id in &block.operations {
                let operation = operations_blocks
                    .entry(*operation_id)
                    .or_insert_with(HashSet::new);
                operation.insert(block.group_id);
            }
        }
        operations_blocks
    }

    fn map_blocks_status(
        pending_blocks: &HashMap<OperationId, PendingBlock>,
    ) -> HashMap<OperationId, BlockStatus> {
        let mut blocks_status = HashMap::new();
        for (block_group_id, block) in pending_blocks {
            blocks_status.insert(*block_group_id, block.status);
        }
        blocks_status
    }

    fn potential_next_blocks(&self) -> Vec<&PendingBlock> {
        // we sort potential next blocks by which block has better potential to become a block
        self.blocks
            .values()
            .filter(|block| block.status == BlockStatus::NextPotential)
            .sorted_by(|a, b| PendingBlock::compare_potential_next_block(a, b).reverse())
            .collect()
    }
}

///
/// Information about a block in the pending store.
///
/// This block could be a past block (committed to chain or refused), which will eventually be cleaned up,
/// or could be a next potential or refused block.
///
struct PendingBlock {
    group_id: OperationId,
    status: BlockStatus,

    proposal: PendingBlockProposal,
    refusals: Vec<PendingBlockRefusal>,
    signatures: Vec<PendingBlockSignature>,
    has_my_refusal: bool,
    has_my_signature: bool,

    operations: Vec<OperationId>,
}

impl PendingBlock {
    fn add_my_signature(&mut self, signature: PendingBlockSignature) {
        self.signatures.push(signature);
        self.has_my_signature = true;
    }

    fn add_my_refusal(&mut self, refusal: PendingBlockRefusal) {
        self.refusals.push(refusal);
        self.has_my_refusal = true;
    }

    fn validate_signature(&self, cell: &Cell, signature: &PendingBlockSignature) -> bool {
        let nodes = cell.nodes();
        let node = if let Some(node) = nodes.get(&signature.node_id) {
            node
        } else {
            return false;
        };

        let block = if let Ok(block) = self.proposal.get_block() {
            block
        } else {
            return false;
        };

        let signature_data = block.inner().inner().multihash_bytes();
        signature.signature.validate(node, signature_data)
    }

    fn compare_potential_next_block(a: &PendingBlock, b: &PendingBlock) -> Ordering {
        if a.has_my_signature {
            return Ordering::Greater;
        } else if b.has_my_signature {
            return Ordering::Less;
        }

        if a.signatures.len() > b.signatures.len() {
            return Ordering::Greater;
        } else if a.signatures.len() < b.signatures.len() {
            return Ordering::Less;
        }

        // fallback to operation id, which is time ordered
        if a.group_id < b.group_id {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl std::fmt::Debug for PendingBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("PendingBlock")
            .field("offset", &self.proposal.offset)
            .field("group_id", &self.group_id)
            .field("status", &self.status)
            .field("nb_signatures", &self.signatures.len())
            .field("has_my_signature", &self.has_my_signature)
            .field("has_my_refusal", &self.has_my_refusal)
            .finish()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum BlockStatus {
    PastRefused,
    PastCommitted,
    NextExpired,
    NextPotential,
    NextRefused,
}

///
/// Block proposal wrapper
///
struct PendingBlockProposal {
    offset: BlockOffset,
    operation: pending::StoredOperation,
}

impl PendingBlockProposal {
    fn get_block(&self) -> Result<crate::block::BlockHeaderFrame<&[u8]>, Error> {
        let operation_reader = self.operation.frame.get_reader()?;
        let inner_operation = operation_reader.get_operation();
        match inner_operation.which()? {
            chain_operation::operation::Which::BlockPropose(block_prop) => {
                Ok(crate::block::read_header_frame(block_prop?.get_block()?)?)
            }
            _ => Err(Error::Other(
                "Expected block sign pending op to create block signature, but got something else"
                    .to_string(),
            )),
        }
    }

    fn has_expired(&self, config: &CommitManagerConfig, now: ConsistentTimestamp) -> bool {
        let elapsed = now.checked_sub(self.operation.operation_id).unwrap_or(0);
        consistent_timestamp_to_duration(elapsed) >= config.block_proposal_timeout
    }
}

///
/// Block refusal wrapper
///
struct PendingBlockRefusal {
    node_id: NodeId,
}

impl PendingBlockRefusal {
    fn from_operation(
        operation_reader: chain_operation::Reader,
    ) -> Result<PendingBlockRefusal, Error> {
        let inner_operation = operation_reader.get_operation();
        match inner_operation.which()? {
            chain_operation::operation::Which::BlockRefuse(_sig) => {
                let node_id_str = operation_reader.get_node_id()?;
                let node_id = NodeId::from_str(node_id_str).map_err(|_| {
                    Error::Other(format!("Couldn't convert to NodeID: {}", node_id_str))
                })?;
                Ok(PendingBlockRefusal { node_id })
            }
            _ => Err(Error::Other(
                "Expected block refuse pending op to create block refusal, but got something else"
                    .to_string(),
            )),
        }
    }
}

///
/// Block signature wrapper
///
struct PendingBlockSignature {
    node_id: NodeId,
    signature: Signature,
}

impl PendingBlockSignature {
    fn from_operation(
        operation_reader: chain_operation::Reader,
    ) -> Result<PendingBlockSignature, Error> {
        let inner_operation = operation_reader.get_operation();
        match inner_operation.which()? {
            chain_operation::operation::Which::BlockSign(sig) => {
                let op_signature_reader = sig?;
                let signature_reader = op_signature_reader.get_signature()?;

                let node_id_str = operation_reader.get_node_id()?;
                let node_id = NodeId::from_str(node_id_str).map_err(|_| {
                    Error::Other(format!("Couldn't convert to NodeID: {}", node_id_str))
                })?;
                let signature = Signature::from_bytes(signature_reader.get_node_signature()?);

                Ok(PendingBlockSignature { node_id, signature })
            }
            _ => Err(Error::Other(
                "Expected block sign pending op to create block signature, but got something else"
                    .to_string(),
            )),
        }
    }
}

///
/// CommitManager related error
///
#[derive(Clone, Debug, Fail)]
pub enum CommitManagerError {
    #[fail(display = "Invalid signature in commit manager: {}", _0)]
    InvalidSignature(String),
    #[fail(display = "A referenced operation is missing from local store: {}", _0)]
    MissingOperation(OperationId),
}

#[cfg(test)]
mod tests {
    use crate::chain::ChainStore;
    use crate::engine::testing::*;
    use crate::operation::{NewOperation, OperationBuilder};
    use crate::pending::PendingStore;

    use super::*;
    use std::time::Instant;

    #[test]
    fn should_propose_block_on_new_operations() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // nothing will be done since nothing is in pending store
        cluster.tick_commit_manager(0)?;
        assert_eq!(0, cluster.pending_stores[0].operations_count());

        // append new operation
        append_new_operation(&mut cluster, b"hello world")?;

        // this should create a block proposal (2nd op in pending store)
        cluster.tick_commit_manager(0)?;
        assert_eq!(2, cluster.pending_stores[0].operations_count()); // operation + block

        // shouldn't have signature yet
        let blocks = get_pending_blocks(&cluster)?;
        assert!(!blocks.blocks.iter().nth(0).unwrap().1.has_my_signature);

        // this should sign + commit block to chain
        cluster.tick_commit_manager(0)?;
        assert_eq!(3, cluster.pending_stores[0].operations_count()); // operation + block + signature

        let blocks = get_pending_blocks(&cluster)?;
        assert_eq!(
            blocks.blocks.iter().nth(0).unwrap().1.status,
            BlockStatus::PastCommitted
        );
        let last_block = cluster.chains[0].get_last_block()?.unwrap();
        assert_ne!(last_block.offset, 0);

        // this should not do anything, since it's already committed
        cluster.tick_commit_manager(0)?;
        assert_eq!(3, cluster.pending_stores[0].operations_count()); // operation + block + signature

        Ok(())
    }

    #[test]
    fn only_one_node_at_time_should_commit() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_add_genesis_block(0);
        cluster.chain_add_genesis_block(1);
        cluster.tick_chain_synchronizer(0)?;
        cluster.tick_chain_synchronizer(1)?;

        // add operation & try to commit on each node
        append_new_operation(&mut cluster, b"hello world")?;
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;

        append_new_operation(&mut cluster, b"hello world")?;
        cluster.tick_commit_manager(1)?;
        cluster.tick_commit_manager(1)?;

        // only one node should have committed since it was its turn
        assert_ne!(
            cluster.pending_stores[0].operations_count(),
            cluster.pending_stores[1].operations_count()
        );

        Ok(())
    }

    #[test]
    fn commit_block_at_interval() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        let commit_interval = cluster.commit_managers[0].config.commit_maximum_interval;

        cluster.clocks[0].set_fixed_instant(Instant::now());

        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // first block should be committed right away since there is no previous
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
        append_new_operation(&mut cluster, b"hello world")?;
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;
        let block = cluster.chains[0].get_last_block()?.unwrap();
        let first_block_offset = block.offset();
        assert_ne!(0, first_block_offset);

        // second block should wait for time
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
        append_new_operation(&mut cluster, b"hello world")?;
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;
        let block = cluster.chains[0].get_last_block()?.unwrap();
        assert_eq!(first_block_offset, block.offset());

        // time has passed, should now commit
        cluster.clocks[0].add_fixed_instant_duration(commit_interval);
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;
        let block = cluster.chains[0].get_last_block()?.unwrap();
        assert_ne!(first_block_offset, block.offset());

        Ok(())
    }

    #[test]
    fn commit_block_after_maximum_operations() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.clocks[0].set_fixed_instant(Instant::now());

        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // first block should be committed right away since there is not previous
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
        append_new_operation(&mut cluster, b"hello world")?;
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;
        let block = cluster.chains[0].get_last_block()?.unwrap();
        let first_block_offset = block.offset();
        assert_ne!(0, first_block_offset);

        // should not commit new operations because didn't exceed interval & not enough
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
        append_new_operation(&mut cluster, b"hello world")?;
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;
        let block = cluster.chains[0].get_last_block()?.unwrap();
        assert_eq!(first_block_offset, block.offset());

        // now add maximum ops
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
        let max_ops = cluster.commit_managers[0]
            .config
            .commit_maximum_pending_store_count;
        for _i in 0..=max_ops {
            append_new_operation(&mut cluster, b"hello world")?;
        }

        // it should commits
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;
        let block = cluster.chains[0].get_last_block()?.unwrap();
        assert_ne!(first_block_offset, block.offset());

        Ok(())
    }

    #[test]
    fn update_pending_status_for_committed_operations() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.clocks[0].set_fixed_instant(Instant::now());

        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // first block should be committed right away since there is not previous
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
        let op_id = append_new_operation(&mut cluster, b"hello world")?;
        assert_eq!(
            cluster.pending_stores[0]
                .get_operation(op_id)?
                .unwrap()
                .commit_status,
            CommitStatus::Unknown
        );
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;

        let block = cluster.chains[0].get_last_block()?.unwrap();
        assert_eq!(
            cluster.pending_stores[0]
                .get_operation(op_id)?
                .unwrap()
                .commit_status,
            CommitStatus::Committed(block.offset(), block.get_height()?)
        );

        Ok(())
    }

    #[test]
    fn should_sign_valid_proposed_block() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // append an operation
        let op_data = b"hello world";
        let op_id = append_new_operation(&mut cluster, op_data)?;

        // add a block proposal for this operation
        let block_id = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;

        // ticking should sign the block
        cluster.tick_commit_manager(0)?;

        let blocks = get_pending_blocks(&cluster)?;
        assert!(blocks.blocks[&block_id].has_my_signature);

        // should commit to chain
        cluster.tick_commit_manager(0)?;
        let last_block = cluster.chains[0].get_last_block()?.unwrap();
        assert_ne!(last_block.offset, 0);

        Ok(())
    }

    #[test]
    fn should_order_next_best_blocks() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // add 2 proposal
        let op_id = append_new_operation(&mut cluster, b"hello world")?;
        let block_id_signed = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;
        let _block_id_unsigned = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;

        // get blocks and fake signature on 1
        let mut blocks = get_pending_blocks(&cluster)?;
        blocks
            .blocks
            .get_mut(&block_id_signed)
            .unwrap()
            .has_my_signature = true;

        // the signed block should be first
        assert_eq!(
            blocks.potential_next_blocks().first().unwrap().group_id,
            block_id_signed
        );

        Ok(())
    }

    #[test]
    fn should_refuse_invalid_proposed_block() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // append an operation
        let op_data = b"hello world";
        let op_id = append_new_operation(&mut cluster, op_data)?;

        // should sign this block
        let block_id_good = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;
        cluster.tick_commit_manager(0)?;

        // should refuse this block as another one is already signed
        let block_id_bad = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;
        cluster.tick_commit_manager(0)?;

        let blocks = get_pending_blocks(&cluster)?;
        assert!(blocks.blocks[&block_id_good].has_my_signature);
        assert!(blocks.blocks[&block_id_bad].has_my_refusal);

        // should commit the good block, and ignore refused one
        cluster.tick_commit_manager(0)?;
        let last_block = cluster.chains[0].get_last_block()?.unwrap();
        let last_block_header_reader = last_block.header.get_reader()?;
        assert_eq!(
            last_block_header_reader.get_proposed_operation_id(),
            block_id_good
        );

        Ok(())
    }

    #[test]
    fn proposal_should_expire_after_timeout() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);

        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        let config = cluster.commit_managers[0].config;

        // create block with 1 operation
        cluster.clocks[0].set_fixed_instant(Instant::now());
        let op_data = b"hello world";
        let op_id = append_new_operation(&mut cluster, op_data)?;
        let block_id = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;

        // not expired
        let now = cluster.consistent_timestamp(0);
        let blocks = get_pending_blocks(&cluster)?;
        assert!(!blocks.blocks[&block_id].proposal.has_expired(&config, now));
        assert_eq!(blocks.blocks[&block_id].status, BlockStatus::NextPotential);

        // expired
        cluster.clocks[0].add_fixed_instant_duration(config.block_proposal_timeout);
        let now = cluster.consistent_timestamp(0);
        let blocks = get_pending_blocks(&cluster)?;
        assert!(blocks.blocks[&block_id].proposal.has_expired(&config, now));
        assert_eq!(blocks.blocks[&block_id].status, BlockStatus::NextExpired);

        // should propose a new block since previous has expired
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
        cluster.tick_commit_manager(0)?;
        let blocks = get_pending_blocks(&cluster)?;
        let potential_next = blocks.potential_next_blocks();
        assert_eq!(potential_next.len(), 1);

        Ok(())
    }

    #[test]
    fn test_is_node_commit_turn() -> Result<(), failure::Error> {
        let cluster = EngineTestCluster::new(2);
        let node1 = cluster.get_node(0);
        let node2 = cluster.get_node(1);

        // we use node id to sort nodes
        let (first_node, sec_node) = if node1.id().to_str() < node2.id().to_str() {
            (&node1, &node2)
        } else {
            (&node2, &node1)
        };

        let config = CommitManagerConfig {
            commit_maximum_interval: Duration::from_secs(2),
            ..CommitManagerConfig::default()
        };

        let nodes = cluster.cells[0].nodes();
        let now = consistent_timestamp_from_duration(Duration::from_millis(0));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(!is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);

        let now = consistent_timestamp_from_duration(Duration::from_millis(1999));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(!is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);

        let now = consistent_timestamp_from_duration(Duration::from_millis(2000));
        assert!(!is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);

        let now = consistent_timestamp_from_duration(Duration::from_millis(3999));
        assert!(!is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);

        Ok(())
    }

    #[test]
    fn cleanup_past_committed_operations() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.clocks[0].set_fixed_instant(Instant::now());

        let assert_not_in_pending = |cluster: &EngineTestCluster, operation_id: u64| {
            assert!(&cluster.pending_stores[0]
                .get_operation(operation_id)
                .unwrap()
                .is_none());
        };

        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        let config = cluster.commit_managers[0].config;

        let mut operations_id = Vec::new();
        for _i in 0..=config.operations_cleanup_after_block_depth {
            // advance clock so that we make sure it commits
            cluster.clocks[0].add_fixed_instant_duration(config.commit_maximum_interval);

            let op_id = append_new_operation(&mut cluster, b"hello world")?;
            operations_id.push(op_id);

            // should create proposal, sign it and commit it
            cluster.tick_commit_manager(0)?;
            cluster.tick_commit_manager(0)?;

            // make sure it's committed to chain
            assert!(cluster.chains[0]
                .get_block_by_operation_id(op_id)?
                .is_some());
        }

        // this will cleanup
        cluster.tick_commit_manager(0)?;

        // the first op should have been removed from pending store
        let first_op_id = *operations_id.first().unwrap();
        assert_not_in_pending(&cluster, first_op_id);

        // check if the block, signatures are still in pending
        let block: crate::block::BlockRef = cluster.chains[0]
            .get_block_by_operation_id(first_op_id)?
            .unwrap();
        let block_frame = block.header.get_reader()?;
        let block_group_id = block_frame.get_proposed_operation_id();
        assert_not_in_pending(&cluster, block_group_id);

        // check that SyncState was updated correctly
        let (cleanup_offset, cleanup_height) =
            cluster.sync_states[0].pending_last_cleanup_block.unwrap();
        assert_eq!(cleanup_height, block.get_height()?);
        assert_eq!(cleanup_offset, block.offset());

        // check if individual operations are still in pending
        for operation in block.operations_iter()? {
            let operation_reader = operation.get_reader()?;
            assert_not_in_pending(&cluster, operation_reader.get_operation_id());
        }

        Ok(())
    }

    #[test]
    fn dont_cleanup_operations_from_commit_refused_blocks() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_generate_dummy(0, 10, 1234);
        cluster.tick_chain_synchronizer(0)?;

        let preceding_valid_block = cluster.chains[0]
            .blocks_iter(0)?
            .map(|b| b.to_owned())
            .nth(2)
            .unwrap();

        // generate operations that won't be in a block yet
        let mut operations_id = Vec::new();
        let operations = (0..10).map(|i| {
            let op_id = append_new_operation(&mut cluster, b"hello world").unwrap();
            operations_id.push(op_id);
            cluster.pending_stores[0]
                .get_operation(operations_id[i])
                .unwrap()
                .unwrap()
                .frame
        });

        // we generate a block that is after block #2 in the chain, but is invalid since there is already
        // a block a this position
        let block_operations = BlockOperations::from_operations(operations)?;
        let invalid_block = BlockOwned::new_with_prev_block(
            &cluster.cells[0],
            &preceding_valid_block,
            cluster.consistent_timestamp(0),
            block_operations,
        )?;
        let invalid_block_op_id = cluster.consistent_timestamp(0);
        let block_proposal = OperationBuilder::new_block_proposal(
            invalid_block_op_id,
            cluster.get_node(0).id(),
            &invalid_block,
        )?;

        let local_node = cluster.get_local_node(0);
        cluster.pending_stores[0].put_operation(block_proposal.sign_and_build(&local_node)?)?;

        // created blocks should all be invalid
        let pending_blocks = get_pending_blocks(&cluster)?;
        assert_eq!(
            BlockStatus::PastRefused,
            pending_blocks.blocks_status[&invalid_block_op_id]
        );

        // trigger cleanup
        let mut sync_context = cluster.get_sync_context(0);
        cluster.commit_managers[0].maybe_cleanup_pending_store(
            &mut sync_context,
            &pending_blocks,
            &mut cluster.pending_stores[0],
            &cluster.chains[0],
        )?;

        // all operations previously created should still be there since they aren't committed
        // and were in a past refused block
        for operation_id in &operations_id {
            assert!(&cluster.pending_stores[0]
                .get_operation(*operation_id)
                .unwrap()
                .is_some());
        }

        Ok(())
    }

    #[test]
    fn cleanup_dangling_operations() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.clocks[0].set_fixed_instant(Instant::now());

        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        let config = cluster.commit_managers[0].config;

        let mut operations_id = Vec::new();
        for _i in 0..=config.operations_cleanup_after_block_depth {
            // advance clock so that we make sure it commits
            cluster.clocks[0].add_fixed_instant_duration(config.commit_maximum_interval);

            let op_id = append_new_operation(&mut cluster, b"hello world")?;
            operations_id.push(op_id);

            // should create proposal, sign it and commit it
            cluster.tick_commit_manager(0)?;
            cluster.tick_commit_manager(0)?;

            // make sure it's committed to chain
            assert!(cluster.chains[0]
                .get_block_by_operation_id(op_id)?
                .is_some());
        }

        // clear pending store
        cluster.pending_stores[0].clear();

        // revive old operation
        let first_op_id = *operations_id.first().unwrap();
        let block: crate::block::BlockRef = cluster.chains[0]
            .get_block_by_operation_id(first_op_id)?
            .unwrap();
        let operation = block.get_operation(first_op_id)?.unwrap();
        cluster.pending_stores[0].put_operation(NewOperation::from_frame(operation.to_owned()))?;
        assert_eq!(1, cluster.pending_stores[0].operations_count());

        // this should trigger cleanup of dandling operation
        cluster.tick_commit_manager(0)?;

        assert_eq!(0, cluster.pending_stores[0].operations_count());

        Ok(())
    }

    fn append_new_operation(
        cluster: &mut EngineTestCluster,
        data: &[u8],
    ) -> Result<OperationId, Error> {
        let op_id = cluster.consistent_timestamp(0);

        for node in cluster.nodes.iter() {
            let idx = cluster.get_node_index(node.id());
            let op_builder = OperationBuilder::new_entry(op_id, node.id(), data);
            let operation = op_builder.sign_and_build(&node)?;
            cluster.pending_stores[idx].put_operation(operation)?;
        }

        Ok(op_id)
    }

    fn append_block_proposal_from_operations(
        cluster: &mut EngineTestCluster,
        op_ids: Vec<OperationId>,
    ) -> Result<OperationId, Error> {
        let node = &cluster.nodes[0];

        let previous_block = cluster.chains[0].get_last_block()?.unwrap();
        let block_operations = op_ids.iter().map(|op_id| {
            cluster.pending_stores[0]
                .get_operation(*op_id)
                .unwrap()
                .unwrap()
                .frame
        });
        let block_operations = BlockOperations::from_operations(block_operations)?;
        let block_operation_id = cluster.clocks[0].consistent_time(&node);
        let block = BlockOwned::new_with_prev_block(
            &cluster.cells[0],
            &previous_block,
            block_operation_id,
            block_operations,
        )?;
        let block_proposal_frame_builder =
            operation::OperationBuilder::new_block_proposal(block_operation_id, node.id(), &block)?;
        let operation = block_proposal_frame_builder.sign_and_build(node)?;

        cluster.pending_stores[0].put_operation(operation)?;

        Ok(block_operation_id)
    }

    fn get_pending_blocks(cluster: &EngineTestCluster) -> Result<PendingBlocks, Error> {
        PendingBlocks::new(
            &cluster.commit_managers[0].config,
            &cluster.clocks[0],
            &cluster.cells[0],
            &cluster.pending_stores[0],
            &cluster.chains[0],
        )
    }
}
