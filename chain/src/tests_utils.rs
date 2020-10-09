use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::prelude::*;
use itertools::Itertools;
use tempfile::TempDir;

use exocore_core::cell::{CellNode, CellNodeRole, LocalNode};
use exocore_core::futures::Runtime;
use exocore_core::tests_utils::expect_result_eventually;
use exocore_core::time::Clock;

use crate::block::{BlockOffset, BlockOwned};
use crate::chain::ChainStore;
use crate::engine::{EngineHandle, EngineOperation, Event, RequestTrackerConfig};
use crate::operation::OperationId;
use crate::*;
use exocore_core::cell::FullCell;
use exocore_transport::testing::MockTransport;
use exocore_transport::ServiceType;

/// exocore-chain testing utility
pub struct TestChainCluster {
    pub tempdir: TempDir,
    pub runtime: Runtime,
    pub transport_hub: MockTransport,

    pub nodes: Vec<LocalNode>,
    pub cells: Vec<FullCell>,
    pub clocks: Vec<Clock>,

    pub engines_config: Vec<EngineConfig>,
    pub chain_stores: Vec<Option<DirectoryChainStore>>,
    pub pending_stores: Vec<Option<MemoryPendingStore>>,
    pub handles: Vec<Option<EngineHandle<DirectoryChainStore, MemoryPendingStore>>>,

    pub events_receiver: Vec<Option<mpsc::Receiver<Event>>>,
    pub events_received: Vec<Option<Arc<Mutex<Vec<Event>>>>>,
}

pub struct ClusterSpec {
    pub full_chain_nodes: usize,
    pub chain_nodes: usize,
}

impl TestChainCluster {
    pub fn new(count: usize) -> Result<TestChainCluster, anyhow::Error> {
        let tempdir = tempfile::tempdir()?;

        let runtime = Runtime::new()?;

        let transport_hub = MockTransport::default();

        let mut clocks = Vec::new();
        let mut nodes = Vec::new();
        let mut cells = Vec::new();
        let mut engines_config = Vec::new();
        let mut handles = Vec::new();
        let mut chain_stores = Vec::new();
        let mut pending_stores = Vec::new();

        let mut events_receiver = Vec::new();
        let mut events_received = Vec::new();

        for node_idx in 0..count {
            let local_node = LocalNode::generate();
            let cell = FullCell::generate(local_node.clone())
                .with_path(tempdir.path().join(format!("{}", node_idx)));
            nodes.push(local_node);
            cells.push(cell);

            let clock = Clock::new_mocked();
            clocks.push(clock);

            let engine_config = EngineConfig {
                manager_timer_interval: Duration::from_millis(20),
                pending_sync_config: PendingSyncConfig {
                    request_tracker_config: RequestTrackerConfig {
                        min_interval: Duration::from_millis(200),
                        max_interval: Duration::from_millis(1000),
                    },
                    ..PendingSyncConfig::default()
                },
                commit_manager_config: CommitManagerConfig {
                    commit_maximum_interval: Duration::from_millis(100),
                    block_proposal_timeout: Duration::from_millis(333),
                    ..CommitManagerConfig::default()
                },
                ..EngineConfig::default()
            };
            engines_config.push(engine_config);

            chain_stores.push(None);
            pending_stores.push(None);
            handles.push(None);
            events_receiver.push(None);
            events_received.push(None);
        }

        // Add each node to all other nodes' cell
        // All nodes have full chain access
        for cell in &cells {
            let mut cell_nodes = cell.nodes_mut();

            for node in &nodes {
                if cell.local_node().id() != node.id() {
                    let mut cell_node = CellNode::new(node.node().clone());
                    cell_node.add_role(CellNodeRole::Chain);
                    cell_nodes.add_cell_node(cell_node);
                } else {
                    cell_nodes
                        .local_cell_node_mut()
                        .add_role(CellNodeRole::Chain);
                }
            }
        }

        let mut cluster = TestChainCluster {
            tempdir,
            runtime,
            nodes,
            transport_hub,
            clocks,

            cells,
            engines_config,
            chain_stores,
            pending_stores,
            handles,

            events_receiver,
            events_received,
        };

        for i in 0..count {
            cluster.add_node_role(i, CellNodeRole::Chain);
        }

        Ok(cluster)
    }

    pub fn new_single_and_start() -> Result<TestChainCluster, anyhow::Error> {
        let mut cluster = TestChainCluster::new(1)?;

        cluster.create_node(0)?;
        cluster.create_chain_genesis_block(0);
        cluster.start_engine(0);

        // wait for engine to start
        cluster.wait_started(0);

        Ok(cluster)
    }

    pub fn node_data_dir(&self, node_idx: usize) -> PathBuf {
        self.tempdir
            .path()
            .join(self.nodes[node_idx].id().to_string())
    }

    pub fn create_node(&mut self, node_idx: usize) -> anyhow::Result<()> {
        let data_dir = self.node_data_dir(node_idx);
        let data_exists = std::fs::metadata(&data_dir).is_ok();

        if !data_exists {
            std::fs::create_dir(&data_dir)?;
        }

        let chain_config = DirectoryChainStoreConfig::default();
        let chain = if !data_exists {
            DirectoryChainStore::create(chain_config, &data_dir)?
        } else {
            DirectoryChainStore::open(chain_config, &data_dir)?
        };
        self.chain_stores[node_idx] = Some(chain);

        let pending_store = MemoryPendingStore::new();
        self.pending_stores[node_idx] = Some(pending_store);

        Ok(())
    }

    pub fn create_chain_genesis_block(&mut self, node_idx: usize) {
        let block = BlockOwned::new_genesis(&self.cells[node_idx]).unwrap();
        self.chain_stores[node_idx]
            .as_mut()
            .unwrap()
            .write_block(&block)
            .unwrap();
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

    pub fn start_engine(&mut self, node_idx: usize) {
        let transport = self
            .transport_hub
            .get_transport(self.nodes[node_idx].clone(), ServiceType::Chain);

        let mut engine = Engine::new(
            self.engines_config[node_idx].clone(),
            self.clocks[node_idx].clone(),
            transport,
            self.chain_stores[node_idx].take().unwrap(),
            self.pending_stores[node_idx].take().unwrap(),
            self.cells[node_idx].cell().clone(),
        );

        let engine_handle = engine.get_handle();
        self.handles[node_idx] = Some(engine_handle);

        self.collect_events_stream(node_idx);

        self.runtime.spawn(async {
            let res = engine.run().await;
            info!("Engine done: {:?}", res);
        });
    }

    pub fn wait_started(&self, node_idx: usize) {
        self.wait_any_event(node_idx);
    }

    pub fn collect_events_stream(&mut self, node_idx: usize) {
        let (events_sender, events_receiver) = mpsc::channel();
        let received_events = Arc::new(Mutex::new(Vec::new()));

        {
            let events = Arc::clone(&received_events);
            let mut stream_events = self.handles[node_idx]
                .as_mut()
                .unwrap()
                .take_events_stream()
                .unwrap();
            self.runtime.spawn(async move {
                while let Some(event) = stream_events.next().await {
                    let mut events = events.lock().unwrap();
                    events.push(event.clone());
                    events_sender.send(event).unwrap();
                }
            });
        }

        self.events_received[node_idx] = Some(received_events);
        self.events_receiver[node_idx] = Some(events_receiver);
    }

    pub fn get_received_events(&self, node_idx: usize) -> Vec<Event> {
        let events_locked = self.events_received[node_idx].as_ref().unwrap();
        let events = events_locked.lock().unwrap();
        events.clone()
    }

    pub fn pop_received_event(&self, node_idx: usize) -> Option<Event> {
        let events_locked = self.events_received[node_idx].as_ref().unwrap();
        let mut events = events_locked.lock().unwrap();
        if !events.is_empty() {
            // slow, but it's for tests
            Some(events.remove(0))
        } else {
            None
        }
    }

    pub fn drain_received_events(&self, node_idx: usize) -> Vec<Event> {
        let events_locked = self.events_received[node_idx].as_ref().unwrap();
        let mut events = events_locked.lock().unwrap();
        std::mem::replace(events.as_mut(), Vec::new())
    }

    pub fn get_handle(
        &self,
        node_idx: usize,
    ) -> &EngineHandle<DirectoryChainStore, MemoryPendingStore> {
        self.handles[node_idx].as_ref().unwrap()
    }

    pub fn get_handle_mut(
        &mut self,
        node_idx: usize,
    ) -> &mut EngineHandle<DirectoryChainStore, MemoryPendingStore> {
        self.handles[node_idx].as_mut().unwrap()
    }

    pub fn get_new_handle(
        &self,
        node_idx: usize,
    ) -> EngineHandle<DirectoryChainStore, MemoryPendingStore> {
        self.handles[node_idx].as_ref().unwrap().clone()
    }

    pub fn wait_any_event(&self, node_idx: usize) -> Event {
        self.wait_for_event(node_idx, |_| true)
    }

    pub fn wait_for_event<F>(&self, node_idx: usize, predicate: F) -> Event
    where
        F: Fn(&Event) -> bool,
    {
        loop {
            let event = self.events_receiver[node_idx]
                .as_ref()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .unwrap();

            if predicate(&event) {
                return event;
            }
        }
    }

    pub fn wait_operations_emitted(&self, node_idx: usize, operations_id: &[u64]) {
        expect_result_eventually::<_, _, anyhow::Error>(|| {
            let events = self.get_received_events(node_idx);
            let found_ops = extract_ops_events(&events);

            if (&operations_id).iter().all(|op| found_ops.contains(op)) {
                Ok(found_ops)
            } else {
                Err(anyhow!(format!(
                    "Not all ops found: found={:?} expected={:?}",
                    found_ops, &operations_id
                )))
            }
        });
    }

    pub fn wait_operation_committed(
        &self,
        node_idx: usize,
        operation_id: OperationId,
    ) -> EngineOperation {
        expect_result_eventually::<_, _, anyhow::Error>(|| {
            self.get_handle(node_idx)
                .get_operation(operation_id)?
                .filter(|op| op.status.is_committed())
                .ok_or_else(|| anyhow!("Operation not on node"))
        })
    }

    pub fn wait_operations_committed(&self, node_idx: usize, operations_id: &[OperationId]) {
        expect_result_eventually::<_, _, anyhow::Error>(|| {
            for operation_id in operations_id {
                self.get_handle(node_idx)
                    .get_operation(*operation_id)?
                    .filter(|op| op.status.is_committed())
                    .ok_or_else(|| anyhow!("Operation not on node"))?;
            }

            Ok(())
        });
    }

    pub fn wait_operations_exist<I>(&self, node_idx: usize, operations_id: &[OperationId]) {
        expect_result_eventually::<_, _, anyhow::Error>(|| {
            for operation_id in operations_id {
                self.get_handle(node_idx)
                    .get_operation(*operation_id)?
                    .ok_or_else(|| anyhow!("Operation not on node"))?;
            }

            Ok(())
        });
    }

    pub fn wait_next_block_commit(&self, node_idx: usize) -> Vec<BlockOffset> {
        expect_result_eventually::<_, _, anyhow::Error>(|| {
            let events = self.get_received_events(node_idx);
            let offsets = extract_blocks_events(&events);

            if !offsets.is_empty() {
                Ok(offsets)
            } else {
                Err(anyhow!("No block found".to_string()))
            }
        })
    }

    pub fn restart_node(&mut self, node_idx: usize) -> anyhow::Result<()> {
        self.stop_node(node_idx);
        self.create_node(node_idx)?;
        self.start_engine(node_idx);

        let handle = self.handles[node_idx].as_ref().unwrap().clone();
        self.runtime.block_on(handle.on_started());

        Ok(())
    }

    pub fn stop_node(&mut self, node_idx: usize) {
        self.chain_stores[node_idx] = None;
        self.pending_stores[node_idx] = None;
        self.handles[node_idx] = None;

        self.events_received[node_idx] = None;
        self.events_receiver[node_idx] = None;
    }
}

pub fn extract_ops_events(events: &[Event]) -> Vec<OperationId> {
    events
        .iter()
        .flat_map(|event| match event {
            Event::NewPendingOperation(op) => Some(*op),
            _ => None,
        })
        .sorted()
        .collect()
}

pub fn extract_blocks_events(events: &[Event]) -> Vec<BlockOffset> {
    events
        .iter()
        .flat_map(|event| match event {
            Event::NewChainBlock(offset) => Some(*offset),
            _ => None,
        })
        .sorted()
        .collect()
}
