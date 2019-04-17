use tempdir::TempDir;

use exocore_common::node::{Node, Nodes};
use exocore_common::serialization::framed::{
    FrameBuilder, MultihashFrameSigner, OwnedTypedFrame, TypedFrame,
};
use exocore_common::serialization::protos::data_chain_capnp::{
    block, block_signatures, pending_operation,
};

use crate::chain;
use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig as DirectoryConfig};
use crate::chain::{BlockOwned, ChainStore};
use crate::engine::commit_manager::CommitManager;
use crate::engine::pending_sync;
use crate::engine::{chain_sync, SyncContext};
use crate::pending::memory::MemoryPendingStore;
use crate::pending::PendingStore;
use exocore_common::serialization::protos::{GroupID, OperationID};
use exocore_common::time::Clock;

pub(super) struct TestCluster {
    pub nodes: Nodes,
    pub temp_dirs: Vec<TempDir>,

    pub clocks: Vec<Clock>,
    pub chains: Vec<DirectoryChainStore>,
    pub chains_synchronizer: Vec<chain_sync::ChainSynchronizer<DirectoryChainStore>>,

    pub pending_stores: Vec<MemoryPendingStore>,
    pub pending_stores_synchronizer: Vec<pending_sync::PendingSynchronizer<MemoryPendingStore>>,

    pub commit_managers: Vec<CommitManager<MemoryPendingStore, DirectoryChainStore>>,
}

impl TestCluster {
    pub fn new(count: usize) -> TestCluster {
        let mut nodes = Nodes::new();
        let mut temp_dirs = Vec::new();
        let mut clocks = Vec::new();
        let mut chains = Vec::new();
        let mut chains_synchronizer = Vec::new();
        let mut pending_stores = Vec::new();
        let mut pending_stores_synchronizer = Vec::new();
        let mut commit_managers = Vec::new();

        for i in 0..count {
            let node_id = format!("node{}", i);
            nodes.add(Node::new(node_id.clone()));

            let tempdir = TempDir::new("test_cluster").unwrap();

            let clock = Clock::new_mocked();
            clocks.push(clock.clone());

            let chain_config = DirectoryConfig {
                segment_max_size: 100_000,
                segment_over_allocate_size: 101_000,
                ..DirectoryConfig::default()
            };
            chains.push(DirectoryChainStore::create(chain_config, tempdir.as_ref()).unwrap());
            chains_synchronizer.push(chain_sync::ChainSynchronizer::new(
                node_id.clone(),
                chain_sync::ChainSyncConfig::default(),
            ));

            pending_stores.push(MemoryPendingStore::new());
            pending_stores_synchronizer.push(pending_sync::PendingSynchronizer::new(
                node_id.clone(),
                pending_sync::PendingSyncConfig::default(),
            ));

            commit_managers.push(CommitManager::new(
                node_id.clone(),
                crate::engine::commit_manager::CommitManagerConfig::default(),
                clock.clone(),
            ));

            temp_dirs.push(tempdir);
        }

        TestCluster {
            nodes,
            temp_dirs,
            clocks,
            chains,
            chains_synchronizer,
            pending_stores,
            pending_stores_synchronizer,
            commit_managers,
        }
    }

    pub fn get_node(&self, node_idx: usize) -> Node {
        self.nodes
            .get(&format!("node{}", node_idx))
            .unwrap()
            .clone()
    }

    pub fn chain_generate_dummy(&mut self, node_idx: usize, count: usize, seed: u64) {
        let mut offsets = Vec::new();
        let mut next_offset = 0;

        for i in 0..count {
            offsets.push(next_offset);

            let previous_block = if i != 0 {
                Some(
                    self.chains[node_idx]
                        .get_block_from_next_offset(next_offset)
                        .unwrap(),
                )
            } else {
                None
            };

            let prev_block_msg = previous_block.map(|b| b.block);
            let operations_data = vec![0u8; 123];
            let signatures = create_dummy_block_sigs(operations_data.len() as u32);
            let block_frame = create_dummy_block(
                next_offset,
                i as u64,
                operations_data.len() as u32,
                signatures.frame_size() as u16,
                prev_block_msg,
                seed,
            );
            let block = BlockOwned::new(next_offset, block_frame, operations_data, signatures);
            next_offset = self.chains[node_idx].write_block(&block).unwrap();
        }
    }

    pub fn pending_generate_dummy(&mut self, node_idx: usize, count: usize) {
        for operation in dummy_pending_ops_generator(count) {
            self.pending_stores[node_idx]
                .put_operation(operation)
                .unwrap();
        }
    }

    pub fn chain_add_genesis_block(&mut self, node_idx: usize) {
        let my_node = self.get_node(node_idx);
        let block = chain::BlockOwned::new_genesis(&self.nodes, &my_node).unwrap();
        self.chains[node_idx].write_block(&block).unwrap();
    }

    pub fn tick_chain_synchronizer(
        &mut self,
        node_idx: usize,
    ) -> Result<SyncContext, crate::engine::Error> {
        let mut sync_context = SyncContext::new();

        self.chains_synchronizer[node_idx].tick(
            &mut sync_context,
            &self.chains[node_idx],
            &self.nodes,
        )?;
        Ok(sync_context)
    }

    pub fn tick_commit_manager(
        &mut self,
        node_idx: usize,
    ) -> Result<SyncContext, crate::engine::Error> {
        let mut sync_context = SyncContext::new();

        self.commit_managers[node_idx].tick(
            &mut sync_context,
            &mut self.pending_stores_synchronizer[node_idx],
            &mut self.pending_stores[node_idx],
            &mut self.chains_synchronizer[node_idx],
            &mut self.chains[node_idx],
            &self.nodes,
        )?;

        Ok(sync_context)
    }

    pub fn consistent_clock(&self, node_idx: usize) -> u64 {
        self.clocks[node_idx].consistent_time(&self.get_node(node_idx))
    }
}

pub fn create_dummy_block<B: TypedFrame<block::Owned>>(
    offset: u64,
    depth: u64,
    operations_size: u32,
    signatures_size: u16,
    previous_block: Option<B>,
    seed: u64,
) -> OwnedTypedFrame<block::Owned> {
    let mut msg_builder = FrameBuilder::<block::Owned>::new();

    {
        let mut block_builder: block::Builder = msg_builder.get_builder_typed();
        block_builder.set_offset(offset);
        block_builder.set_depth(depth);
        block_builder.set_operations_size(operations_size);
        block_builder.set_signatures_size(signatures_size);
        block_builder.set_proposed_node_id(&format!("seed={}", seed));

        if let Some(previous_block) = previous_block {
            let previous_block_reader: block::Reader = previous_block.get_typed_reader().unwrap();
            block_builder.set_previous_offset(previous_block_reader.get_offset());
            block_builder.set_previous_hash(previous_block.signature_data().unwrap());
        }
    }

    let signer = MultihashFrameSigner::new_sha3256();
    msg_builder.as_owned_framed(signer).unwrap()
}

pub fn create_dummy_block_sigs(operations_size: u32) -> OwnedTypedFrame<block_signatures::Owned> {
    let mut msg_builder = FrameBuilder::<block_signatures::Owned>::new();
    let mut block_builder = msg_builder.get_builder_typed();
    block_builder.set_operations_size(operations_size);

    let signer = MultihashFrameSigner::new_sha3256();
    msg_builder.as_owned_framed(signer).unwrap()
}

pub fn dummy_pending_ops_generator(
    count: usize,
) -> impl Iterator<Item = OwnedTypedFrame<pending_operation::Owned>> {
    (1..=count).map(|i| {
        let (group_id, operation_id) = ((i % 10 + 1) as u64, i as u64);
        create_dummy_new_entry_op(operation_id, group_id)
    })
}

pub fn create_dummy_new_entry_op(
    operation_id: OperationID,
    group_id: GroupID,
) -> OwnedTypedFrame<pending_operation::Owned> {
    let mut msg_builder = FrameBuilder::<pending_operation::Owned>::new();

    {
        let mut op_builder: pending_operation::Builder = msg_builder.get_builder_typed();
        op_builder.set_group_id(group_id);
        op_builder.set_operation_id(operation_id);
        op_builder.set_node_id("node_id");

        let inner_op_builder = op_builder.init_operation();
        let mut new_entry_builder = inner_op_builder.init_entry();

        new_entry_builder.set_data(b"bob");
    }

    let frame_signer = MultihashFrameSigner::new_sha3256();
    msg_builder.as_owned_framed(frame_signer).unwrap()
}
