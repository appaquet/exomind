use crate::chain::ChainStore;
use crate::engine::testing::*;
use crate::operation::{NewOperation, OperationBuilder};
use crate::pending::PendingStore;

use super::*;
use std::time::{Duration, Instant};

#[test]
fn should_propose_block_on_new_operations() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    // nothing will be done since nothing is in pending store
    cluster.tick_commit_manager(0)?;
    assert_eq!(0, cluster.pending_stores[0].operations_count());

    // append new operation
    append_new_operation(&mut cluster, b"hello world")?;

    // this should create a block proposal (2nd op in pending store)
    cluster.tick_commit_manager(0)?;
    assert_eq!(2, cluster.pending_stores[0].operations_count()); // operation + block

    // shouldn't have signature yet
    let blocks = get_pending_blocks(&cluster)?;
    assert!(!blocks.blocks.iter().next().unwrap().1.has_my_signature);

    // this should sign + commit block to chain
    cluster.tick_commit_manager(0)?;
    assert_eq!(3, cluster.pending_stores[0].operations_count()); // operation + block + signature

    let blocks = get_pending_blocks(&cluster)?;
    assert_eq!(
        blocks.blocks.iter().next().unwrap().1.status,
        BlockStatus::PastCommitted
    );
    let last_block = cluster.chains[0].get_last_block()?.unwrap();
    assert_ne!(last_block.offset, 0);

    // this should not do anything, since it's already committed
    cluster.tick_commit_manager(0)?;
    assert_eq!(3, cluster.pending_stores[0].operations_count()); // operation + block + signature

    Ok(())
}

#[test]
fn should_not_propose_block_if_not_data() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.remove_node_role(0, CellNodeRole::Chain);

    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    cluster.tick_commit_manager(0)?;

    // append new operation
    append_new_operation(&mut cluster, b"hello world")?;

    // this should have create a block proposal if node could commit
    // but there should still be only 1 operation
    cluster.tick_commit_manager(0)?;
    assert_eq!(1, cluster.pending_stores[0].operations_count()); // operation

    Ok(())
}

#[test]
fn only_one_node_at_time_should_commit() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.chain_add_genesis_block(0);
    cluster.chain_add_genesis_block(1);
    cluster.tick_chain_synchronizer(0)?;
    cluster.tick_chain_synchronizer(1)?;

    // add operation & try to commit on each node
    append_new_operation(&mut cluster, b"hello world")?;
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;

    append_new_operation(&mut cluster, b"hello world")?;
    cluster.tick_commit_manager(1)?;
    cluster.tick_commit_manager(1)?;

    // only one node should have committed since it was its turn
    assert_ne!(
        cluster.pending_stores[0].operations_count(),
        cluster.pending_stores[1].operations_count()
    );

    Ok(())
}

#[test]
fn commit_block_at_interval() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    let commit_interval = cluster.commit_managers[0].config.commit_maximum_interval;

    cluster.clocks[0].set_fixed_instant(Instant::now());

    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    // first block should be committed right away since there is no previous
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
    append_new_operation(&mut cluster, b"hello world")?;
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;
    let block = cluster.chains[0].get_last_block()?.unwrap();
    let first_block_offset = block.offset();
    assert_ne!(0, first_block_offset);

    // second block should wait for time
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
    append_new_operation(&mut cluster, b"hello world")?;
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;
    let block = cluster.chains[0].get_last_block()?.unwrap();
    assert_eq!(first_block_offset, block.offset());

    // time has passed, should now commit
    cluster.clocks[0].add_fixed_instant_duration(commit_interval);
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;
    let block = cluster.chains[0].get_last_block()?.unwrap();
    assert_ne!(first_block_offset, block.offset());

    Ok(())
}

#[test]
fn commit_block_after_maximum_operations() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.clocks[0].set_fixed_instant(Instant::now());

    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    // first block should be committed right away since there is not previous
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
    append_new_operation(&mut cluster, b"hello world")?;
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;
    let block = cluster.chains[0].get_last_block()?.unwrap();
    let first_block_offset = block.offset();
    assert_ne!(0, first_block_offset);

    // should not commit new operations because didn't exceed interval & not enough
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
    append_new_operation(&mut cluster, b"hello world")?;
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;
    let block = cluster.chains[0].get_last_block()?.unwrap();
    assert_eq!(first_block_offset, block.offset());

    // now add maximum ops
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
    let max_ops = cluster.commit_managers[0]
        .config
        .commit_maximum_pending_store_count;
    for _i in 0..=max_ops {
        append_new_operation(&mut cluster, b"hello world")?;
    }

    // it should commits
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;
    let block = cluster.chains[0].get_last_block()?.unwrap();
    assert_ne!(first_block_offset, block.offset());

    Ok(())
}

#[test]
fn update_pending_status_for_committed_operations() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.clocks[0].set_fixed_instant(Instant::now());

    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    // first block should be committed right away since there is not previous
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
    let op_id = append_new_operation(&mut cluster, b"hello world")?;
    assert_eq!(
        cluster.pending_stores[0]
            .get_operation(op_id)?
            .unwrap()
            .commit_status,
        CommitStatus::Unknown
    );
    cluster.tick_commit_manager(0)?;
    cluster.tick_commit_manager(0)?;

    let block = cluster.chains[0].get_last_block()?.unwrap();
    assert_eq!(
        cluster.pending_stores[0]
            .get_operation(op_id)?
            .unwrap()
            .commit_status,
        CommitStatus::Committed(block.offset(), block.get_height()?)
    );

    Ok(())
}

#[test]
fn should_sign_valid_proposed_block() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    // append an operation
    let op_data = b"hello world";
    let op_id = append_new_operation(&mut cluster, op_data)?;

    // add a block proposal for this operation
    let block_id = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;

    // ticking should sign the block
    cluster.tick_commit_manager(0)?;

    let blocks = get_pending_blocks(&cluster)?;
    assert!(blocks.blocks[&block_id].has_my_signature);

    // should commit to chain
    cluster.tick_commit_manager(0)?;
    let last_block = cluster.chains[0].get_last_block()?.unwrap();
    assert_ne!(last_block.offset, 0);

    Ok(())
}

#[test]
fn should_order_next_best_blocks() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    // add 2 proposal
    let op_id = append_new_operation(&mut cluster, b"hello world")?;
    let block_id_signed = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;
    let _block_id_unsigned = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;

    // get blocks and fake signature on 1
    let mut blocks = get_pending_blocks(&cluster)?;
    blocks
        .blocks
        .get_mut(&block_id_signed)
        .unwrap()
        .has_my_signature = true;

    // the signed block should be first
    assert_eq!(
        blocks.potential_next_blocks().first().unwrap().group_id,
        block_id_signed
    );

    Ok(())
}

#[test]
fn should_refuse_invalid_proposed_block() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    // append an operation
    let op_data = b"hello world";
    let op_id = append_new_operation(&mut cluster, op_data)?;

    // should sign this block
    let block_id_good = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;
    cluster.tick_commit_manager(0)?;

    // should refuse this block as another one is already signed
    let block_id_bad = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;
    cluster.tick_commit_manager(0)?;

    let blocks = get_pending_blocks(&cluster)?;
    assert!(blocks.blocks[&block_id_good].has_my_signature);
    assert!(blocks.blocks[&block_id_bad].has_my_refusal);

    // should commit the good block, and ignore refused one
    cluster.tick_commit_manager(0)?;
    let last_block = cluster.chains[0].get_last_block()?.unwrap();
    let last_block_header_reader = last_block.header.get_reader()?;
    assert_eq!(
        last_block_header_reader.get_proposed_operation_id(),
        block_id_good
    );

    Ok(())
}

#[test]
fn proposal_should_expire_after_timeout() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);

    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    let config = cluster.commit_managers[0].config;

    // create block with 1 operation
    cluster.clocks[0].set_fixed_instant(Instant::now());
    let op_data = b"hello world";
    let op_id = append_new_operation(&mut cluster, op_data)?;
    let block_id = append_block_proposal_from_operations(&mut cluster, vec![op_id])?;

    // not expired
    let now = cluster.consistent_timestamp(0);
    let blocks = get_pending_blocks(&cluster)?;
    assert!(!blocks.blocks[&block_id].proposal.has_expired(&config, now));
    assert_eq!(blocks.blocks[&block_id].status, BlockStatus::NextPotential);

    // expired
    cluster.clocks[0].add_fixed_instant_duration(config.block_proposal_timeout);
    let now = cluster.consistent_timestamp(0);
    let blocks = get_pending_blocks(&cluster)?;
    assert!(blocks.blocks[&block_id].proposal.has_expired(&config, now));
    assert_eq!(blocks.blocks[&block_id].status, BlockStatus::NextExpired);

    // should propose a new block since previous has expired
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_millis(10));
    cluster.tick_commit_manager(0)?;
    let blocks = get_pending_blocks(&cluster)?;
    let potential_next = blocks.potential_next_blocks();
    assert_eq!(potential_next.len(), 1);

    Ok(())
}

#[test]
fn test_is_node_commit_turn() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    let node1 = cluster.get_node(0);
    let node2 = cluster.get_node(1);

    // we use node id to sort nodes
    let (first_node, sec_node, sec_node_idx) = if node1.id().to_string() < node2.id().to_string() {
        (&node1, &node2, 1)
    } else {
        (&node2, &node1, 0)
    };

    let config = CommitManagerConfig {
        commit_maximum_interval: Duration::from_secs(2),
        ..CommitManagerConfig::default()
    };

    {
        // test normal with all nodes having full chain
        let nodes = cluster.cells[0].nodes();
        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(0));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(!is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);

        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(1999));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(!is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);

        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(2000));
        assert!(!is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);

        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(3999));
        assert!(!is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
        assert!(is_node_commit_turn(&nodes, sec_node.id(), now, &config)?);
    }

    {
        // only node 0 has full chain
        cluster.remove_node_role(sec_node_idx, CellNodeRole::Chain);

        let nodes = cluster.cells[0].nodes();
        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(0));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);

        // second node can't commit
        assert!(is_node_commit_turn(&nodes, sec_node.id(), now, &config).is_err());

        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(1999));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);

        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(2000));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);

        let now = ConsistentTimestamp::from_unix_elapsed(Duration::from_millis(3999));
        assert!(is_node_commit_turn(&nodes, first_node.id(), now, &config)?);
    }

    Ok(())
}

#[test]
fn cleanup_past_committed_operations() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.clocks[0].set_fixed_instant(Instant::now());

    let assert_not_in_pending = |cluster: &EngineTestCluster, operation_id: u64| {
        assert!(&cluster.pending_stores[0]
            .get_operation(operation_id)
            .unwrap()
            .is_none());
    };

    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    let config = cluster.commit_managers[0].config;

    let mut operations_id = Vec::new();
    for _i in 0..=config.operations_cleanup_after_block_depth {
        // advance clock so that we make sure it commits
        cluster.clocks[0].add_fixed_instant_duration(config.commit_maximum_interval);

        let op_id = append_new_operation(&mut cluster, b"hello world")?;
        operations_id.push(op_id);

        // should create proposal, sign it and commit it
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;

        // make sure it's committed to chain
        assert!(cluster.chains[0]
            .get_block_by_operation_id(op_id)?
            .is_some());
    }

    // this will cleanup
    cluster.tick_commit_manager(0)?;

    // the first op should have been removed from pending store
    let first_op_id = *operations_id.first().unwrap();
    assert_not_in_pending(&cluster, first_op_id);

    // check if the block, signatures are still in pending
    let block: crate::block::BlockRef = cluster.chains[0]
        .get_block_by_operation_id(first_op_id)?
        .unwrap();
    let block_frame = block.header.get_reader()?;
    let block_group_id = block_frame.get_proposed_operation_id();
    assert_not_in_pending(&cluster, block_group_id);

    // check that SyncState was updated correctly
    let (cleanup_offset, cleanup_height) =
        cluster.sync_states[0].pending_last_cleanup_block.unwrap();
    assert_eq!(cleanup_height, block.get_height()?);
    assert_eq!(cleanup_offset, block.offset());

    // check if individual operations are still in pending
    for operation in block.operations_iter()? {
        let operation_reader = operation.get_reader()?;
        assert_not_in_pending(&cluster, operation_reader.get_operation_id());
    }

    Ok(())
}

#[test]
fn dont_cleanup_operations_from_commit_refused_blocks() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.chain_generate_dummy(0, 10, 1234);
    cluster.tick_chain_synchronizer(0)?;

    let preceding_valid_block = cluster.chains[0]
        .blocks_iter(0)?
        .map(|b| b.to_owned())
        .nth(2)
        .unwrap();

    // generate operations that won't be in a block yet
    let mut operations_id = Vec::new();
    let operations = (0..10).map(|i| {
        let op_id = append_new_operation(&mut cluster, b"hello world").unwrap();
        operations_id.push(op_id);
        cluster.pending_stores[0]
            .get_operation(operations_id[i])
            .unwrap()
            .unwrap()
            .frame
    });

    // we generate a block that is after block #2 in the chain, but is invalid since
    // there is already a block a this position
    let block_operations = BlockOperations::from_operations(operations)?;
    let block_id = cluster.consistent_timestamp(0).into();
    let invalid_block = BlockOwned::new_with_prev_block(
        &cluster.cells[0],
        &preceding_valid_block,
        block_id,
        block_operations,
    )?;
    let invalid_block_op_id = cluster.consistent_timestamp(0).into();
    let block_proposal = OperationBuilder::new_block_proposal(
        invalid_block_op_id,
        cluster.get_node(0).id(),
        &invalid_block,
    )?;

    let local_node = cluster.get_local_node(0);
    cluster.pending_stores[0].put_operation(block_proposal.sign_and_build(&local_node)?)?;

    // created blocks should all be invalid
    let pending_blocks = get_pending_blocks(&cluster)?;
    assert_eq!(
        BlockStatus::PastRefused,
        pending_blocks.blocks_status[&invalid_block_op_id]
    );

    // trigger cleanup
    let mut sync_context = cluster.get_sync_context(0);
    cluster.commit_managers[0].maybe_cleanup_pending_store(
        &mut sync_context,
        &pending_blocks,
        &mut cluster.pending_stores[0],
        &cluster.chains[0],
    )?;

    // all operations previously created should still be there since they aren't
    // committed and were in a past refused block
    for operation_id in &operations_id {
        assert!(&cluster.pending_stores[0]
            .get_operation(*operation_id)
            .unwrap()
            .is_some());
    }

    Ok(())
}

#[test]
fn cleanup_dangling_operations() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(1);
    cluster.clocks[0].set_fixed_instant(Instant::now());

    cluster.chain_add_genesis_block(0);
    cluster.tick_chain_synchronizer(0)?;

    let config = cluster.commit_managers[0].config;

    let mut operations_id = Vec::new();
    for _i in 0..=config.operations_cleanup_after_block_depth {
        // advance clock so that we make sure it commits
        cluster.clocks[0].add_fixed_instant_duration(config.commit_maximum_interval);

        let op_id = append_new_operation(&mut cluster, b"hello world")?;
        operations_id.push(op_id);

        // should create proposal, sign it and commit it
        cluster.tick_commit_manager(0)?;
        cluster.tick_commit_manager(0)?;

        // make sure it's committed to chain
        assert!(cluster.chains[0]
            .get_block_by_operation_id(op_id)?
            .is_some());
    }

    // clear pending store
    cluster.pending_stores[0].clear();

    // revive old operation
    let first_op_id = *operations_id.first().unwrap();
    let block: crate::block::BlockRef = cluster.chains[0]
        .get_block_by_operation_id(first_op_id)?
        .unwrap();
    let operation = block.get_operation(first_op_id)?.unwrap();
    cluster.pending_stores[0].put_operation(NewOperation::from_frame(operation.to_owned()))?;
    assert_eq!(1, cluster.pending_stores[0].operations_count());

    // this should trigger cleanup of dandling operation
    cluster.tick_commit_manager(0)?;

    assert_eq!(0, cluster.pending_stores[0].operations_count());

    Ok(())
}

fn append_new_operation(
    cluster: &mut EngineTestCluster,
    data: &[u8],
) -> Result<OperationId, EngineError> {
    let op_id = cluster.consistent_timestamp(0).into();

    for node in cluster.nodes.iter() {
        let idx = cluster.get_node_index(node.id());
        let op_builder = OperationBuilder::new_entry(op_id, node.id(), data);
        let operation = op_builder.sign_and_build(&node)?;
        cluster.pending_stores[idx].put_operation(operation)?;
    }

    Ok(op_id)
}

fn append_block_proposal_from_operations(
    cluster: &mut EngineTestCluster,
    op_ids: Vec<OperationId>,
) -> Result<OperationId, EngineError> {
    let node = &cluster.nodes[0];

    let previous_block = cluster.chains[0].get_last_block()?.unwrap();
    let block_operations = op_ids.iter().map(|op_id| {
        cluster.pending_stores[0]
            .get_operation(*op_id)
            .unwrap()
            .unwrap()
            .frame
    });
    let block_operations = BlockOperations::from_operations(block_operations)?;
    let block_operation_id = cluster.clocks[0].consistent_time(&node).into();
    let block = BlockOwned::new_with_prev_block(
        &cluster.cells[0],
        &previous_block,
        block_operation_id,
        block_operations,
    )?;
    let block_proposal_frame_builder =
        OperationBuilder::new_block_proposal(block_operation_id, node.id(), &block)?;
    let operation = block_proposal_frame_builder.sign_and_build(node)?;

    cluster.pending_stores[0].put_operation(operation)?;

    Ok(block_operation_id)
}

fn get_pending_blocks(cluster: &EngineTestCluster) -> Result<PendingBlocks, EngineError> {
    PendingBlocks::new(
        &cluster.commit_managers[0].config,
        &cluster.clocks[0],
        &cluster.cells[0],
        &cluster.pending_stores[0],
        &cluster.chains[0],
    )
}
