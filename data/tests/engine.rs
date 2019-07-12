use std::time::{Duration, Instant};

use failure::err_msg;
use itertools::Itertools;

use exocore_common::tests_utils::expect_result;

use exocore_data::operation::Operation;
use exocore_data::tests_utils::*;
use exocore_data::*;

// TODO: To be completed in https://github.com/appaquet/exocore/issues/42

#[test]
fn single_node_full_chain_write_read() -> Result<(), failure::Error> {
    let mut cluster = DataTestCluster::new(1)?;
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
    let block_offsets = cluster.wait_next_block_commit(0);
    let first_block_offset = block_offsets.first().unwrap();

    // check if we really created a block
    let (chain_last_offset, chain_last_depth) =
        cluster.get_handle(0).get_chain_last_block()?.unwrap();
    assert!(chain_last_offset >= *first_block_offset);
    assert!(chain_last_depth >= 1);

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
    let mut cluster = DataTestCluster::new(1)?;
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
    cluster.wait_next_block_commit(0);

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
    let mut cluster = DataTestCluster::new(1)?;
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
    cluster.wait_next_block_commit(0);

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
    let mut cluster = DataTestCluster::new(2)?;
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
    let mut cluster = DataTestCluster::new(2)?;
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
