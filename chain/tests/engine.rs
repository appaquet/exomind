use itertools::Itertools;

use exocore_core::tests_utils::expect_result;

use exocore_chain::operation::Operation;
use exocore_chain::tests_utils::*;
use exocore_chain::*;
use exocore_core::cell::CellNodeRole;

#[macro_use]
extern crate anyhow;

// TODO: To be completed in https://github.com/appaquet/exocore/issues/42

#[test]
fn single_node_full_chain_write_read() -> anyhow::Result<()> {
    let mut cluster = TestChainCluster::new(1)?;
    cluster.create_node(0)?;
    cluster.create_chain_genesis_block(0);
    cluster.start_engine(0);
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

    // wait for all operations to be committed to a block
    cluster.wait_operations_committed(0, &[op1, op2]);
    let block_offsets = cluster.wait_next_block_commit(0);
    let first_block_offset = block_offsets.first().unwrap();

    // check if we really created a block
    let (chain_last_offset, chain_last_height) =
        cluster.get_handle(0).get_chain_last_block_info()?.unwrap();
    assert!(chain_last_offset >= *first_block_offset);
    assert!(chain_last_height >= 1);

    // get operation from chain
    let entry_operation = cluster
        .get_handle(0)
        .get_chain_operation(*first_block_offset, op1)?
        .unwrap();
    assert_eq!(b"i love rust 1", entry_operation.as_entry_data()?);
    assert!(entry_operation.status.is_committed());

    // get operation from anywhere, should now be committed
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
fn single_node_chain_iteration() -> anyhow::Result<()> {
    let mut cluster = TestChainCluster::new(1)?;
    cluster.create_node(0)?;
    cluster.create_chain_genesis_block(0);
    cluster.start_engine(0);
    cluster.wait_started(0);

    let chain_operations = cluster.get_handle(0).get_chain_operations(None);
    assert_eq!(0, chain_operations.count());

    let op1 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 1")?;
    let op2 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 2")?;
    cluster.wait_operations_committed(0, &[op1, op2]);

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
fn single_node_restart() -> anyhow::Result<()> {
    let mut cluster = TestChainCluster::new(1)?;
    cluster.create_node(0)?;
    cluster.create_chain_genesis_block(0);
    cluster.start_engine(0);
    cluster.wait_started(0);

    // wait for all operations to be emitted on stream
    let op1 = cluster
        .get_handle_mut(0)
        .write_entry_operation(b"i love rust 1")?;
    cluster.wait_operations_emitted(0, &[op1]);

    // wait for operations to be committed
    cluster.wait_operation_committed(0, op1);

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
fn two_nodes_full_replication() -> anyhow::Result<()> {
    let mut cluster = TestChainCluster::new(2)?;
    cluster.create_node(0)?;
    cluster.create_node(1)?;

    cluster.create_chain_genesis_block(0);

    cluster.start_engine(0);
    cluster.start_engine(1);
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
    cluster.wait_operations_committed(0, &[op1, op2]);
    cluster.wait_operations_committed(1, &[op1, op2]);

    // chain should be the same on both node with operations committed
    let segments_0 = cluster.get_handle(0).get_chain_segments()?;
    let segments_1 = cluster.get_handle(1).get_chain_segments()?;
    assert_eq!(segments_0, segments_1);

    Ok(())
}

#[test]
fn two_nodes_pending_store_cleanup() -> anyhow::Result<()> {
    let mut cluster = TestChainCluster::new(2)?;
    cluster.create_node(0)?;
    cluster.create_node(1)?;

    cluster.create_chain_genesis_block(0);

    // both nodes will cleanup after 2 height
    cluster.engines_config[0]
        .commit_manager_config
        .operations_cleanup_after_block_depth = 2;
    cluster.engines_config[1]
        .commit_manager_config
        .operations_cleanup_after_block_depth = 2;

    cluster.start_engine(0);
    cluster.start_engine(1);
    cluster.wait_started(0);
    cluster.wait_started(1);

    let mut operations_id = Vec::new();
    for _i in 0..=2 {
        let op_id = cluster
            .get_handle(0)
            .write_entry_operation(b"i love rust")?;
        operations_id.push(op_id);

        // wait for operation to be committed on node 1
        cluster.wait_operation_committed(1, op_id);
    }

    // first operation should not be in pending store anymore as it got cleaned up
    let first_op = operations_id.first().unwrap();
    expect_result::<_, _, anyhow::Error>(|| {
        let node1_op = cluster.get_handle(0).get_pending_operation(*first_op)?;
        if node1_op.is_some() {
            return Err(anyhow!("Was still on node 0"));
        }

        let node1_op = cluster.get_handle(1).get_pending_operation(*first_op)?;
        if node1_op.is_some() {
            return Err(anyhow!("Was still on node 1"));
        }

        Ok(())
    });

    Ok(())
}

#[test]
fn two_nodes_one_data_node() -> anyhow::Result<()> {
    let mut cluster = TestChainCluster::new(2)?;
    cluster.create_node(0)?;
    cluster.create_node(1)?;

    let node0_id = cluster.nodes[0].id().to_string();

    // 2nd node doesn't have chain role
    cluster.remove_node_role(1, CellNodeRole::Chain);

    cluster.create_chain_genesis_block(0);

    cluster.start_engine(0);
    cluster.start_engine(1);
    cluster.wait_started(0);
    cluster.wait_started(1);

    // Node 0 should still be able to advance even if second node is not part of
    // chain nodes
    for _i in 0..3 {
        let op = cluster
            .get_handle_mut(0)
            .write_entry_operation(b"i love rust")?;
        cluster.wait_operations_committed(0, &[op]);

        // make sure the block was proposed by node0
        let handle = cluster.get_handle_mut(0);
        let (offset, _height) = handle.get_chain_last_block_info()?.unwrap();
        let block = handle.get_chain_block(offset)?.unwrap();
        let block_header_reader = block.header.get_reader()?;
        assert_eq!(node0_id, block_header_reader.get_proposed_node_id()?);
    }

    Ok(())
}
