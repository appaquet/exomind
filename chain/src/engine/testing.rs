use super::commit_manager::CommitManager;
use super::{chain_sync, SyncContext};
use super::{pending_sync, SyncState};
use crate::block::{
    Block, BlockHeight, BlockOffset, BlockOperations, BlockOwned, BlockSignatures,
    BlockSignaturesSize, SignaturesFrame,
};
use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig};
use crate::chain::ChainStore;
use crate::operation::{GroupId, NewOperation, Operation, OperationBuilder, OperationId};
use crate::pending::memory::MemoryPendingStore;
use crate::pending::PendingStore;
use exocore_core::cell::{CellNodeRole, FullCell};
use exocore_core::cell::{LocalNode, Node, NodeId};
use exocore_core::crypto::hash::Sha3_256;
use exocore_core::framing::{
    CapnpFrameBuilder, FrameBuilder, FrameReader, MultihashFrameBuilder, SizedFrameBuilder,
};
use exocore_core::protos::generated::data_chain_capnp::block_header;
use exocore_core::time::{Clock, ConsistentTimestamp};
use std::borrow::Borrow;
use std::collections::HashMap;
use tempfile::TempDir;

pub(super) struct EngineTestCluster {
    pub cells: Vec<FullCell>,

    pub nodes: Vec<LocalNode>,
    pub nodes_index: HashMap<NodeId, usize>,

    pub temp_dirs: Vec<TempDir>,

    pub clocks: Vec<Clock>,
    pub chains: Vec<DirectoryChainStore>,
    pub chains_synchronizer: Vec<chain_sync::ChainSynchronizer<DirectoryChainStore>>,

    pub pending_stores: Vec<MemoryPendingStore>,
    pub pending_stores_synchronizer: Vec<pending_sync::PendingSynchronizer<MemoryPendingStore>>,

    pub commit_managers: Vec<CommitManager<MemoryPendingStore, DirectoryChainStore>>,

    pub sync_states: Vec<SyncState>,
}

pub(super) struct EngineTestClusterConfig {
    pub nodes_count: usize,
    pub chain_config: DirectoryChainStoreConfig,
}

impl Default for EngineTestClusterConfig {
    fn default() -> Self {
        EngineTestClusterConfig {
            nodes_count: 1,
            chain_config: DirectoryChainStoreConfig {
                segment_max_size: 100_000,
                segment_over_allocate_size: 101_000,
                ..Default::default()
            },
        }
    }
}

impl EngineTestCluster {
    pub fn new(count: usize) -> EngineTestCluster {
        let config = EngineTestClusterConfig {
            nodes_count: count,
            ..Default::default()
        };
        Self::new_from_config(config)
    }

    pub fn new_from_config(config: EngineTestClusterConfig) -> EngineTestCluster {
        let mut cells = Vec::new();

        let mut nodes = Vec::new();
        let mut nodes_index = HashMap::new();

        let mut temp_dirs = Vec::new();
        let mut clocks = Vec::new();
        let mut chains = Vec::new();
        let mut chains_synchronizer = Vec::new();
        let mut pending_stores = Vec::new();
        let mut pending_stores_synchronizer = Vec::new();
        let mut commit_managers = Vec::new();
        let mut sync_states = Vec::new();

        for i in 0..config.nodes_count {
            let local_node = LocalNode::generate();

            let cell = FullCell::generate(local_node.clone());
            cells.push(cell.clone());

            nodes_index.insert(local_node.id().clone(), i);
            nodes.push(local_node.clone());

            let tempdir = tempfile::tempdir().unwrap();

            let clock = Clock::new_mocked();
            clocks.push(clock.clone());

            chains
                .push(DirectoryChainStore::create(config.chain_config, tempdir.as_ref()).unwrap());
            chains_synchronizer.push(chain_sync::ChainSynchronizer::new(
                chain_sync::ChainSyncConfig::default(),
                cell.cell().clone(),
                clock.clone(),
            ));

            pending_stores.push(MemoryPendingStore::new());
            pending_stores_synchronizer.push(pending_sync::PendingSynchronizer::new(
                pending_sync::PendingSyncConfig::default(),
                cell.cell().clone(),
                clock.clone(),
            ));

            commit_managers.push(CommitManager::new(
                crate::engine::commit_manager::CommitManagerConfig::default(),
                cell.cell().clone(),
                clock.clone(),
            ));

            sync_states.push(SyncState::default());

            temp_dirs.push(tempdir);
        }

        // Add each node to all other nodes' cell
        for cell in &cells {
            let mut cell_nodes = cell.nodes_mut();
            for node in &nodes {
                if cell.local_node().id() != node.id() {
                    cell_nodes.add(node.node().clone());
                }
            }
        }

        let mut cluster = EngineTestCluster {
            cells,
            nodes,
            nodes_index,

            temp_dirs,
            clocks,
            chains,
            chains_synchronizer,
            pending_stores,
            pending_stores_synchronizer,
            commit_managers,

            sync_states,
        };

        for i in 0..config.nodes_count {
            cluster.add_node_role(i, CellNodeRole::Chain);
        }

        cluster
    }

    pub fn get_node(&self, node_idx: usize) -> Node {
        self.nodes[node_idx].node().clone()
    }

    pub fn get_local_node(&self, node_idx: usize) -> LocalNode {
        self.nodes[node_idx].clone()
    }

    pub fn get_node_index(&self, node_id: &NodeId) -> usize {
        self.nodes_index[node_id]
    }

    pub fn chain_generate_dummy(&mut self, node_idx: usize, count: usize, seed: u64) {
        self.chain_generate_dummy_from_offset(node_idx, 0, 0, count, seed);
    }

    pub fn add_node_role(&mut self, node_idx: usize, role: CellNodeRole) {
        let node_id = self.nodes[node_idx].id().clone();
        for cell in &mut self.cells {
            let mut nodes = cell.nodes_mut();
            let node = nodes.get_mut(&node_id).unwrap();
            node.add_role(role);
        }
    }

    pub fn remove_node_role(&mut self, node_idx: usize, role: CellNodeRole) {
        let node_id = self.nodes[node_idx].id().clone();
        for cell in &mut self.cells {
            let mut nodes = cell.nodes_mut();
            let node = nodes.get_mut(&node_id).unwrap();
            node.remove_role(role);
        }
    }

    pub fn chain_append_dummy(&mut self, node_idx: usize, count: usize, seed: u64) {
        let (next_offset, next_height) =
            self.chains[node_idx]
                .get_last_block()
                .unwrap()
                .map_or((0, 0), |block| {
                    let block_header_reader = block.header().get_reader().unwrap();
                    let block_height = block_header_reader.get_height();

                    (block.next_offset(), block_height + 1)
                });

        self.chain_generate_dummy_from_offset(node_idx, next_offset, next_height, count, seed);
    }

    pub fn chain_generate_dummy_from_offset(
        &mut self,
        node_idx: usize,
        from_offset: BlockOffset,
        from_height: BlockHeight,
        count: usize,
        seed: u64,
    ) {
        let mut next_offset = from_offset;

        for i in 0..count {
            let previous_block = if i != 0 {
                Some(
                    self.chains[node_idx]
                        .get_block_from_next_offset(next_offset)
                        .unwrap(),
                )
            } else {
                None
            };

            let prev_block_msg = previous_block.map(|b| b.header);
            let operations_data = vec![0u8; 123];
            let signatures = create_dummy_block_sigs(operations_data.len() as u32);
            let signatures_size = signatures.whole_data_size() as BlockSignaturesSize;

            let block_frame = create_dummy_block(
                next_offset,
                from_height + i as u64,
                operations_data.len() as u32,
                signatures_size,
                prev_block_msg,
                seed,
            );
            let block = BlockOwned::new(next_offset, block_frame, operations_data, signatures);
            next_offset = self.chains[node_idx].write_block(&block).unwrap();
        }
    }

    pub fn pending_generate_dummy(
        &mut self,
        node_idx: usize,
        generator_node_idx: usize,
        count: usize,
    ) -> Vec<OperationId> {
        let generator_node = &self.nodes[generator_node_idx];
        let mut operations_id = Vec::new();
        for operation in dummy_pending_ops_generator(generator_node, count) {
            operations_id.push(operation.get_id().unwrap());
            self.pending_stores[node_idx]
                .put_operation(operation)
                .unwrap();
        }
        operations_id
    }

    pub fn chain_add_genesis_block(&mut self, node_idx: usize) {
        let block = BlockOwned::new_genesis(&self.cells[node_idx]).unwrap();
        self.chains[node_idx].write_block(&block).unwrap();
    }

    pub fn chain_add_block_with_operations<I, M, F>(
        &mut self,
        node_idx: usize,
        operations: I,
    ) -> Result<(), crate::engine::EngineError>
    where
        I: Iterator<Item = M>,
        M: Borrow<crate::operation::OperationFrame<F>>,
        F: FrameReader,
    {
        if self.chains[node_idx].get_last_block()?.is_none() {
            self.chain_add_genesis_block(node_idx);
        }

        let last_block = self.chains[node_idx].get_last_block()?.unwrap();

        let block_operation_id = self.consistent_timestamp(node_idx).into();
        let block_operations = BlockOperations::from_operations(operations)?;
        let block = BlockOwned::new_with_prev_block(
            &self.cells[node_idx],
            &last_block,
            block_operation_id,
            block_operations,
        )?;
        self.chains[node_idx].write_block(&block)?;

        Ok(())
    }

    pub fn get_sync_context(&self, node_idx: usize) -> SyncContext {
        SyncContext::new(self.sync_states[node_idx])
    }

    pub fn apply_sync_state(&mut self, node_idx: usize, sync_context: &SyncContext) {
        self.sync_states[node_idx] = sync_context.sync_state;
    }

    pub fn tick_pending_synchronizer(
        &mut self,
        node_idx: usize,
    ) -> Result<SyncContext, crate::engine::EngineError> {
        let mut sync_context = self.get_sync_context(node_idx);
        self.pending_stores_synchronizer[node_idx]
            .tick(&mut sync_context, &self.pending_stores[node_idx])?;
        self.apply_sync_state(node_idx, &sync_context);

        Ok(sync_context)
    }

    pub fn tick_chain_synchronizer(
        &mut self,
        node_idx: usize,
    ) -> Result<SyncContext, crate::engine::EngineError> {
        let mut sync_context = self.get_sync_context(node_idx);
        self.chains_synchronizer[node_idx].tick(&mut sync_context, &self.chains[node_idx])?;
        self.apply_sync_state(node_idx, &sync_context);

        Ok(sync_context)
    }

    pub fn tick_commit_manager(
        &mut self,
        node_idx: usize,
    ) -> Result<SyncContext, crate::engine::EngineError> {
        let mut sync_context = self.get_sync_context(node_idx);
        self.commit_managers[node_idx].tick(
            &mut sync_context,
            &mut self.pending_stores_synchronizer[node_idx],
            &mut self.pending_stores[node_idx],
            &mut self.chains[node_idx],
        )?;
        self.apply_sync_state(node_idx, &sync_context);

        Ok(sync_context)
    }

    pub fn consistent_timestamp(&self, node_idx: usize) -> ConsistentTimestamp {
        self.clocks[node_idx].consistent_time(&self.nodes[node_idx])
    }
}

pub fn create_dummy_block<I: FrameReader>(
    offset: u64,
    height: u64,
    operations_size: u32,
    signatures_size: u16,
    previous_block: Option<crate::block::BlockHeaderFrame<I>>,
    seed: u64,
) -> crate::block::BlockHeaderFrame<Vec<u8>> {
    let mut msg_builder = CapnpFrameBuilder::<block_header::Owned>::new();

    {
        let mut block_builder: block_header::Builder = msg_builder.get_builder();
        block_builder.set_offset(offset);
        block_builder.set_height(height);
        block_builder.set_operations_size(operations_size);
        block_builder.set_signatures_size(signatures_size);
        block_builder.set_proposed_node_id(&format!("seed={}", seed));

        if let Some(previous_block) = previous_block {
            let previous_block_header_reader: block_header::Reader =
                previous_block.get_reader().unwrap();
            block_builder.set_previous_offset(previous_block_header_reader.get_offset());
            block_builder.set_previous_hash(previous_block.inner().inner().multihash_bytes());
        }
    }

    let hash_frame = MultihashFrameBuilder::<Sha3_256, _>::new(msg_builder);
    let block_frame_data = SizedFrameBuilder::new(hash_frame);
    crate::block::read_header_frame(block_frame_data.as_bytes()).unwrap()
}

pub fn create_dummy_block_sigs(operations_size: u32) -> SignaturesFrame<Vec<u8>> {
    let block_signatures = BlockSignatures::new_from_signatures(vec![]);
    block_signatures
        .to_frame_for_new_block(operations_size)
        .unwrap()
}

pub fn dummy_pending_ops_generator(
    local_node: &LocalNode,
    count: usize,
) -> impl Iterator<Item = NewOperation> {
    let local_node = local_node.clone();
    (1..=count).map(move |i| {
        let (group_id, operation_id) = ((i % 10 + 1) as u64, i as u64);
        create_dummy_new_entry_op(&local_node, operation_id, group_id)
    })
}

pub fn create_dummy_new_entry_op(
    local_node: &LocalNode,
    operation_id: OperationId,
    group_id: GroupId,
) -> NewOperation {
    let mut builder = OperationBuilder::new_entry(operation_id, local_node.id(), b"bob");
    let mut frame_builder = builder.frame_builder.get_builder();
    frame_builder.set_group_id(group_id);

    builder.sign_and_build(local_node).unwrap()
}
