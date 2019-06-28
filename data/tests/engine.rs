#[macro_use]
extern crate log;

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use failure::err_msg;
use futures::prelude::*;
use itertools::Itertools;
use tempdir;
use tokio::runtime::Runtime;

use exocore_common::node::LocalNode;
use exocore_common::tests_utils::expect_result;
use exocore_common::time::Clock;

use exocore_common::cell::FullCell;
use exocore_data::block::{BlockOffset, BlockOwned};
use exocore_data::chain::ChainStore;
use exocore_data::engine::{EngineHandle, EngineOperation, Event};
use exocore_data::operation::{Operation, OperationId};
use exocore_data::*;
use exocore_transport::mock::MockTransport;

// TODO: To be completed in https://github.com/appaquet/exocore/issues/42

#[test]
fn single_node_full_chain_write_read() -> Result<(), failure::Error> {
    let mut cluster = TestCluster::new(1)?;
    cluster.create_node(0)?;
    cluster.create_chain_genesis_block(0);
    cluster.start_engine(0);

    // wait for engine to start
    cluster.collect_events_stream(0);
    cluster.wait_started(0);

    let op1 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 1")?;
    let entry_operation = cluster.get_handle(0).get_operation(op1)?.unwrap();
    assert_eq!(b"i love rust 1", entry_operation.as_entry_data()?);
    assert_eq!(EngineOperationStatus::Pending, entry_operation.status);

    let op2 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 2")?;
    let entry_operation = cluster.get_handle(0).get_operation(op2)?.unwrap();
    assert_eq!(b"i love rust 2", entry_operation.as_entry_data()?);
    assert_eq!(EngineOperationStatus::Pending, entry_operation.status);

    // wait for all operations to be emitted on stream
    expect_operations_emitted(&cluster, &[op1, op2]);
    let block_offsets = wait_next_block_commit(&cluster);
    let first_block_offset = block_offsets.first().unwrap();

    // get operation from chain
    let entry_operation = cluster
        .get_handle(0)
        .get_chain_operation(*first_block_offset, op1)?
        .unwrap();
    assert_eq!(b"i love rust 1", entry_operation.as_entry_data()?);
    assert!(entry_operation.status.is_committed());

    // get operation from anywhere, should not be committed
    let entry_operation = cluster.get_handle(0).get_operation(op1)?.unwrap();
    assert_eq!(b"i love rust 1", entry_operation.as_entry_data()?);
    assert!(entry_operation.status.is_committed());

    let entry_operation = cluster.get_handle(0).get_operation(op2)?.unwrap();
    assert_eq!(b"i love rust 2", entry_operation.as_entry_data()?);
    assert!(entry_operation.status.is_committed());

    // test pending operations range
    let operations = cluster.get_handle(0).get_pending_operations(..)?;
    let ops_id = operations
        .iter()
        .map(|op| op.operation_id)
        .sorted()
        .collect_vec();
    assert!(ops_id.contains(&op1));
    assert!(ops_id.contains(&op2));

    Ok(())
}

#[test]
fn single_node_chain_iteration() -> Result<(), failure::Error> {
    let mut cluster = TestCluster::new(1)?;
    cluster.create_node(0)?;
    cluster.create_chain_genesis_block(0);
    cluster.start_engine(0);

    // wait for engine to start
    cluster.collect_events_stream(0);
    cluster.wait_started(0);

    let chain_operations = cluster.get_handle(0).get_chain_operations(None);
    assert_eq!(0, chain_operations.count());

    let op1 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 1")?;
    let op2 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 2")?;
    wait_next_block_commit(&cluster);

    let chain_operations = cluster
        .get_handle(0)
        .get_chain_operations(None)
        .collect_vec();
    assert_eq!(2, chain_operations.len());
    let op_reader = chain_operations[0].operation_frame.get_reader()?;
    assert_eq!(op1, op_reader.get_operation_id());
    let op_reader = chain_operations[1].operation_frame.get_reader()?;
    assert_eq!(op2, op_reader.get_operation_id());

    Ok(())
}

#[test]
fn single_node_restart() -> Result<(), failure::Error> {
    let mut cluster = TestCluster::new(1)?;
    cluster.create_node(0)?;
    cluster.create_chain_genesis_block(0);
    cluster.start_engine(0);

    cluster.collect_events_stream(0);
    cluster.wait_started(0);

    // wait for all operations to be emitted on stream
    let op1 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 1")?;
    expect_operations_emitted(&cluster, &[op1]);

    // wait for operations to be committed
    wait_next_block_commit(&cluster);

    // make sure operation is in chain
    let entry_before = cluster.get_handle(0).get_operation(op1)?.unwrap();
    assert!(entry_before.status.is_committed());

    // stop and restart node
    cluster.restart_node(0)?;

    // committed data should still exist
    let entry_after = cluster.get_handle(0).get_operation(op1)?.unwrap();
    assert!(entry_after.status.is_committed());

    Ok(())
}

#[test]
fn two_nodes_full_replication() -> Result<(), failure::Error> {
    let mut cluster = TestCluster::new(2)?;
    cluster.create_node(0)?;
    cluster.create_node(1)?;

    cluster.create_chain_genesis_block(0);

    cluster.engines_config[0]
        .commit_manager_config
        .commit_maximum_pending_count = 1;

    cluster.start_engine(0);
    cluster.start_engine(1);
    cluster.collect_events_stream(0);
    cluster.collect_events_stream(1);
    cluster.wait_started(0);
    cluster.wait_started(1);

    // add operation on each nodes
    let op1 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 0")?;

    // TODO: We need to sleep because the 2 nodes may generate same operation id until https://github.com/appaquet/exocore/issues/6
    std::thread::sleep(Duration::from_millis(10));

    let op2 = cluster
        .get_handle_mut(1)
        .write_entry_operation(b"i love rust 1")?;

    // wait for both nodes to have the operation committed locally
    cluster.wait_operation_committed(0, op2);
    cluster.wait_operation_committed(1, op1);

    // chain should be the same on both node with operations committed
    let segments_0 = cluster.get_handle(0).get_chain_segments()?;
    let segments_1 = cluster.get_handle(1).get_chain_segments()?;
    assert_eq!(segments_0, segments_1);

    Ok(())
}

#[test]
fn two_nodes_pending_store_cleanup() -> Result<(), failure::Error> {
    let mut cluster = TestCluster::new(2)?;
    cluster.create_node(0)?;
    cluster.create_node(1)?;

    cluster.create_chain_genesis_block(0);

    // we let node 0 commit every second
    cluster.engines_config[0]
        .commit_manager_config
        .commit_maximum_interval = Duration::from_millis(500);

    // both nodes will cleanup after 2 depth
    cluster.engines_config[0]
        .commit_manager_config
        .operations_cleanup_after_block_depth = 2;
    cluster.engines_config[1]
        .commit_manager_config
        .operations_cleanup_after_block_depth = 2;

    cluster.start_engine(0);
    cluster.start_engine(1);
    cluster.collect_events_stream(0);
    cluster.collect_events_stream(1);
    cluster.wait_started(0);
    cluster.wait_started(1);

    cluster.clocks[0].set_fixed_instant(Instant::now());
    let mut operations_id = Vec::new();
    for _i in 0..=2 {
        let op_id = cluster
            .get_handle(0)
            .write_entry_operation(b"i love rust")?;
        operations_id.push(op_id);

        // advance clock by 2 secs, which should trigger node 0 to commit
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(2));

        // wait for operation to be committed on node 1
        cluster.wait_operation_committed(1, op_id);
    }

    // first operation should not be in pending store anymore as it got cleaned up
    let first_op = operations_id.first().unwrap();
    expect_result::<_, _, failure::Error>(|| {
        let node1_op = cluster.get_handle(0).get_pending_operation(*first_op)?;
        if node1_op.is_some() {
            return Err(err_msg("Was still on node 0"));
        }

        let node1_op = cluster.get_handle(1).get_pending_operation(*first_op)?;
        if node1_op.is_some() {
            return Err(err_msg("Was still on node 1"));
        }

        Ok(())
    });

    Ok(())
}

///
/// Cluster testing utility
///
struct TestCluster {
    tempdir: tempdir::TempDir,
    runtime: Runtime,
    transport_hub: MockTransport,

    nodes: Vec<LocalNode>,
    cells: Vec<FullCell>,
    clocks: Vec<Clock>,

    engines_config: Vec<EngineConfig>,
    chain_stores: Vec<Option<DirectoryChainStore>>,
    pending_stores: Vec<Option<MemoryPendingStore>>,
    handles: Vec<Option<EngineHandle<DirectoryChainStore, MemoryPendingStore>>>,

    events_receiver: Vec<Option<mpsc::Receiver<Event>>>,
    events_received: Vec<Option<Arc<Mutex<Vec<Event>>>>>,
}

impl TestCluster {
    fn new(count: usize) -> Result<TestCluster, failure::Error> {
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

        Ok(TestCluster {
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

    fn node_data_dir(&self, node_idx: usize) -> PathBuf {
        self.tempdir.path().join(self.nodes[node_idx].id().to_str())
    }

    fn create_node(&mut self, node_idx: usize) -> Result<(), failure::Error> {
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

    fn create_chain_genesis_block(&mut self, node_idx: usize) {
        let block = BlockOwned::new_genesis(&self.cells[node_idx]).unwrap();
        self.chain_stores[node_idx]
            .as_mut()
            .unwrap()
            .write_block(&block)
            .unwrap();
    }

    fn start_engine(&mut self, node_idx: usize) {
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

    fn wait_started(&self, node_idx: usize) {
        self.wait_any_event(node_idx);
    }

    fn collect_events_stream(&mut self, node_idx: usize) {
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

    fn get_received_events(&self, node_idx: usize) -> Vec<Event> {
        let events_locked = self.events_received[node_idx].as_ref().unwrap();
        let events = events_locked.lock().unwrap();
        events.clone()
    }

    fn get_handle(
        &self,
        node_idx: usize,
    ) -> &EngineHandle<DirectoryChainStore, MemoryPendingStore> {
        self.handles[node_idx].as_ref().unwrap()
    }

    fn get_handle_mut(
        &mut self,
        node_idx: usize,
    ) -> &mut EngineHandle<DirectoryChainStore, MemoryPendingStore> {
        self.handles[node_idx].as_mut().unwrap()
    }

    fn wait_any_event(&self, node_idx: usize) -> Event {
        self.wait_for_event(node_idx, |_| true)
    }

    fn wait_for_event<F>(&self, node_idx: usize, predicate: F) -> Event
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

    fn wait_operation_committed(
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

    fn restart_node(&mut self, node_idx: usize) -> Result<(), failure::Error> {
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

    fn stop_node(&mut self, node_idx: usize) {
        self.chain_stores[node_idx] = None;
        self.pending_stores[node_idx] = None;
        self.handles[node_idx] = None;

        self.events_received[node_idx] = None;
        self.events_receiver[node_idx] = None;
    }
}

fn extract_ops_events(events: &[Event]) -> Vec<OperationId> {
    events
        .iter()
        .flat_map(|event| match event {
            Event::PendingOperationNew(op) => Some(*op),
            _ => None,
        })
        .sorted()
        .collect()
}

fn extract_blocks_events(events: &[Event]) -> Vec<BlockOffset> {
    events
        .iter()
        .flat_map(|event| match event {
            Event::ChainBlockNew(offset) => Some(*offset),
            _ => None,
        })
        .sorted()
        .collect()
}

fn expect_operations_emitted(cluster: &TestCluster, expected_ops: &[u64]) {
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

fn wait_next_block_commit(cluster: &TestCluster) -> Vec<BlockOffset> {
    expect_result::<_, _, failure::Error>(|| {
        let events = cluster.get_received_events(0);
        let offsets = extract_blocks_events(&events);

        if !offsets.is_empty() {
            Ok(offsets)
        } else {
            Err(failure::err_msg("No block found".to_string()))
        }
    })
}
