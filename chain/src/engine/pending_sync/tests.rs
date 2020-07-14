use super::*;

use crate::engine::testing::*;
use crate::engine::{SyncContextMessage, SyncState};
use crate::operation::{NewOperation, OperationType};
use crate::pending::memory::MemoryPendingStore;
use crate::pending::CommitStatus;
use exocore_core::cell::LocalNode;
use exocore_core::framing::{CapnpFrameBuilder, FrameBuilder};
use exocore_core::protos::generated::data_chain_capnp::{chain_operation, chain_operation_header};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn tick_send_to_other_nodes() -> anyhow::Result<()> {
    // only one node, shouldn't send to ourself
    let mut cluster = EngineTestCluster::new(1);
    let mut sync_context = SyncContext::new(SyncState::default());
    cluster.pending_stores_synchronizer[0].tick(&mut sync_context, &cluster.pending_stores[0])?;
    assert_eq!(sync_context.messages.len(), 0);

    // two nodes should send to other node
    let mut cluster = EngineTestCluster::new(2);
    let mut sync_context = SyncContext::new(SyncState::default());
    cluster.pending_stores_synchronizer[0].tick(&mut sync_context, &cluster.pending_stores[0])?;
    assert_eq!(sync_context.messages.len(), 1);

    Ok(())
}

#[test]
fn create_sync_range_request() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.pending_generate_dummy(0, 0, 100);

    let mut sync_context = SyncContext::new(SyncState::default());
    cluster.pending_stores_synchronizer[0].tick(&mut sync_context, &cluster.pending_stores[0])?;
    let (_to_node, sync_request_frame) = extract_request_from_result(&sync_context);
    let sync_request_reader = sync_request_frame.get_reader()?;

    let ranges = sync_request_reader.get_ranges()?;
    assert_eq!(ranges.len(), 4);

    let range0 = ranges.get(0);
    assert_eq!(range0.get_from_operation(), 0);

    let range1 = ranges.get(1);
    assert_eq!(range0.get_to_operation(), range1.get_from_operation());

    let range3 = ranges.get(3);
    assert_eq!(range3.get_to_operation(), 0);

    Ok(())
}

#[test]
fn create_sync_range_request_with_height() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.clocks[0].set_fixed_instant(Instant::now());

    let config_height_offset = cluster.pending_stores_synchronizer[0]
        .config
        .operations_depth_after_cleanup;

    // we update operations status as if they were all committed at height 10
    let operations_id = cluster.pending_generate_dummy(0, 0, 100);
    for operation_id in operations_id {
        let status = CommitStatus::Committed(10, 10);
        cluster.pending_stores[0].update_operation_commit_status(operation_id, status)?;
    }

    // no filter should generate multiple ranges
    let mut sync_context = SyncContext::new(SyncState::default());
    cluster.pending_stores_synchronizer[0].tick(&mut sync_context, &cluster.pending_stores[0])?;
    let (_to_node, sync_request_frame) = extract_request_from_result(&sync_context);
    let sync_request_reader: pending_sync_request::Reader = sync_request_frame.get_reader()?;
    assert_eq!(0, sync_request_reader.get_from_block_height());
    let ranges = sync_request_reader.get_ranges()?;
    assert!(ranges.len() > 1);

    // filter with height of 1000 should generate only 1 empty range since it
    // matches no operations
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
    let mut sync_context = SyncContext::new(SyncState::default());
    sync_context.sync_state.pending_last_cleanup_block = Some((0, 1000));
    cluster.pending_stores_synchronizer[0].tick(&mut sync_context, &cluster.pending_stores[0])?;
    let (_to_node, sync_request_frame) = extract_request_from_result(&sync_context);
    let sync_request_reader: pending_sync_request::Reader = sync_request_frame.get_reader()?;
    assert_eq!(
        1000 + config_height_offset,
        sync_request_reader.get_from_block_height()
    );
    let ranges = sync_request_reader.get_ranges()?;
    assert_eq!(ranges.len(), 1);
    assert_eq!(0, ranges.get(0).get_operations_count());

    Ok(())
}

#[test]
fn new_operation_after_last_operation() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.pending_generate_dummy(0, 0, 50);
    cluster.pending_generate_dummy(1, 0, 50);

    // create operation after last operation id
    let generator_node = &cluster.nodes[0];
    let new_operation = create_dummy_new_entry_op(generator_node, 52, 52);
    let mut sync_context = SyncContext::new(SyncState::default());
    cluster.pending_stores_synchronizer[0].handle_new_operation(
        &mut sync_context,
        &mut cluster.pending_stores[0],
        new_operation,
    )?;
    let (_to_node, request) = extract_request_from_result(&sync_context);

    // should send the new operation directly, without requiring further requests
    let (count_a_to_b, count_b_to_a) =
        sync_nodes_with_initial_request(&mut cluster, 0, 1, request)?;
    assert_eq!(count_a_to_b, 1);
    assert_eq!(count_b_to_a, 0);

    // op should now be in each store
    let ops = cluster.pending_stores[0].get_group_operations(52)?.unwrap();
    assert_eq!(ops.operations.len(), 1);
    let ops = cluster.pending_stores[1].get_group_operations(52)?.unwrap();
    assert_eq!(ops.operations.len(), 1);

    Ok(())
}

#[test]
fn new_operation_among_current_operations() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);

    // generate operations with even operation id
    let generator_node = &cluster.nodes[0];
    let ops_generator = (0..=50).map(|i| {
        let (group_id, operation_id) = (((i * 2) % 10 + 1) as u64, i * 2 as u64);
        create_dummy_new_entry_op(generator_node, operation_id, group_id)
    });

    for operation in ops_generator {
        cluster.pending_stores[0].put_operation(operation.clone())?;
        cluster.pending_stores[1].put_operation(operation)?;
    }

    // create operation in middle of current ranges, with odd operation id
    let mut sync_context = SyncContext::new(SyncState::default());
    let new_operation = create_dummy_new_entry_op(generator_node, 51, 51);
    cluster.pending_stores_synchronizer[0].handle_new_operation(
        &mut sync_context,
        &mut cluster.pending_stores[0],
        new_operation,
    )?;
    let (_to_node, request) = extract_request_from_result(&sync_context);

    // should send the new operation directly, without requiring further requests
    let (count_a_to_b, count_b_to_a) =
        sync_nodes_with_initial_request(&mut cluster, 0, 1, request)?;
    assert_eq!(count_a_to_b, 1);
    assert_eq!(count_b_to_a, 0);

    // op should now be in each store
    let ops = cluster.pending_stores[0].get_group_operations(51)?.unwrap();
    assert_eq!(ops.operations.len(), 1);
    let ops = cluster.pending_stores[1].get_group_operations(51)?.unwrap();
    assert_eq!(ops.operations.len(), 1);

    Ok(())
}

#[test]
fn handle_sync_equals() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.pending_generate_dummy(0, 0, 100);
    cluster.pending_generate_dummy(1, 0, 100);

    let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(count_a_to_b, 1);
    assert_eq!(count_b_to_a, 0);

    Ok(())
}

#[test]
fn handle_sync_empty_to_many() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.pending_generate_dummy(0, 0, 100);

    let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(count_a_to_b, 2);
    assert_eq!(count_b_to_a, 1);

    Ok(())
}

#[test]
fn handle_sync_many_to_empty() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.pending_generate_dummy(1, 1, 100);

    let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(count_a_to_b, 1);
    assert_eq!(count_b_to_a, 1);

    Ok(())
}

#[test]
fn handle_sync_full_to_some() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.pending_generate_dummy(0, 0, 100);

    // insert 1/2 operations in second node
    let generator_node = &cluster.nodes[0];
    for operation in pending_ops_generator(generator_node, 100) {
        if operation.get_id()? % 2 == 0 {
            cluster.pending_stores[1].put_operation(operation)?;
        }
    }

    let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(count_a_to_b, 2);
    assert_eq!(count_b_to_a, 1);

    Ok(())
}

#[test]
fn handle_sync_some_to_all() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.pending_generate_dummy(1, 1, 100);

    // insert 1/2 operations in first node
    let generator_node = &cluster.nodes[1];
    for operation in pending_ops_generator(generator_node, 100) {
        if operation.get_id()? % 2 == 0 {
            cluster.pending_stores[0].put_operation(operation)?;
        }
    }

    let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(count_a_to_b, 2);
    assert_eq!(count_b_to_a, 2);

    Ok(())
}

#[test]
fn handle_sync_different_some_to_different_some() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);

    let generator_node = &cluster.nodes[0];
    for operation in pending_ops_generator(generator_node, 10) {
        if operation.get_id()? % 2 == 0 {
            cluster.pending_stores[0].put_operation(operation)?;
        } else if operation.get_id()? % 3 == 0 {
            cluster.pending_stores[1].put_operation(operation)?;
        }
    }

    let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(count_a_to_b, 2);
    assert_eq!(count_b_to_a, 2);

    Ok(())
}

#[test]
fn handle_sync_cleaned_up_depth() -> anyhow::Result<()> {
    let mut cluster = EngineTestCluster::new(2);
    cluster.clocks[0].set_fixed_instant(Instant::now());

    // we generate operations on node 0 spread in 10 blocks
    let operations_id = cluster.pending_generate_dummy(0, 0, 100);
    for operation_id in operations_id {
        let height = operation_id / 10;
        let status = CommitStatus::Committed(height, height);

        cluster.pending_stores[0].update_operation_commit_status(operation_id, status)?;
    }

    // syncing 0 to 1 without height filter should sync all operations
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
    sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(100, cluster.pending_stores[1].operations_count());

    // clear node 1 operations
    cluster.pending_stores[1].clear();

    // we mark node 0 as cleaned up up to block with height 3
    // syncing should not sync non-matching operations to node 1
    cluster.sync_states[0].pending_last_cleanup_block = Some((3, 3));
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
    sync_nodes(&mut cluster, 0, 1)?;
    assert_eq!(51, cluster.pending_stores[1].operations_count());

    // syncing 1 to 0 without height should not revive cleaned up operations
    cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
    sync_nodes(&mut cluster, 1, 0)?;
    assert_eq!(51, cluster.pending_stores[1].operations_count());

    Ok(())
}

#[test]
fn should_extract_from_block_offset() -> anyhow::Result<()> {
    let cluster = EngineTestCluster::new(1);

    let pending_store = &cluster.pending_stores_synchronizer[0];

    let mut req_frame_builder = CapnpFrameBuilder::<pending_sync_request::Owned>::new();
    {
        let mut req_builder: pending_sync_request::Builder = req_frame_builder.get_builder();
        req_builder.set_from_block_height(0);
    }

    // 0 in sync request means none
    let sync_context = SyncContext::new(SyncState::default());
    let frame = req_frame_builder.as_owned_frame();
    let frame_reader = frame.get_reader()?;
    let height = pending_store.get_from_block_height(&sync_context, Some(frame_reader));
    assert_eq!(None, height);

    // in sync state
    let mut sync_context = SyncContext::new(SyncState::default());
    sync_context.sync_state.pending_last_cleanup_block = Some((0, 10));
    let frame = req_frame_builder.as_owned_frame();
    let frame_reader = frame.get_reader()?;
    let height = pending_store.get_from_block_height(&sync_context, Some(frame_reader));
    assert_eq!(Some(12), height); // 10 + 2

    // request has priority
    let mut sync_context = SyncContext::new(SyncState::default());
    sync_context.sync_state.pending_last_cleanup_block = Some((0, 10));
    {
        let mut req_builder: pending_sync_request::Builder = req_frame_builder.get_builder();
        req_builder.set_from_block_height(20);
    }
    let frame = req_frame_builder.as_owned_frame();
    let frame_reader = frame.get_reader()?;
    let height = pending_store.get_from_block_height(&sync_context, Some(frame_reader));
    assert_eq!(Some(20), height);

    Ok(())
}

#[test]
fn operations_iter_filtered_height() -> anyhow::Result<()> {
    let cluster = EngineTestCluster::new(1);

    let local_node = &cluster.nodes[0];
    let pending_store = &cluster.pending_stores_synchronizer[0];

    let mut store = MemoryPendingStore::new();
    store.put_operation(create_dummy_new_entry_op(&local_node, 100, 100))?;
    store.put_operation(create_dummy_new_entry_op(&local_node, 101, 101))?;
    store.put_operation(create_dummy_new_entry_op(&local_node, 102, 102))?;

    let res = pending_store
        .operations_iter_from_height(&store, .., None)?
        .collect_vec();
    assert_eq!(3, res.len());

    // should return everything since they are all `Unknown` status
    let res = pending_store
        .operations_iter_from_height(&store, .., Some(2))?
        .collect_vec();
    assert_eq!(3, res.len());

    // should return not committed
    store.update_operation_commit_status(100, CommitStatus::Unknown)?;
    let res = pending_store
        .operations_iter_from_height(&store, .., Some(2))?
        .collect_vec();
    assert_eq!(3, res.len());

    // should return equal height
    store.update_operation_commit_status(101, CommitStatus::Committed(0, 2))?;
    let res = pending_store
        .operations_iter_from_height(&store, .., Some(2))?
        .collect_vec();
    assert_eq!(3, res.len());

    // should not return smaller height
    let res = pending_store
        .operations_iter_from_height(&store, .., Some(3))?
        .collect_vec();
    assert_eq!(2, res.len());

    Ok(())
}

#[test]
fn sync_ranges_push_operation() {
    let local_node = LocalNode::generate();
    let mut sync_ranges = SyncRangesBuilder::new(PendingSyncConfig::default());
    for operation in stored_ops_generator(&local_node, 90) {
        sync_ranges.push_operation(operation, OperationDetailsLevel::None);
    }

    assert_eq!(sync_ranges.ranges.len(), 3);
    assert_eq!(
        sync_ranges.ranges.first().map(|r| r.from_operation),
        Some(Bound::Unbounded)
    );

    // check continuity of ranges
    let mut last_range_to: Option<Bound<OperationId>> = None;
    for range in sync_ranges.ranges.iter() {
        match (last_range_to, range.from_operation) {
            (None, _) => assert_eq!(range.from_operation, Bound::Unbounded),
            (Some(Bound::Included(last_to)), Bound::Excluded(current_from)) => {
                assert_eq!(last_to, current_from)
            }
            other => panic!("Unexpected last bound: {:?}", other),
        }

        last_range_to = Some(range.to_operation);
    }

    assert_eq!(last_range_to, Some(Bound::Included(90)));
}

#[test]
fn sync_range_to_frame_builder_with_hash() -> anyhow::Result<()> {
    let local_node = LocalNode::generate();
    let frames_builder = build_sync_ranges_frames(&local_node, 90, OperationDetailsLevel::None);
    assert_eq!(frames_builder.len(), 3);

    let frame0 = frames_builder[0].as_owned_frame();
    let frame0_reader: pending_sync_range::Reader = frame0.get_reader()?;
    let frame0_hash = frame0_reader.reborrow().get_operations_hash().unwrap();
    assert_eq!(frame0_reader.has_operations_frames(), false);
    assert_eq!(frame0_reader.has_operations_headers(), false);

    let frame1 = frames_builder[1].as_owned_frame();
    let frame1_reader: pending_sync_range::Reader = frame1.get_reader()?;
    let frame1_hash = frame1_reader.reborrow().get_operations_hash()?;
    assert_eq!(frame1_reader.has_operations_frames(), false);
    assert_eq!(frame1_reader.has_operations_headers(), false);

    assert_ne!(frame0_hash, frame1_hash);

    Ok(())
}

#[test]
fn sync_range_to_frame_builder_with_headers() -> anyhow::Result<()> {
    let local_node = LocalNode::generate();
    let frames_builder = build_sync_ranges_frames(&local_node, 90, OperationDetailsLevel::Header);

    let frame0 = frames_builder[0].as_owned_frame();
    let frame0_reader: pending_sync_range::Reader = frame0.get_reader()?;
    assert_eq!(frame0_reader.has_operations_frames(), false);
    assert_eq!(frame0_reader.has_operations_headers(), true);

    let operations = frame0_reader.get_operations_headers()?;
    let operation0_header: chain_operation_header::Reader = operations.get(0);
    assert_eq!(operation0_header.get_group_id(), 2);

    Ok(())
}

#[test]
fn sync_range_to_frame_builder_with_data() -> anyhow::Result<()> {
    let local_node = LocalNode::generate();
    let frames_builder = build_sync_ranges_frames(&local_node, 90, OperationDetailsLevel::Full);

    let frame0 = frames_builder[0].as_owned_frame();
    let frame0_reader: pending_sync_range::Reader = frame0.get_reader()?;
    assert_eq!(frame0_reader.has_operations_frames(), true);
    assert_eq!(frame0_reader.has_operations_headers(), false);

    let operations = frame0_reader.get_operations_frames()?;
    let operation0_data = operations.get(0)?;
    let operation0_frame = crate::operation::read_operation_frame(operation0_data)?;

    let operation0_reader: chain_operation::Reader = operation0_frame.get_reader()?;
    let operation0_inner_reader = operation0_reader.get_operation();
    assert!(operation0_inner_reader.has_entry());

    Ok(())
}

fn sync_nodes(
    cluster: &mut EngineTestCluster,
    node_id_a: usize,
    node_id_b: usize,
) -> Result<(usize, usize), anyhow::Error> {
    // tick the first node, which will generate a sync request
    let sync_context = cluster.tick_pending_synchronizer(node_id_a)?;
    let (_to_node, initial_request) = extract_request_from_result(&sync_context);

    sync_nodes_with_initial_request(cluster, node_id_a, node_id_b, initial_request)
}

fn sync_nodes_with_initial_request(
    cluster: &mut EngineTestCluster,
    node_id_a: usize,
    node_id_b: usize,
    initial_request: TypedCapnpFrame<Vec<u8>, pending_sync_request::Owned>,
) -> Result<(usize, usize), anyhow::Error> {
    let node_a = cluster.get_node(node_id_a);
    let node_b = cluster.get_node(node_id_b);

    let mut count_a_to_b = 0;
    let mut count_b_to_a = 0;

    let mut next_request = initial_request;
    debug!("Request from a={} to b={}", node_id_a, node_id_b);
    print_sync_request(&next_request);

    loop {
        if count_a_to_b > 100 {
            panic!(
                "Seem to be stucked in an infinite sync loop (a_to_b={} b_to_a={})",
                count_a_to_b, count_b_to_a
            );
        }

        //
        // B to A
        //
        count_a_to_b += 1;
        let mut sync_context = SyncContext::new(cluster.sync_states[node_id_b]);
        cluster.pending_stores_synchronizer[node_id_b].handle_incoming_sync_request(
            &node_a,
            &mut sync_context,
            &mut cluster.pending_stores[node_id_b],
            next_request,
        )?;
        if sync_context.messages.is_empty() {
            debug!("No request from b={} to a={}", node_id_b, node_id_a);
            break;
        }
        cluster.sync_states[node_id_b] = sync_context.sync_state;

        count_b_to_a += 1;
        let (to_node, request) = extract_request_from_result(&sync_context);
        assert_eq!(&to_node, node_a.id());
        debug!("Request from b={} to a={}", node_id_b, node_id_a);
        print_sync_request(&request);

        //
        // A to B
        //
        let mut sync_context = SyncContext::new(cluster.sync_states[node_id_a]);
        cluster.pending_stores_synchronizer[node_id_a].handle_incoming_sync_request(
            &node_b,
            &mut sync_context,
            &mut cluster.pending_stores[node_id_a],
            request,
        )?;
        if sync_context.messages.is_empty() {
            debug!("No request from a={} to b={}", node_id_a, node_id_b);
            break;
        }
        cluster.sync_states[node_id_a] = sync_context.sync_state;

        let (to_node, request) = extract_request_from_result(&sync_context);
        assert_eq!(&to_node, node_b.id());
        debug!("Request from a={} to b={}", node_id_a, node_id_b);
        next_request = request;
        print_sync_request(&next_request);
    }

    Ok((count_a_to_b, count_b_to_a))
}

fn build_sync_ranges_frames(
    local_node: &LocalNode,
    count: usize,
    details: OperationDetailsLevel,
) -> Vec<CapnpFrameBuilder<pending_sync_range::Owned>> {
    let mut sync_ranges = SyncRangesBuilder::new(PendingSyncConfig::default());
    for operation in stored_ops_generator(local_node, count) {
        sync_ranges.push_operation(operation, details);
    }
    sync_ranges
        .ranges
        .into_iter()
        .map(|range| {
            let mut range_frame_builder = CapnpFrameBuilder::<pending_sync_range::Owned>::new();
            let mut range_msg_builder = range_frame_builder.get_builder();
            range
                .write_into_sync_range_builder(&mut range_msg_builder)
                .unwrap();
            range_frame_builder
        })
        .collect()
}

fn extract_request_from_result(
    sync_context: &SyncContext,
) -> (
    NodeId,
    TypedCapnpFrame<Vec<u8>, pending_sync_request::Owned>,
) {
    match sync_context.messages.last().unwrap() {
        SyncContextMessage::PendingSyncRequest(node_id, req) => {
            (node_id.clone(), req.as_owned_frame())
        }
        _other => panic!("Expected a pending sync request, got another type of message"),
    }
}

fn pending_ops_generator(
    local_node: &LocalNode,
    count: usize,
) -> impl Iterator<Item = NewOperation> {
    let local_node = local_node.clone();
    (1..=count).map(move |i| {
        let (group_id, operation_id) = ((i % 10 + 1) as u64, i as u64);
        create_dummy_new_entry_op(&local_node, operation_id, group_id)
    })
}

fn stored_ops_generator(
    local_node: &LocalNode,
    count: usize,
) -> impl Iterator<Item = StoredOperation> {
    let local_node = local_node.clone();
    (1..=count).map(move |i| {
        let (group_id, operation_id) = ((i % 10 + 1) as u64, i as u64);
        let new_operation = create_dummy_new_entry_op(&local_node, operation_id, group_id);
        let frame = Arc::new(new_operation.frame);

        StoredOperation {
            group_id,
            operation_type: OperationType::Entry,
            operation_id,
            commit_status: CommitStatus::Unknown,
            frame,
        }
    })
}

fn print_sync_request<F: FrameReader>(request: &TypedCapnpFrame<F, pending_sync_request::Owned>) {
    let reader: pending_sync_request::Reader = request.get_reader().unwrap();
    let ranges = reader.get_ranges().unwrap();

    for range in ranges.iter() {
        let ((bound_from, bound_to), _from, _to) = extract_sync_bounds(&range).unwrap();
        debug!("  Range {:?} to {:?}", bound_from, bound_to,);
        debug!("    Hash={:?}", range.get_operations_hash().unwrap());
        debug!("    Count={}", range.get_operations_count());

        if range.has_operations_headers() {
            debug!(
                "    Headers={}",
                range.get_operations_headers().unwrap().len()
            );
        } else {
            debug!("    Headers=None");
        }

        if range.has_operations_frames() {
            debug!(
                "    Frames={}",
                range.get_operations_frames().unwrap().len()
            );
        } else {
            debug!("    Frames=None");
        }
    }
}
