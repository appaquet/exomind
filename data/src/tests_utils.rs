use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use failure::err_msg;
use futures::prelude::*;
use itertools::Itertools;
use tempdir;
use tokio::runtime::Runtime;

use exocore_common::node::LocalNode;
use exocore_common::tests_utils::expect_result;
use exocore_common::time::Clock;

use crate::block::{BlockOffset, BlockOwned};
use crate::chain::ChainStore;
use crate::engine::{EngineHandle, EngineOperation, Event};
use crate::operation::OperationId;
use crate::*;
use exocore_common::cell::FullCell;
use exocore_transport::mock::MockTransport;

///
/// exocore-data testing utility
///
pub struct DataTestCluster {
    pub tempdir: tempdir::TempDir,
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

impl DataTestCluster {
    pub fn new(count: usize) -> Result<DataTestCluster, failure::Error> {
        let tempdir = tempdir::TempDir::new("engine_tests")?;

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

        for _node_idx in 0..count {
            let local_node = LocalNode::generate();
            let cell = FullCell::generate(local_node.clone());
            nodes.push(local_node);
            cells.push(cell);

            let clock = Clock::new_mocked();
            clocks.push(clock);

            let engine_config = EngineConfig {
                manager_timer_interval: Duration::from_millis(100),
                commit_manager_config: CommitManagerConfig {
                    commit_maximum_interval: Duration::from_millis(200),
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

        // add each node to all other nodes' cell
        for cell in &cells {
            for other_node in &nodes {
                if cell.local_node().id() != other_node.id() {
                    let mut cell_nodes = cell.nodes_mut();
                    cell_nodes.add(other_node.node().clone());
                }
            }
        }

        Ok(DataTestCluster {
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
        })
    }

    pub fn node_data_dir(&self, node_idx: usize) -> PathBuf {
        self.tempdir.path().join(self.nodes[node_idx].id().to_str())
    }

    pub fn create_node(&mut self, node_idx: usize) -> Result<(), failure::Error> {
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

    pub fn start_engine(&mut self, node_idx: usize) {
        let transport = self
            .transport_hub
            .get_transport(self.nodes[node_idx].clone());

        let mut engine = Engine::new(
            self.engines_config[node_idx],
            self.clocks[node_idx].clone(),
            transport,
            self.chain_stores[node_idx].take().unwrap(),
            self.pending_stores[node_idx].take().unwrap(),
            self.cells[node_idx].cell().clone(),
        );

        let engine_handle = engine.get_handle();
        self.handles[node_idx] = Some(engine_handle);

        self.runtime
            .spawn(engine.map_err(|err| error!("Got an error in engine: {}", err)));
    }

    pub fn wait_started(&self, node_idx: usize) {
        self.wait_any_event(node_idx);
    }

    pub fn collect_events_stream(&mut self, node_idx: usize) {
        let (events_sender, events_receiver) = mpsc::channel();
        let received_events = Arc::new(Mutex::new(Vec::new()));

        {
            let events = Arc::clone(&received_events);
            self.runtime.spawn(
                self.handles[node_idx]
                    .as_mut()
                    .unwrap()
                    .take_events_stream()
                    .for_each(move |event| {
                        let mut events = events.lock().unwrap();
                        events.push(event.clone());
                        events_sender.send(event.clone()).unwrap();
                        Ok(())
                    })
                    .map_err(|_| ()),
            );
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
            // not performant, but it's for tests
            Some(events.remove(0))
        } else {
            None
        }
    }

    pub fn clear_received_events(&self, node_idx: usize) {
        let events_locked = self.events_received[node_idx].as_ref().unwrap();
        let mut events = events_locked.lock().unwrap();
        events.clear();
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
                .recv_timeout(Duration::from_secs(5))
                .unwrap();

            if predicate(&event) {
                return event;
            }
        }
    }

    pub fn wait_operation_committed(
        &self,
        node_idx: usize,
        operation_id: OperationId,
    ) -> EngineOperation {
        expect_result::<_, _, failure::Error>(|| {
            self.get_handle(node_idx)
                .get_operation(operation_id)?
                .filter(|op| op.status.is_committed())
                .ok_or_else(|| err_msg("Operation not on node"))
        })
    }

    pub fn wait_next_block_commit(&self, node_idx: usize) -> Vec<BlockOffset> {
        expect_result::<_, _, failure::Error>(|| {
            let events = self.get_received_events(node_idx);
            let offsets = extract_blocks_events(&events);

            if !offsets.is_empty() {
                Ok(offsets)
            } else {
                Err(failure::err_msg("No block found".to_string()))
            }
        })
    }

    pub fn restart_node(&mut self, node_idx: usize) -> Result<(), failure::Error> {
        let was_collecting_events = self.events_received[node_idx].is_some();

        self.stop_node(node_idx);
        self.create_node(node_idx)?;
        self.start_engine(node_idx);

        if was_collecting_events {
            self.collect_events_stream(node_idx);
            self.wait_started(node_idx);
        }

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

pub fn expect_operations_emitted(cluster: &DataTestCluster, expected_ops: &[u64]) {
    expect_result::<_, _, failure::Error>(|| {
        let events = cluster.get_received_events(0);
        let found_ops = extract_ops_events(&events);

        if (&expected_ops).iter().all(|op| found_ops.contains(op)) {
            Ok(found_ops)
        } else {
            Err(failure::err_msg(format!(
                "Not all ops found: found={:?} expected={:?}",
                found_ops, &expected_ops
            )))
        }
    });
}
