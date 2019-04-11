#[macro_use]
extern crate log;

use futures::prelude::*;
use tempdir;
use tokio::runtime::Runtime;

use exocore_common::node::{Node, Nodes};
use exocore_common::serialization::framed::TypedFrame;
use exocore_common::serialization::protos::data_chain_capnp::pending_operation;
use exocore_common::serialization::protos::OperationID;
use exocore_common::tests_utils::expect_result;
use exocore_common::time::Clock;
use exocore_data::chain::{BlockOffset, BlockOwned, ChainStore};
use exocore_data::engine::{Event, Handle};
use exocore_data::{
    DirectoryChainStore, DirectoryChainStoreConfig, Engine, EngineConfig, MemoryPendingStore,
    MockTransportHub,
};
use failure::err_msg;
use itertools::Itertools;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// TODO: To be completed in https://github.com/appaquet/exocore/issues/42

#[test]
fn test_engine_integration_single_node() -> Result<(), failure::Error> {
    //exocore_common::utils::setup_logging();

    let mut cluster = TestCluster::new(1)?;
    cluster.chain_add_genesis_block(0);
    cluster.start_engine(0);

    // wait for engine to start
    let (events_received, events_receiver) = cluster.extract_events_stream(0);
    events_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();

    let op1 = cluster.get_handle(0).write_entry(b"i love jello 1")?;
    let op2 = cluster.get_handle(0).write_entry(b"i love jello 2")?;
    let op3 = cluster.get_handle(0).write_entry(b"i love jello 3")?;
    let op4 = cluster.get_handle(0).write_entry(b"i love jello 4")?;

    // check if we got all operations in stream
    expect_result::<_, _, failure::Error>(|| {
        let events = events_received.lock().unwrap();

        let found_ops = extract_ops_events(&events);
        let expected_ops = vec![op1, op2, op3, op4];

        if expected_ops.iter().all(|op| found_ops.contains(op)) {
            Ok(found_ops)
        } else {
            Err(failure::err_msg(format!(
                "Not all ops found: found={:?} expected={:?}",
                found_ops, expected_ops
            )))
        }
    });

    let block_offsets = expect_result::<_, _, failure::Error>(|| {
        let events = events_received.lock().unwrap();
        let offsets = extract_blocks_events(&events);

        if !offsets.is_empty() {
            Ok(offsets)
        } else {
            Err(failure::err_msg("No block found".to_string()))
        }
    });

    let first_block_offset = block_offsets.first().unwrap();

    let pending_operations = cluster.get_handle(0).get_pending_operations(..)?;
    let segments = cluster.get_handle(0).get_chain_segments()?;
    let entry = cluster
        .get_handle(0)
        .get_chain_entry(*first_block_offset, op1)?;
    info!("Got {} pending op", pending_operations.len());
    info!("Available segments: {:?}", segments);
    info!(
        "Chain op: {:?}",
        String::from_utf8_lossy(entry.operation_frame.frame_data())
    );

    Ok(())
}

#[test]
fn test_engine_integration_replicate_genesis() -> Result<(), failure::Error> {
    //exocore_common::utils::setup_logging();

    let mut cluster = TestCluster::new(2)?;
    cluster.chain_add_genesis_block(0);

    cluster.start_engine(0);
    cluster.start_engine(1);

    // wait for engines to start
    let (_events_received0, events_receiver0) = cluster.extract_events_stream(0);
    events_receiver0
        .recv_timeout(Duration::from_secs(5))
        .unwrap();

    let (_events_received1, events_receiver1) = cluster.extract_events_stream(1);
    events_receiver1
        .recv_timeout(Duration::from_secs(5))
        .unwrap();

    // TODO: Make sure that block was added to second node
    // TODO: Disable transport first

    // add operation on each nodes
    let op1 = cluster.get_handle(0).write_entry(b"i love jello 0")?;
    let _op2 = cluster.get_handle(1).write_entry(b"i love jello 1")?;

    // expect operation to appear on node 1
    let handle = cluster.get_handle(1);
    let op = expect_result::<_, _, failure::Error>(|| {
        let ops = handle.get_pending_operations(op1..=op1)?;
        let first_op = ops.first();

        first_op
            .ok_or_else(|| err_msg("Operation not found"))
            .map(|op| op.frame.clone())
    });

    let op_reader: pending_operation::Reader = op.get_typed_reader()?;
    match op_reader.get_operation().which()? {
        pending_operation::operation::Entry(entry) => {
            let reader = entry?;
            println!("DATA: {:?}", String::from_utf8_lossy(reader.get_data()?));
        }
        _ => panic!(""),
    }

    Ok(())
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

///
///
///
struct TestCluster {
    _tempdir: tempdir::TempDir,
    runtime: Runtime,
    nodes: Nodes,
    transport_hub: MockTransportHub,
    clock: Clock,

    chain_stores: Vec<Option<DirectoryChainStore>>,
    pending_stores: Vec<Option<MemoryPendingStore>>,
    handles: Vec<Option<Handle<DirectoryChainStore, MemoryPendingStore>>>,
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

        for node_id in 0..count {
            let node = Node::new(format!("node{}", node_id));
            nodes.add(node.clone());

            let node_data_dir = tempdir.path().join(format!("{}", node_id));
            std::fs::create_dir(&node_data_dir)?;

            let chain_config = DirectoryChainStoreConfig::default();
            let chain = DirectoryChainStore::create(chain_config, &node_data_dir)?;
            chain_stores.push(Some(chain));

            let pending_store = MemoryPendingStore::new();
            pending_stores.push(Some(pending_store));

            handles.push(None);
        }

        Ok(TestCluster {
            _tempdir: tempdir,
            runtime,
            nodes,
            transport_hub,
            clock,
            chain_stores,
            pending_stores,
            handles,
        })
    }

    fn chain_add_genesis_block(&mut self, node_idx: usize) {
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

    fn extract_events_stream(
        &mut self,
        node_idx: usize,
    ) -> (Arc<Mutex<Vec<Event>>>, mpsc::Receiver<Event>) {
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

        (received_events, events_receiver)
    }

    fn get_node(&self, node_idx: usize) -> Node {
        self.nodes
            .get(&format!("node{}", node_idx))
            .unwrap()
            .clone()
    }

    fn get_handle(
        &mut self,
        node_idx: usize,
    ) -> &mut Handle<DirectoryChainStore, MemoryPendingStore> {
        self.handles[node_idx].as_mut().unwrap()
    }
}
