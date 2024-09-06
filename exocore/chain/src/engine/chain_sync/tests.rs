use std::time::{Duration, Instant};

use exocore_core::framing::FrameBuilder;
use itertools::Itertools;

use super::*;
use crate::{
    chain::directory::DirectoryChainStore,
    engine::{testing::*, SyncState},
    operation::OperationBuilder,
};

#[test]
fn handle_sync_response_blocks() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(0, 10, 1234);
    cluster.chain_generate_dummy(1, 100, 1234);

    let node0 = cluster.get_node(0);
    let node1 = cluster.get_node(1);

    cluster.sync_chain_node_to_node(0, 1)?;
    cluster.tick_chain_synchronizer(0)?;
    assert_eq!(cluster.chains_synchronizer[0].status, Status::Downloading);
    assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

    // response from non-leader should result in an error
    let blocks_iter = cluster.chains[1].blocks_iter(0);
    let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
        &cluster.chains_synchronizer[1].config,
        10,
        0,
        blocks_iter,
    )?;
    let response_frame = response.as_owned_frame();
    let mut sync_context = SyncContext::new(SyncState::default());
    let result = cluster.chains_synchronizer[0].handle_sync_response(
        &mut sync_context,
        &node0,
        &mut cluster.chains[0],
        response_frame,
    );
    assert!(result.is_err());
    assert!(sync_context.messages.is_empty());

    // response from leader with blocks that aren't next should fail
    let blocks_iter = cluster.chains[1].blocks_iter(0);
    let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
        &cluster.chains_synchronizer[1].config,
        10,
        0,
        blocks_iter,
    )?;
    let response_frame = response.as_owned_frame();
    let mut sync_context = SyncContext::new(SyncState::default());
    let result = cluster.chains_synchronizer[0].handle_sync_response(
        &mut sync_context,
        &node1,
        &mut cluster.chains[0],
        response_frame,
    );
    assert!(result.is_err());

    // response from leader with blocks at right position should succeed and append
    let blocks_iter = cluster.chains[1].blocks_iter(0).skip(10); // skip 10 will go to 10th block
    let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
        &cluster.chains_synchronizer[0].config,
        10,
        0,
        blocks_iter,
    )?;
    let response_frame = response.as_owned_frame();
    let mut sync_context = SyncContext::new(SyncState::default());
    cluster.chains_synchronizer[0].handle_sync_response(
        &mut sync_context,
        &node1,
        &mut cluster.chains[0],
        response_frame,
    )?;

    Ok(())
}

#[test]
fn sync_empty_node1_to_full_node2() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(1, 100, 3434);

    let node1 = cluster.get_node(1);

    cluster.sync_chain_node_to_node(0, 1)?;
    {
        let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
        assert_eq!(NodeStatus::Synchronized, node1_node2_info.status(),);
        assert_eq!(
            None,
            node1_node2_info
                .last_common_block
                .as_ref()
                .map(|b| b.height),
        );
        assert_eq!(
            Some(99),
            node1_node2_info.last_known_block.as_ref().map(|b| b.height),
        );
    }

    // this will sync blocks & mark as synchronized
    cluster.sync_chain_node_to_node(0, 1)?;
    assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);
    assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

    // force status back to downloading to check if tick will turn back to
    // synchronized
    cluster.chains_synchronizer[0].status = Status::Downloading;
    cluster.sync_chain_node_to_node(0, 1)?;
    assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);

    cluster.assert_node_chain_equals(0, 1);

    Ok(())
}

#[test]
fn sync_full_node1_to_empty_node2() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(0, 100, 3434);

    let node1 = cluster.get_node(1);

    // running sync twice will yield to nothing as node2 is empty
    for _i in 0..2 {
        cluster.sync_chain_node_to_node(0, 1)?;
        let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
        assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
        assert_eq!(
            node1_node2_info
                .last_common_block
                .as_ref()
                .map(|b| b.height),
            None
        );
        assert_eq!(
            node1_node2_info.last_known_block.as_ref().map(|b| b.height),
            None
        );
    }

    // node1 is full, it has quorum (1 out of 2 nodes >= 50%)
    assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

    Ok(())
}

#[test]
fn sync_full_node1_to_half_node2() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(0, 100, 3434);
    cluster.chain_generate_dummy(1, 50, 3434);

    let node0 = cluster.get_node(0);
    let node1 = cluster.get_node(1);

    // running sync twice will yield to nothing as node1 is leader
    for _i in 0..2 {
        cluster.sync_chain_node_to_node(0, 1)?;
        let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
        assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
        assert_eq!(
            node1_node2_info
                .last_common_block
                .as_ref()
                .map(|b| b.height),
            Some(49)
        );
        assert_eq!(
            node1_node2_info.last_known_block.as_ref().map(|b| b.height),
            Some(49)
        );
    }

    // we're leader and synchronized because of it
    assert!(cluster.chains_synchronizer[0].is_leader(node0.id()));
    assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

    Ok(())
}

#[test]
fn sync_half_node1_to_full_node2() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(0, 50, 3434);
    cluster.chain_generate_dummy(1, 100, 3434);

    let node1 = cluster.get_node(1);

    cluster.sync_chain_node_to_node(0, 1)?;
    {
        let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
        assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
        assert_eq!(
            node1_node2_info
                .last_common_block
                .as_ref()
                .map(|b| b.height),
            Some(49)
        );
        assert_eq!(
            node1_node2_info.last_known_block.as_ref().map(|b| b.height),
            Some(99)
        );
    }

    // this will sync blocks & mark as synchronized
    cluster.sync_chain_node_to_node(0, 1)?;

    // node2 is leader
    assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));
    assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

    cluster.assert_node_chain_equals(0, 1);

    Ok(())
}

#[test]
fn sync_fully_divergent_node1_to_full_node2() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(0, 100, 1234);
    cluster.chain_generate_dummy(1, 100, 9876);

    let node1 = cluster.get_node(1);

    cluster.sync_chain_node_to_node(0, 1)?;
    {
        let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
        assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
        assert_eq!(
            node1_node2_info
                .last_common_block
                .as_ref()
                .map(|b| b.height),
            None,
        );
        assert_eq!(
            node1_node2_info.last_known_block.as_ref().map(|b| b.height),
            Some(99),
        );
    }

    match cluster.sync_chain_node_to_node(0, 1).err() {
        Some(EngineError::ChainSync(ChainSyncError::Diverged(_))) => {}
        other => panic!("Expected a diverged error, got {:?}", other),
    }

    // still unknown since we don't have a clear leader, as we've diverged from it
    assert_eq!(cluster.chains_synchronizer[0].status, Status::Unknown);

    Ok(())
}

#[test]
fn sync_single_block_even_if_max_out_size() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);

    let node_0 = cluster.get_local_node(0);
    cluster.chain_add_genesis_block(0);

    // generate a block that exceeds maximum send size
    let operation_count = cluster.chains_synchronizer[0].config.blocks_max_send_size / (5 * 1024);
    let operation_size =
        cluster.chains_synchronizer[0].config.blocks_max_send_size / operation_count;
    let operations = (0..operation_count)
        .map(|_i| {
            let op_id = cluster.consistent_timestamp(0).into();
            let data = vec![0u8; operation_size + 1];
            OperationBuilder::new_entry(op_id, node_0.id(), &data)
                .sign_and_build(&node_0)
                .unwrap()
                .frame
        })
        .collect_vec();
    cluster.chain_add_block_with_operations(0, operations.into_iter())?;

    let node0_last_block = cluster.chains[0].get_last_block()?.unwrap();
    let node0_last_block_size = node0_last_block.operations_data().len();
    assert!(node0_last_block_size > cluster.chains_synchronizer[0].config.blocks_max_send_size);

    // node 1 is empty
    cluster.chain_generate_dummy(1, 0, 1234);

    // make node 1 fetch data from node 0
    cluster.sync_chain_node_to_node(1, 0)?;
    cluster.sync_chain_node_to_node(1, 0)?;

    // node 1 should have the block even if it was bigger than maximum size, but it
    // should have sent blocks 1 by 1 instead
    let node1_last_block = cluster.chains[1].get_last_block()?.unwrap();
    assert_eq!(
        node0_last_block_size,
        node1_last_block.operations_data().len()
    );

    Ok(())
}

#[test]
fn cannot_sync_all_divergent() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(4);
    cluster.chain_generate_dummy(0, 100, 1234);
    cluster.chain_generate_dummy(1, 100, 9876);
    cluster.chain_generate_dummy(2, 100, 9876);
    cluster.chain_generate_dummy(3, 100, 9876);

    cluster.sync_chain_node_to_all(0)?;
    match cluster.sync_chain_node_to_all(0).err() {
        Some(EngineError::ChainSync(ChainSyncError::Diverged(_))) => {}
        other => panic!("Expected a diverged error, got {:?}", other),
    }

    // still unknown since we don't have a clear leader, as we've diverged from it
    assert_eq!(cluster.chains_synchronizer[0].status, Status::Unknown);

    Ok(())
}

#[test]
fn sync_half_divergent_node1_to_full_node2() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(0, 100, 1234);
    cluster.chain_generate_dummy(1, 50, 1234);
    cluster.chain_append_dummy(1, 50, 1234);

    let node1 = cluster.get_node(1);

    cluster.sync_chain_node_to_node(0, 1)?;
    {
        let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
        assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
        assert_eq!(
            node1_node2_info
                .last_common_block
                .as_ref()
                .map(|b| b.height),
            Some(49),
        );
        assert_eq!(
            node1_node2_info.last_known_block.as_ref().map(|b| b.height),
            Some(99),
        );
    }

    match cluster.sync_chain_node_to_node(0, 1).err() {
        Some(EngineError::ChainSync(ChainSyncError::Diverged(_))) => {}
        other => panic!("Expected a diverged error, got {:?}", other),
    }

    // still unknown since we don't have a clear leader, as we've diverged from it
    assert_eq!(cluster.chains_synchronizer[0].status, Status::Unknown);

    Ok(())
}

#[test]
fn sync_empty_node1_to_big_chain_node2() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);

    // this will force multiple back and forth for data
    cluster.chains_synchronizer[0].config.blocks_max_send_size = 1024;

    cluster.chain_generate_dummy(1, 1024, 3434);

    // first sync for metadata
    cluster.sync_chain_node_to_node(0, 1)?;

    // second sync for data
    cluster.sync_chain_node_to_node(0, 1)?;

    assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

    Ok(())
}

#[test]
fn leader_lost_metadata_out_of_date() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(4);
    cluster.chain_generate_dummy(0, 50, 3434);
    cluster.chain_generate_dummy(1, 100, 3434);
    cluster.chain_generate_dummy(2, 90, 3434);
    cluster.chain_generate_dummy(3, 90, 3434);

    let node1 = cluster.get_node(1);

    cluster.sync_chain_node_to_all(0)?;
    cluster.sync_chain_node_to_all(0)?;

    // node 1 is now leader
    assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

    {
        // we remove sync metadata from leader
        let node_info = cluster.chains_synchronizer[0].get_or_create_node_info_mut(node1.id());
        assert_eq!(node_info.status(), NodeStatus::Synchronized);
        node_info.last_common_is_known = false;
        node_info.last_known_block = None;
        assert_eq!(node_info.status(), NodeStatus::Unknown);
    }

    // node 1 is not leader anymore
    cluster.tick_chain_synchronizer(0)?;
    assert!(!cluster.chains_synchronizer[0].is_leader(node1.id()));

    Ok(())
}

#[test]
fn leader_lost_chain_too_far() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_generate_dummy(0, 50, 3434);
    cluster.chain_generate_dummy(1, 100, 3434);
    cluster.clocks[0].set_fixed_instant(Instant::now());

    let node1 = cluster.get_node(1);

    cluster.sync_chain_node_to_node(0, 1)?;
    cluster.sync_chain_node_to_node(0, 1)?;

    assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

    // make leader add 2 blocks, which shouldn't be considered as too far ahead
    cluster.chain_append_dummy(1, 2, 3434);
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(10));
    cluster.sync_chain_node_to_node(0, 1)?;
    assert_eq!(
        Status::Synchronized,
        cluster.chains_synchronizer[0].status(),
    );

    // make leader add 10 blocks, which should now be considered as too far ahead
    cluster.chain_append_dummy(1, 10, 3434);
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(10));
    cluster.sync_chain_node_to_node(0, 1)?;

    // now, a simple tick should reset status to downloading since we need to catch
    // up with master
    cluster.tick_chain_synchronizer(0)?;
    assert_eq!(Status::Downloading, cluster.chains_synchronizer[0].status(),);

    Ok(())
}

#[test]
fn quorum_lost_and_regain() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(3);
    cluster.chain_generate_dummy(0, 50, 3434);
    cluster.chain_generate_dummy(1, 100, 3434);
    cluster.chain_generate_dummy(2, 100, 3434);

    cluster.sync_chain_node_to_all(0)?;
    cluster.sync_chain_node_to_all(0)?;

    assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);

    // wipe metadata for node 1 and 2
    for node_idx in 1..=2 {
        let node = cluster.get_node(node_idx);
        let node_info = cluster.chains_synchronizer[0].get_or_create_node_info_mut(node.id());
        assert_eq!(NodeStatus::Synchronized, node_info.check_status());
        node_info.request_tracker.set_response_failure_count(100);
        assert_eq!(NodeStatus::Unknown, node_info.check_status());
    }

    // we lost quorum, we should now be synchronized anymore, no matter how many
    // ticks we do
    cluster.tick_chain_synchronizer(0)?;
    cluster.tick_chain_synchronizer(0)?;
    cluster.tick_chain_synchronizer(0)?;
    assert_eq!(Status::Unknown, cluster.chains_synchronizer[0].status);

    // reset request tracker to prevent waiting for last request timeout
    for node_idx in 1..=2 {
        let node = cluster.get_node(node_idx);
        let node_info = cluster.chains_synchronizer[0].get_or_create_node_info_mut(node.id());
        node_info.request_tracker.reset();
    }

    // now we do full sync between nodes, it will put back status
    cluster.sync_chain_node_to_all(0)?;
    cluster.sync_chain_node_to_all(0)?;
    assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);

    Ok(())
}
