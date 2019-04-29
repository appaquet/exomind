#[macro_use]
extern crate log;

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use failure::err_msg;
use futures::prelude::*;
use itertools::Itertools;
use tempdir;
use tokio::runtime::Runtime;

use exocore_common::node::{Node, Nodes};
use exocore_common::serialization::protos::OperationID;
use exocore_common::tests_utils::expect_result;
use exocore_common::time::Clock;

use exocore_data::chain::{BlockOffset, BlockOwned, ChainStore};
use exocore_data::engine::{Event, Handle};
use exocore_data::operation::Operation;
use exocore_data::{
    DirectoryChainStore, DirectoryChainStoreConfig, Engine, EngineConfig, MemoryPendingStore,
    MockTransportHub, OperationStatus,
};

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
    assert_eq!(OperationStatus::Pending, entry_operation.status);

    let op2 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 2")?;
    let entry_operation = cluster.get_handle(0).get_operation(op2)?.unwrap();
    assert_eq!(b"i love rust 2", entry_operation.as_entry_data()?);
    assert_eq!(OperationStatus::Pending, entry_operation.status);

    // wait for all operations to be emitted on stream
    expect_operations_emitted(&cluster, &[op1, op2]);
    let block_offsets = expect_block_committed(&cluster);
    let first_block_offset = block_offsets.first().unwrap();

    // get operation from chain
    let entry_operation = cluster
        .get_handle(0)
        .get_chain_operation(*first_block_offset, op1)?
        .unwrap();
    assert_eq!(b"i love rust 1", entry_operation.as_entry_data()?);
    assert_eq!(OperationStatus::Committed, entry_operation.status);

    // get operation from anywhere, should not be committed
    let entry_operation = cluster.get_handle(0).get_operation(op1)?.unwrap();
    assert_eq!(b"i love rust 1", entry_operation.as_entry_data()?);
    assert_eq!(OperationStatus::Committed, entry_operation.status);

    let entry_operation = cluster.get_handle(0).get_operation(op2)?.unwrap();
    assert_eq!(b"i love rust 2", entry_operation.as_entry_data()?);
    assert_eq!(OperationStatus::Committed, entry_operation.status);

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
    expect_block_committed(&cluster);

    // make sure operation is in chain
    let entry_before = cluster.get_handle(0).get_operation(op1)?.unwrap();
    assert_eq!(OperationStatus::Committed, entry_before.status);

    // stop and restart node
    cluster.stop_node(0);
    cluster.create_node(0)?;
    cluster.start_engine(0);
    cluster.collect_events_stream(0);
    cluster.wait_started(0);

    // data should still exist
    let entry_before = cluster.get_handle(0).get_operation(op1)?.unwrap();
    assert_eq!(OperationStatus::Committed, entry_before.status);

    Ok(())
}

#[test]
fn two_nodes_simple_replication() -> Result<(), failure::Error> {
    let mut cluster = TestCluster::new(2)?;
    cluster.create_node(0)?;
    cluster.create_node(1)?;

    cluster.create_chain_genesis_block(0);

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
    let op2 = cluster
        .get_handle_mut(1)
        .write_entry_operation(b"i love rust 1")?;

    // wait for both nodes to have the operation committed locally
    expect_result::<_, _, failure::Error>(|| {
        // op 1 should now be on node 2
        cluster
            .get_handle(1)
            .get_operation(op1)?
            .filter(|op| op.status == OperationStatus::Committed)
            .ok_or_else(|| err_msg("Operation not on node"))?;

        // op 0 should now be on node 1
        cluster
            .get_handle(0)
            .get_operation(op2)?
            .filter(|op| op.status == OperationStatus::Committed)
            .ok_or_else(|| err_msg("Operation not on node"))?;

        Ok(())
    });

    // chain should be the same on both node with operations committed
    let segments_0 = cluster.get_handle(0).get_chain_segments()?;
    let segments_1 = cluster.get_handle(1).get_chain_segments()?;
    assert_eq!(segments_0, segments_1);

    Ok(())
}

#[test]
fn dont_replicate_operations_until_chain_sync() {
    // TODO:
}

#[test]
fn dont_replicate_committed_operations() {
    // TODO: Make node accept operation
    // TODO: Make node go offline
    // TODO: Wait for rest of node commit
    // TODO: Make node come back online
    // TODO: Don't expect nodes to get operations back in their pending store
}

///
///
///
struct TestCluster {
    tempdir: tempdir::TempDir,
    runtime: Runtime,
    nodes: Nodes,
    transport_hub: MockTransportHub,
    clock: Clock,

    chain_stores: Vec<Option<DirectoryChainStore>>,
    pending_stores: Vec<Option<MemoryPendingStore>>,
    handles: Vec<Option<Handle<DirectoryChainStore, MemoryPendingStore>>>,

    events_receiver: Vec<Option<mpsc::Receiver<Event>>>,
    events_received: Vec<Option<Arc<Mutex<Vec<Event>>>>>,
}

impl TestCluster {
    fn new(count: usize) -> Result<TestCluster, failure::Error> {
        let tempdir = tempdir::TempDir::new("engine_tests")?;

        let runtime = Runtime::new()?;
        let mut nodes = Nodes::new();

        let transport_hub = MockTransportHub::default();
        let clock = Clock::new();

        let mut handles = Vec::new();
        let mut chain_stores = Vec::new();
        let mut pending_stores = Vec::new();

        let mut events_receiver = Vec::new();
        let mut events_received = Vec::new();

        for node_id in 0..count {
            let node = Node::new(format!("node{}", node_id));
            nodes.add(node.clone());

            chain_stores.push(None);
            pending_stores.push(None);
            handles.push(None);
            events_receiver.push(None);
            events_received.push(None);
        }

        Ok(TestCluster {
            tempdir,
            runtime,
            nodes,
            transport_hub,
            clock,

            chain_stores,
            pending_stores,
            handles,

            events_receiver,
            events_received,
        })
    }

    fn node_data_dir(&self, node_idx: usize) -> PathBuf {
        let node = self.get_node(node_idx);
        self.tempdir.path().join(node.id().to_string())
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
        let my_node = self.get_node(node_idx);
        let block = BlockOwned::new_genesis(&self.nodes, &my_node).unwrap();
        self.chain_stores[node_idx]
            .as_mut()
            .unwrap()
            .write_block(&block)
            .unwrap();
    }

    fn start_engine(&mut self, node_idx: usize) {
        let engine_config = EngineConfig {
            manager_timer_interval: Duration::from_millis(100),
            ..EngineConfig::default()
        };

        self.start_engine_with_config(node_idx, engine_config);
    }

    fn start_engine_with_config(&mut self, node_idx: usize, engine_config: EngineConfig) {
        let node = self.get_node(node_idx);

        let transport = self.transport_hub.get_transport(node.clone());

        let mut engine = Engine::new(
            engine_config,
            node.id().to_string(),
            self.clock.clone(),
            transport,
            self.chain_stores[node_idx].take().unwrap(),
            self.pending_stores[node_idx].take().unwrap(),
            self.nodes.clone(),
        );

        let engine_handle = engine.get_handle();
        self.handles[node_idx] = Some(engine_handle);

        self.runtime
            .spawn(engine.map_err(|err| error!("Got an error in engine: {:?}", err)));
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

    fn get_node(&self, node_idx: usize) -> Node {
        self.nodes
            .get(&format!("node{}", node_idx))
            .unwrap()
            .clone()
    }

    fn get_received_events(&self, node_idx: usize) -> Vec<Event> {
        let events_locked = self.events_received[node_idx].as_ref().unwrap();
        let events = events_locked.lock().unwrap();
        events.clone()
    }

    fn get_handle(&self, node_idx: usize) -> &Handle<DirectoryChainStore, MemoryPendingStore> {
        self.handles[node_idx].as_ref().unwrap()
    }

    fn get_handle_mut(
        &mut self,
        node_idx: usize,
    ) -> &mut Handle<DirectoryChainStore, MemoryPendingStore> {
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

    fn stop_node(&mut self, node_idx: usize) {
        self.chain_stores[node_idx] = None;
        self.pending_stores[node_idx] = None;
        self.handles[node_idx] = None;

        self.events_received[node_idx] = None;
        self.events_receiver[node_idx] = None;
    }
}

fn extract_ops_events(events: &[Event]) -> Vec<OperationID> {
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

fn expect_block_committed(cluster: &TestCluster) -> Vec<BlockOffset> {
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
