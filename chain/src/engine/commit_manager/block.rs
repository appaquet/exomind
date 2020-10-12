use crate::block::{Block, BlockOffset};
use crate::engine::EngineError;
use crate::operation::{GroupId, OperationId, OperationType};
use crate::{chain, pending, CommitManagerConfig};
use exocore_core::cell::{Cell, CellNodes, Node};
use exocore_core::cell::{CellNodeRole, NodeId};
use exocore_core::protos::generated::data_chain_capnp::chain_operation;
use exocore_core::sec::signature::Signature;
use exocore_core::time::{Clock, ConsistentTimestamp};
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// Structure that contains information on the pending store and blocks in it.
/// It is used by the commit manager to know if it needs to propose, sign,
/// commit blocks
pub struct PendingBlocks {
    pub blocks: HashMap<GroupId, PendingBlock>,
    pub blocks_status: HashMap<GroupId, BlockStatus>,
    pub operations_blocks: HashMap<OperationId, HashSet<GroupId>>,
    pub entries_operations_count: usize,
}

impl PendingBlocks {
    pub fn new<PS: pending::PendingStore, CS: chain::ChainStore>(
        config: &CommitManagerConfig,
        clock: &Clock,
        cell: &Cell,
        pending_store: &PS,
        chain_store: &CS,
    ) -> Result<PendingBlocks, EngineError> {
        let local_node = cell.local_node();
        let now = clock.consistent_time(local_node.node());
        let last_stored_block = chain_store
            .get_last_block()?
            .ok_or(EngineError::UninitializedChain)?;

        debug!(
            "{}: Checking for pending blocks. last_block_offset={} next_offset={}",
            cell,
            last_stored_block.offset(),
            last_stored_block.next_offset(),
        );

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
                warn!(
                    "Didn't have any operations for block proposal with group_id={}, which shouldn't be possible",
                    group_id
                );
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

                        let node_id_str = operation_reader.get_node_id()?;
                        let node_id = NodeId::from_str(node_id_str).map_err(|_| {
                            EngineError::Other(format!(
                                "Couldn't convert to NodeID: {}",
                                node_id_str
                            ))
                        })?;
                        let node = cell.nodes().get(&node_id).map(|cn| cn.node().clone());

                        proposal = Some(PendingBlockProposal {
                            node,
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
                    if proposal.offset < last_stored_block.next_offset() {
                        // means it was a proposed block for a diverged chain
                        BlockStatus::PastRefused
                    } else if nodes.is_quorum(refusals.len(), Some(CellNodeRole::Chain))
                        || has_my_refusal
                    {
                        BlockStatus::NextRefused
                    } else if has_expired {
                        BlockStatus::NextExpired
                    } else {
                        BlockStatus::NextPotential
                    }
                }
            };

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

            debug!("{}: Found new pending block: {:?}", cell, pending_block);
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

    pub fn get_block(&self, block_op_id: &OperationId) -> &PendingBlock {
        self.blocks
            .get(block_op_id)
            .expect("Couldn't find block in map")
    }

    pub fn get_block_mut(&mut self, block_op_id: &OperationId) -> &mut PendingBlock {
        self.blocks
            .get_mut(block_op_id)
            .expect("Couldn't find block in map")
    }

    pub fn map_operations_blocks(
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

    pub fn map_blocks_status(
        pending_blocks: &HashMap<OperationId, PendingBlock>,
    ) -> HashMap<OperationId, BlockStatus> {
        let mut blocks_status = HashMap::new();
        for (block_group_id, block) in pending_blocks {
            blocks_status.insert(*block_group_id, block.status);
        }
        blocks_status
    }

    pub fn potential_next_blocks(&self) -> Vec<&PendingBlock> {
        // we sort potential next blocks by which block has better potential to become a
        // block
        self.blocks
            .values()
            .filter(|block| block.status == BlockStatus::NextPotential)
            .sorted_by(|a, b| PendingBlock::compare_potential_next_block(a, b).reverse())
            .collect()
    }
}

/// Information about a block in the pending store.
///
/// This block could be a past block (committed to chain or refused), which will
/// eventually be cleaned up, or could be a next potential or refused block.
pub struct PendingBlock {
    pub group_id: OperationId,
    pub status: BlockStatus,

    pub proposal: PendingBlockProposal,
    pub refusals: Vec<PendingBlockRefusal>,
    pub signatures: Vec<PendingBlockSignature>,
    pub has_my_refusal: bool,
    pub has_my_signature: bool,

    pub operations: Vec<OperationId>,
}

impl PendingBlock {
    pub fn add_my_signature(&mut self, signature: PendingBlockSignature) {
        self.signatures.push(signature);
        self.has_my_signature = true;
    }

    pub fn add_my_refusal(&mut self, refusal: PendingBlockRefusal) {
        self.refusals.push(refusal);
        self.has_my_refusal = true;
    }

    pub fn validate_signature(&self, cell: &Cell, signature: &PendingBlockSignature) -> bool {
        let nodes = cell.nodes();
        let node = if let Some(cell_node) = nodes.get(&signature.node_id) {
            cell_node.node()
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

    pub fn compare_potential_next_block(a: &PendingBlock, b: &PendingBlock) -> Ordering {
        if a.has_my_signature {
            return Ordering::Greater;
        } else if b.has_my_signature {
            return Ordering::Less;
        }

        match a.signatures.len().cmp(&b.signatures.len()) {
            o @ Ordering::Greater => return o,
            o @ Ordering::Less => return o,
            Ordering::Equal => {}
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
            .field("node", &self.proposal.node)
            .finish()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BlockStatus {
    PastRefused,
    PastCommitted,
    NextExpired,
    NextPotential,
    NextRefused,
}

/// Block proposal wrapper
pub struct PendingBlockProposal {
    pub node: Option<Node>,
    pub offset: BlockOffset,
    pub operation: pending::StoredOperation,
}

impl PendingBlockProposal {
    pub fn get_block(&self) -> Result<crate::block::BlockHeaderFrame<&[u8]>, EngineError> {
        let operation_reader = self.operation.frame.get_reader()?;
        let inner_operation = operation_reader.get_operation();
        match inner_operation.which()? {
            chain_operation::operation::Which::BlockPropose(block_prop) => {
                Ok(crate::block::read_header_frame(block_prop?.get_block()?)?)
            }
            _ => Err(EngineError::Other(
                "Expected block sign pending op to create block signature, but got something else"
                    .to_string(),
            )),
        }
    }

    pub fn has_expired(&self, config: &CommitManagerConfig, now: ConsistentTimestamp) -> bool {
        let op_time = ConsistentTimestamp::from(self.operation.operation_id);
        (now - op_time).map_or(false, |elapsed| elapsed >= config.block_proposal_timeout)
    }
}

/// Block refusal wrapper
pub struct PendingBlockRefusal {
    pub node_id: NodeId,
}

impl PendingBlockRefusal {
    pub fn from_operation(
        operation_reader: chain_operation::Reader,
    ) -> Result<PendingBlockRefusal, EngineError> {
        let inner_operation = operation_reader.get_operation();
        match inner_operation.which()? {
            chain_operation::operation::Which::BlockRefuse(_sig) => {
                let node_id_str = operation_reader.get_node_id()?;
                let node_id = NodeId::from_str(node_id_str).map_err(|_| {
                    EngineError::Other(format!("Couldn't convert to NodeID: {}", node_id_str))
                })?;
                Ok(PendingBlockRefusal { node_id })
            }
            _ => Err(EngineError::Other(
                "Expected block refuse pending op to create block refusal, but got something else"
                    .to_string(),
            )),
        }
    }
}

/// Block signature wrapper
pub struct PendingBlockSignature {
    pub node_id: NodeId,
    pub signature: Signature,
}

impl PendingBlockSignature {
    pub fn from_operation(
        operation_reader: chain_operation::Reader,
    ) -> Result<PendingBlockSignature, EngineError> {
        let inner_operation = operation_reader.get_operation();
        match inner_operation.which()? {
            chain_operation::operation::Which::BlockSign(sig) => {
                let op_signature_reader = sig?;
                let signature_reader = op_signature_reader.get_signature()?;

                let node_id_str = operation_reader.get_node_id()?;
                let node_id = NodeId::from_str(node_id_str).map_err(|_| {
                    EngineError::Other(format!("Couldn't convert to NodeID: {}", node_id_str))
                })?;
                let signature = Signature::from_bytes(signature_reader.get_node_signature()?);

                Ok(PendingBlockSignature { node_id, signature })
            }
            _ => Err(EngineError::Other(
                "Expected block sign pending op to create block signature, but got something else"
                    .to_string(),
            )),
        }
    }
}
