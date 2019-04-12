use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use exocore_common::node::{NodeID, Nodes};
use exocore_common::security::signature::Signature;
use exocore_common::serialization::framed::{SignedFrame, TypedFrame, TypedSliceFrame};
use exocore_common::serialization::protos::data_chain_capnp::{
    block, block_signature, operation_block_propose, operation_block_sign, pending_operation,
};
use exocore_common::serialization::protos::OperationID;
use exocore_common::time::Clock;

use crate::chain;
use crate::chain::BlockOffset;
use crate::chain::{Block, BlockDepth};
use crate::engine::{chain_sync, pending_sync, Event, SyncContext};
use crate::pending;
use crate::pending::OperationType;

use super::Error;

///
/// Manages commit of pending store's operations to the chain. It does that by monitoring the pending store for incoming
/// block proposal, signing/refusing them or proposing new blocks.
///
/// It also manages cleanup of the pending store, by deleting old operations that were committed to the chain and that are
/// in block with sufficient depth.
///
pub(super) struct CommitManager<PS: pending::PendingStore, CS: chain::ChainStore> {
    node_id: NodeID,
    config: CommitManagerConfig,
    clock: Clock,
    phantom: std::marker::PhantomData<(PS, CS)>,
}

impl<PS: pending::PendingStore, CS: chain::ChainStore> CommitManager<PS, CS> {
    pub fn new(
        node_id: NodeID,
        config: CommitManagerConfig,
        clock: Clock,
    ) -> CommitManager<PS, CS> {
        CommitManager {
            node_id,
            config,
            clock,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn tick(
        &mut self,
        sync_context: &mut SyncContext,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        chain_synchronizer: &mut chain_sync::ChainSynchronizer<CS>,
        chain_store: &mut CS,
        nodes: &Nodes,
    ) -> Result<(), Error> {
        if chain_synchronizer.status() != chain_sync::Status::Synchronized {
            // we need to be synchronized in order to do any progress
            return Ok(());
        }

        // find all blocks (proposed, committed, refused, etc.) in pending store
        let mut pending_blocks = PendingBlocks::new(self, pending_store, chain_store, nodes)?;

        // get all potential next blocks sorted by most probable to less probable
        let potential_next_blocks = pending_blocks.potential_next_blocks();

        if let Some(next_block_id) =
            Self::select_potential_next_block(potential_next_blocks.as_slice())
        {
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
                            nodes,
                            mut_next_block,
                        )?;
                    } else {
                        self.refuse_block(
                            sync_context,
                            pending_synchronizer,
                            pending_store,
                            nodes,
                            mut_next_block,
                        )?;
                    }
                }
            }

            let next_block = pending_blocks.get_block(&next_block_id);
            let valid_signatures = next_block
                .signatures
                .iter()
                .filter(|sig| next_block.validate_signature(nodes, sig));
            if next_block.has_my_signature && nodes.is_quorum(valid_signatures.count()) {
                debug!("Block has enough signatures, we should commit");
                self.commit_block(sync_context, next_block, pending_store, chain_store, nodes)?;
            }
        } else if self.should_propose_block(nodes, chain_store, &pending_blocks)? {
            debug!("No current block, and we can propose one");
            self.propose_block(
                sync_context,
                &pending_blocks,
                pending_synchronizer,
                pending_store,
                chain_store,
                nodes,
            )?;
        }

        self.maybe_cleanup_pending_store(&pending_blocks, pending_store)?;

        Ok(())
    }

    fn select_potential_next_block(blocks: &[&PendingBlock]) -> Option<OperationID> {
        // TODO: Better selection than this... If we have more than 1 potential block, we should vote for the one who had more chance
        //       Otherwise, we may not be able to advance consensus if each node vote for a different proposal
        //       Ticket: https://github.com/appaquet/exocore/issues/47
        blocks.first().map(|b| b.group_id)
    }

    fn check_should_sign_block(
        &self,
        block_id: OperationID,
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
                    return Ok(false);
                }

                let operation_in_chain = chain_store
                    .get_block_by_operation_id(*operation_id)?
                    .is_some();
                if operation_in_chain {
                    return Ok(false);
                }
            }
        }

        // validate hash of operations of block
        let block_operations = Self::get_block_operations(block, pending_store)?.map(|op| op.frame);
        let operations_hash = chain::BlockOperations::hash_operations(block_operations)?;
        let block_reader = block_frame.get_typed_reader()?;
        if operations_hash.as_bytes() != block_reader.get_operations_hash()? {
            debug!(
                "Block entries hash didn't match our local hash for block id={} offset={}",
                block.group_id, block.proposal.offset
            );
            return Ok(false);
        }

        Ok(true)
    }

    fn sign_block(
        &self,
        sync_context: &mut SyncContext,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        nodes: &Nodes,
        next_block: &mut PendingBlock,
    ) -> Result<(), Error> {
        let my_node = nodes.get(&self.node_id).ok_or(Error::MyNodeNotFound)?;

        let operation_id = self.clock.consistent_time(&my_node);
        let signature_frame_builder = pending::PendingOperation::new_signature_for_block(
            next_block.group_id,
            operation_id,
            &self.node_id,
            &next_block.proposal.get_block()?,
        )?;

        let signature_frame = signature_frame_builder.as_owned_framed(my_node.frame_signer())?;
        let signature_reader = signature_frame.get_typed_reader()?;
        let pending_signature = PendingBlockSignature::from_pending_operation(signature_reader)?;

        debug!("Signing block {}", next_block.group_id);
        next_block.add_my_signature(pending_signature);

        pending_synchronizer.handle_new_operation(
            sync_context,
            nodes,
            pending_store,
            signature_frame,
        )?;

        Ok(())
    }

    fn refuse_block(
        &self,
        sync_context: &mut SyncContext,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        nodes: &Nodes,
        next_block: &mut PendingBlock,
    ) -> Result<(), Error> {
        let my_node = nodes.get(&self.node_id).ok_or(Error::MyNodeNotFound)?;

        let operation_id = self.clock.consistent_time(&my_node);
        let refusal_frame_builder = pending::PendingOperation::new_refusal(
            next_block.group_id,
            operation_id,
            &self.node_id,
        )?;
        let refusal_frame = refusal_frame_builder.as_owned_framed(my_node.frame_signer())?;
        let refusal_reader = refusal_frame.get_typed_reader()?;
        let pending_refusal = PendingBlockRefusal::from_pending_operation(refusal_reader)?;

        next_block.add_my_refusal(pending_refusal);

        pending_synchronizer.handle_new_operation(
            sync_context,
            nodes,
            pending_store,
            refusal_frame,
        )?;

        Ok(())
    }

    fn should_propose_block(
        &self,
        _nodes: &Nodes,
        _chain_store: &CS,
        _pending_blocks: &PendingBlocks,
    ) -> Result<bool, Error> {
        // TODO: Selection logic ticket: https://github.com/appaquet/exocore/issues/47
        //       I'm synchronized
        //       I have full access
        //       Last block time + duration + hash(nodeid) % 5 secs
        //       - Perhaps we should take current time into consideration so that we don't have 2 nodes proposing at the timeout

        Ok(true)
    }

    fn propose_block(
        &self,
        sync_context: &mut SyncContext,
        pending_blocks: &PendingBlocks,
        pending_synchronizer: &mut pending_sync::PendingSynchronizer<PS>,
        pending_store: &mut PS,
        chain_store: &mut CS,
        nodes: &Nodes,
    ) -> Result<(), Error> {
        let my_node = nodes.get(&self.node_id).ok_or(Error::MyNodeNotFound)?;
        let previous_block = chain_store
            .get_last_block()?
            .ok_or(Error::UninitializedChain)?;

        let block_operations = pending_store
            .operations_iter(..)?
            .filter(|operation| {
                // only include new entries or pending ignore entries
                match operation.operation_type {
                    OperationType::Entry | OperationType::PendingIgnore => true,
                    _ => false,
                }
            })
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

        let block_operations = chain::BlockOperations::from_operations(block_operations)?;
        let block_operation_id = self.clock.consistent_time(&my_node);
        let block = chain::BlockOwned::new_with_prev_block(
            nodes,
            my_node,
            &previous_block,
            block_operation_id,
            block_operations,
        )?;
        if block.operations_iter()?.next().is_none() {
            debug!("No operations need to be committed, so won't be proposing any block");
            return Ok(());
        }

        let block_proposal_frame_builder = pending::PendingOperation::new_block_proposal(
            block_operation_id,
            &self.node_id,
            &block,
        )?;
        let block_proposal_frame =
            block_proposal_frame_builder.as_owned_framed(my_node.frame_signer())?;

        debug!(
            "Proposed block with operation_id {} for offset {}",
            block_operation_id,
            previous_block.next_offset()
        );
        pending_synchronizer.handle_new_operation(
            sync_context,
            nodes,
            pending_store,
            block_proposal_frame,
        )?;

        Ok(())
    }

    fn commit_block(
        &self,
        sync_context: &mut SyncContext,
        next_block: &PendingBlock,
        pending_store: &mut PS,
        chain_store: &mut CS,
        nodes: &Nodes,
    ) -> Result<(), Error> {
        let my_node = nodes.get(&self.node_id).ok_or(Error::MyNodeNotFound)?;

        let block_frame = next_block.proposal.get_block()?;
        let block_reader: block::Reader = block_frame.get_typed_reader()?;

        // fetch block's operations from the pending store
        let block_operations =
            Self::get_block_operations(next_block, pending_store)?.map(|operation| operation.frame);

        // make sure that the hash of operations is same as defined by the block
        // this should never happen since we wouldn't have signed the block if hash didn't match
        let block_operations = chain::BlockOperations::from_operations(block_operations)?;
        if block_operations.multihash_bytes() != block_reader.get_operations_hash()? {
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
                if next_block.validate_signature(nodes, pending_signature) {
                    Some(chain::BlockSignature::new(
                        pending_signature.node_id.clone(),
                        pending_signature.signature.clone(),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let block_signatures = chain::BlockSignatures::new_from_signatures(signatures);
        let signatures_frame =
            block_signatures.to_frame_for_existing_block(my_node, &block_reader)?;

        // finally build the frame
        let block = chain::BlockOwned::new(
            next_block.proposal.offset,
            block_frame.to_owned(),
            block_operations.data().to_vec(),
            signatures_frame,
        );

        debug!("Writing with offset={} to chain", block.offset());
        chain_store.write_block(&block)?;
        sync_context.push_event(Event::ChainBlockNew(next_block.proposal.offset));

        Ok(())
    }

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

    fn maybe_cleanup_pending_store(
        &self,
        _pending_blocks: &PendingBlocks,
        _pending_store: &PS,
    ) -> Result<(), Error> {
        // TODO: Implement cleanup: https://github.com/appaquet/exocore/issues/41
        //       Check if we can advance the last block mark in pending store
        //       Emit "PendingIgnore" for
        //        - Operations that are now in the chain
        //        - Blocks that got refused after more than X
        //       Cleanup committed stuff OR ignored stuff

        Ok(())
    }
}

///
/// CommitManager's configuration
///
#[derive(Copy, Clone, Debug)]
pub struct CommitManagerConfig {
    pub operations_cleanup_after_block_depth: BlockDepth,
}

impl Default for CommitManagerConfig {
    fn default() -> Self {
        CommitManagerConfig {
            operations_cleanup_after_block_depth: 3,
        }
    }
}

///
/// Represents all the blocks that are currently in the pending store
///
struct PendingBlocks {
    blocks: HashMap<OperationID, PendingBlock>, // group_id -> block
    blocks_status: HashMap<OperationID, BlockStatus>, // group_id -> block_status
    operations_blocks: HashMap<OperationID, HashSet<OperationID>>, // operation_id -> set(group_id)
}

impl PendingBlocks {
    fn new<PS: pending::PendingStore, CS: chain::ChainStore>(
        commit_manager: &CommitManager<PS, CS>,
        pending_store: &PS,
        chain_store: &CS,
        nodes: &Nodes,
    ) -> Result<PendingBlocks, Error> {
        let last_stored_block = chain_store
            .get_last_block()?
            .ok_or(Error::UninitializedChain)?;

        // first pass to fetch all groups proposal
        let mut groups_id = Vec::new();
        for pending_op in pending_store.operations_iter(..)? {
            if pending_op.operation_type == pending::OperationType::BlockPropose {
                groups_id.push(pending_op.operation_id);
            }
        }

        let nb_nodes_consensus = (nodes.len() / 2).max(1);
        let next_offset = last_stored_block.next_offset();

        // then we get all operations for each block proposal
        let mut blocks = HashMap::<OperationID, PendingBlock>::new();
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
                let operation_reader: pending_operation::Reader =
                    operation.frame.get_typed_reader()?;
                let node_id = operation_reader.get_node_id()?;

                match operation_reader.get_operation().which()? {
                    pending_operation::operation::Which::BlockPropose(reader) => {
                        let reader: operation_block_propose::Reader = reader?;
                        let block_frame =
                            TypedSliceFrame::<block::Owned>::new(reader.get_block()?)?;
                        let block_reader: block::Reader = block_frame.get_typed_reader()?;

                        for operation_header in block_reader.get_operations_header()? {
                            operations.push(operation_header.get_operation_id());
                        }

                        proposal = Some(PendingBlockProposal {
                            node_id: node_id.to_string(),
                            offset: block_reader.get_offset(),
                            operation,
                        })
                    }
                    pending_operation::operation::Which::BlockSign(_reader) => {
                        signatures.push(PendingBlockSignature::from_pending_operation(
                            operation_reader,
                        )?);
                    }
                    pending_operation::operation::Which::BlockRefuse(_reader) => {
                        refusals.push(PendingBlockRefusal::from_pending_operation(
                            operation_reader,
                        )?);
                    }
                    pending_operation::operation::Which::PendingIgnore(_)
                    | pending_operation::operation::Which::Entry(_) => {
                        warn!("Found a non-block related operation in block group, which shouldn't be possible (group_id={})", group_id);
                    }
                };
            }

            let proposal =
                proposal.expect("Couldn't find proposal operation within a group of the proposal");
            let has_my_refusal = refusals
                .iter()
                .any(|sig| sig.node_id == commit_manager.node_id);
            let has_my_signature = signatures
                .iter()
                .any(|sig| sig.node_id == commit_manager.node_id);

            let status = match chain_store.get_block(proposal.offset).ok() {
                Some(block) => {
                    let block_reader: block::Reader = block.block.get_typed_reader()?;
                    if block_reader.get_proposed_operation_id() == *group_id {
                        BlockStatus::PastCommitted
                    } else {
                        BlockStatus::PastRefused
                    }
                }
                None => {
                    if proposal.offset < next_offset {
                        // means it was a proposed block for a diverged chain
                        BlockStatus::PastRefused
                    } else if refusals.len() >= nb_nodes_consensus || has_my_refusal {
                        BlockStatus::NextRefused
                    } else {
                        BlockStatus::NextPotential
                    }
                }
            };

            debug!(
                "Found pending store's block {} with status {:?}",
                proposal.offset, status
            );
            blocks.insert(
                *group_id,
                PendingBlock {
                    group_id: *group_id,
                    status,

                    proposal,
                    refusals,
                    signatures,

                    has_my_refusal,
                    has_my_signature,

                    operations,
                },
            );
        }

        let operations_blocks = Self::map_operations_blocks(&blocks);
        let blocks_status = Self::map_blocks_status(&blocks);

        Ok(PendingBlocks {
            blocks,
            blocks_status,
            operations_blocks,
        })
    }

    fn get_block(&self, block_op_id: &OperationID) -> &PendingBlock {
        self.blocks
            .get(block_op_id)
            .expect("Couldn't find block in map")
    }

    fn get_block_mut(&mut self, block_op_id: &OperationID) -> &mut PendingBlock {
        self.blocks
            .get_mut(block_op_id)
            .expect("Couldn't find block in map")
    }

    fn map_operations_blocks(
        pending_blocks: &HashMap<OperationID, PendingBlock>,
    ) -> HashMap<OperationID, HashSet<OperationID>> {
        let mut operations_blocks: HashMap<OperationID, HashSet<OperationID>> = HashMap::new();
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
        pending_blocks: &HashMap<OperationID, PendingBlock>,
    ) -> HashMap<OperationID, BlockStatus> {
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
            .sorted_by(|a, b| PendingBlock::compare_potential_next_block(a, b))
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
    group_id: OperationID,
    status: BlockStatus,

    proposal: PendingBlockProposal,
    refusals: Vec<PendingBlockRefusal>,
    signatures: Vec<PendingBlockSignature>,
    has_my_refusal: bool,
    has_my_signature: bool,

    operations: Vec<OperationID>,
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

    fn validate_signature(&self, nodes: &Nodes, signature: &PendingBlockSignature) -> bool {
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

        if let Some(signature_data) = block.signature_data() {
            signature.signature.validate(node, signature_data)
        } else {
            return false;
        }
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
        a.group_id.cmp(&b.group_id)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum BlockStatus {
    PastRefused,
    PastCommitted,
    NextPotential,
    NextRefused,
}

///
/// Block proposal wrapper
///
struct PendingBlockProposal {
    node_id: NodeID,
    offset: BlockOffset,
    operation: pending::StoredOperation,
}

impl PendingBlockProposal {
    fn get_block(&self) -> Result<TypedSliceFrame<block::Owned>, Error> {
        let operation_reader: pending_operation::Reader =
            self.operation.frame.get_typed_reader()?;
        let inner_operation: pending_operation::operation::Reader =
            operation_reader.get_operation();
        match inner_operation.which()? {
            pending_operation::operation::Which::BlockPropose(block_prop) => {
                let block_prop_reader: operation_block_propose::Reader = block_prop?;
                let frame = TypedSliceFrame::new(block_prop_reader.get_block()?)?;
                Ok(frame)
            }
            _ => Err(Error::Other(
                "Expected block sign pending op to create block signature, but got something else"
                    .to_string(),
            )),
        }
    }
}

///
/// Block refusal wrapper
///
struct PendingBlockRefusal {
    node_id: NodeID,
}

impl PendingBlockRefusal {
    fn from_pending_operation(
        operation_reader: pending_operation::Reader,
    ) -> Result<PendingBlockRefusal, Error> {
        let inner_operation: pending_operation::operation::Reader =
            operation_reader.get_operation();
        match inner_operation.which()? {
            pending_operation::operation::Which::BlockRefuse(_sig) => {
                let node_id = operation_reader.get_node_id()?.to_string();
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
    node_id: NodeID,
    signature: Signature,
}

impl PendingBlockSignature {
    fn from_pending_operation(
        operation_reader: pending_operation::Reader,
    ) -> Result<PendingBlockSignature, Error> {
        let inner_operation: pending_operation::operation::Reader =
            operation_reader.get_operation();
        match inner_operation.which()? {
            pending_operation::operation::Which::BlockSign(sig) => {
                let op_signature_reader: operation_block_sign::Reader = sig?;
                let signature_reader: block_signature::Reader =
                    op_signature_reader.get_signature()?;

                let node_id = signature_reader.get_node_id()?.to_string();
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
/// Error
///
#[derive(Debug, Fail)]
pub enum CommitManagerError {
    #[fail(display = "Invalid signature in commit manager: {}", _0)]
    InvalidSignature(String),
    #[fail(display = "A referenced operation is missing from local store: {}", _0)]
    MissingOperation(OperationID),
}

#[cfg(test)]
mod tests {
    use crate::chain::ChainStore;
    use crate::engine::testing::*;
    use crate::pending::PendingOperation;
    use crate::pending::PendingStore;

    use super::*;

    #[test]
    fn should_not_do_anything_until_chain_synchronized() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(1);
        cluster.chain_add_genesis_block(0);

        append_new_operation(&mut cluster, b"hello world")?;

        // if chain was synchronized, this would have proposed a block
        cluster.tick_commit_manager(0)?;
        let blocks = get_pending_blocks(&cluster)?;
        assert!(blocks.blocks.is_empty());

        // make the chain synchronized
        cluster.tick_chain_synchronizer(0)?;

        // we should have created a block now
        cluster.tick_commit_manager(0)?;
        let blocks = get_pending_blocks(&cluster)?;
        assert!(!blocks.blocks.is_empty());

        Ok(())
    }

    #[test]
    fn should_propose_block_on_new_operations() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(1);
        cluster.chain_add_genesis_block(0);
        cluster.tick_chain_synchronizer(0)?;

        // nothing will be done since nothing is in pending store
        cluster.tick_commit_manager(0)?;
        let operations = cluster.pending_stores[0].operations_iter(..)?;
        assert_eq!(operations.count(), 0);

        // append new operation
        append_new_operation(&mut cluster, b"hello world")?;

        // this should create a block proposal (2nd op in pending store)
        cluster.tick_commit_manager(0)?;
        let operations = cluster.pending_stores[0].operations_iter(..)?;
        assert_eq!(operations.count(), 2); // operation + block

        // shouldn't have signature yet
        let blocks = get_pending_blocks(&cluster)?;
        assert!(!blocks.blocks.iter().nth(0).unwrap().1.has_my_signature);

        // this should sign + commit block to chain
        cluster.tick_commit_manager(0)?;
        let operations = cluster.pending_stores[0].operations_iter(..)?;
        assert_eq!(operations.count(), 3); // operation + block + signature

        let blocks = get_pending_blocks(&cluster)?;
        assert_eq!(
            blocks.blocks.iter().nth(0).unwrap().1.status,
            BlockStatus::PastCommitted
        );
        let last_block = cluster.chains[0].get_last_block()?.unwrap();
        assert_ne!(last_block.offset, 0);

        // this should not do anything, since it's already committed
        cluster.tick_commit_manager(0)?;
        let operations = cluster.pending_stores[0].operations_iter(..)?;
        assert_eq!(operations.count(), 3); // operation + block + signature

        Ok(())
    }

    #[test]
    fn should_sign_valid_proposed_block() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(1);
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
    fn should_refuse_invalid_proposed_block() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(1);
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
        let last_block_reader = last_block.block.get_typed_reader()?;
        assert_eq!(last_block_reader.get_proposed_operation_id(), block_id_good);

        Ok(())
    }

    fn append_new_operation(cluster: &mut TestCluster, data: &[u8]) -> Result<OperationID, Error> {
        let node = cluster.get_node(0);

        let op_id = cluster.consistent_clock(0) - 1;
        let op_builder = PendingOperation::new_entry(op_id, node.id(), data);
        let op_frame = op_builder.as_owned_framed(node.frame_signer())?;
        cluster.pending_stores[0].put_operation(op_frame)?;

        Ok(op_id)
    }

    fn append_block_proposal_from_operations(
        cluster: &mut TestCluster,
        op_ids: Vec<OperationID>,
    ) -> Result<OperationID, Error> {
        let node = cluster.get_node(0);

        let previous_block = cluster.chains[0].get_last_block()?.unwrap();
        let block_operations = op_ids.iter().map(|op_id| {
            cluster.pending_stores[0]
                .get_operation(*op_id)
                .unwrap()
                .unwrap()
                .frame
        });
        let block_operations = chain::BlockOperations::from_operations(block_operations)?;
        let block_operation_id = cluster.clocks[0].consistent_time(&node);
        let block = chain::BlockOwned::new_with_prev_block(
            &cluster.nodes,
            &node,
            &previous_block,
            block_operation_id,
            block_operations,
        )?;
        let block_proposal_frame_builder =
            pending::PendingOperation::new_block_proposal(block_operation_id, node.id(), &block)?;
        let block_proposal_frame =
            block_proposal_frame_builder.as_owned_framed(node.frame_signer())?;
        cluster.pending_stores[0].put_operation(block_proposal_frame)?;

        Ok(block_operation_id)
    }

    fn get_pending_blocks(cluster: &TestCluster) -> Result<PendingBlocks, Error> {
        PendingBlocks::new(
            &cluster.commit_managers[0],
            &cluster.pending_stores[0],
            &cluster.chains[0],
            &cluster.nodes,
        )
    }
}
