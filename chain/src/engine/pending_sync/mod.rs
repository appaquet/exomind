use crate::block::BlockHeight;
use crate::engine::{request_tracker, EngineError, Event, SyncContext};
use crate::operation::{NewOperation, Operation, OperationId};
use crate::pending::{CommitStatus, PendingStore, StoredOperation};
use exocore_core::cell::{Cell, CellNodeRole, CellNodes};
use exocore_core::cell::{Node, NodeId};
use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_core::protos::generated::data_chain_capnp::chain_operation_header;
use exocore_core::protos::generated::data_transport_capnp::{
    pending_sync_range, pending_sync_request,
};
use exocore_core::sec::hash::{MultihashDigest, MultihashDigestExt, Sha3_256};
use exocore_core::time::Clock;
use itertools::{EitherOrBoth, Itertools};
use std::collections::{HashMap, HashSet};
use std::ops::{Bound, RangeBounds};

pub use config::PendingSyncConfig;
pub use error::PendingSyncError;
pub use sync_range::{SyncRangeBuilder, SyncRangesBuilder};

mod config;
mod error;
mod sync_range;
#[cfg(test)]
mod tests;

/// Synchronizes local pending store against remote nodes' pending stores. It
/// does that by exchanging PendingSyncRequest messages.
///
/// This PendingSyncRequest message contains information about ranges of
/// operations (by `OperationID`) in a local pending store, and is sent to be
/// applied / compared to the remote pending store. If there are differences in
/// the remote store, the remote nodes reply with a request that represents the
/// intent of the remote store. That intent could be to request missing data, or
/// send missing data.
///
/// The store could be compared as a whole (no boundaries), but that would
/// result in excessive data transmission, because a single difference would
/// require the whole store to be compared. In order to minimize this, when
/// building ranges, a node tries to limit the number of operations by range. If
/// a single range is not equal, only this range will be compared via
/// headers exchange and full operations exchange.
pub(super) struct PendingSynchronizer<PS: PendingStore> {
    config: PendingSyncConfig,
    cell: Cell,
    clock: Clock,
    nodes_info: HashMap<NodeId, NodeSyncInfo>,
    phantom: std::marker::PhantomData<PS>,
}

impl<PS: PendingStore> PendingSynchronizer<PS> {
    pub fn new(config: PendingSyncConfig, cell: Cell, clock: Clock) -> PendingSynchronizer<PS> {
        PendingSynchronizer {
            config,
            cell,
            clock,
            nodes_info: HashMap::new(),
            phantom: std::marker::PhantomData,
        }
    }

    /// Called at interval by the engine to make progress on synchronizing with
    /// other nodes. In theory, all changes are propagated in real-time when
    /// operations get added, but this periodic synchronization makes sure that
    /// we didn't lose anything.
    pub fn tick(&mut self, sync_context: &mut SyncContext, store: &PS) -> Result<(), EngineError> {
        debug!("Sync tick begins");

        let nodes = self.cell.nodes().to_owned();
        for cell_node in nodes
            .iter()
            .all_except_local()
            .filter(|cn| cn.has_role(CellNodeRole::Chain))
        {
            let node = cell_node.node();

            let sync_info = self.get_or_create_node_info_mut(node.id());
            if sync_info.request_tracker.can_send_request() {
                sync_info.request_tracker.set_last_send_now();
                let request =
                    self.create_sync_request_for_range(sync_context, store, .., |_op| {
                        OperationDetailsLevel::None
                    })?;
                sync_context.push_pending_sync_request(node.id().clone(), request);
            }
        }

        debug!("Sync tick ended");
        Ok(())
    }

    /// Handles a new operation coming from our own node, to be added to the
    /// pending store. This will add it to local pending store, and create a
    /// request message to be sent to other nodes.
    pub fn handle_new_operation(
        &mut self,
        sync_context: &mut SyncContext,
        store: &mut PS,
        operation: NewOperation,
    ) -> Result<(), EngineError> {
        let operation_id = operation.get_id()?;
        store.put_operation(operation)?;
        sync_context.push_event(Event::NewPendingOperation(operation_id));

        // create a sync request for which we send full detail for new op, but none for
        // other ops
        let nodes = self.cell.nodes();
        for cell_node in nodes
            .iter()
            .all_except_local()
            .filter(|cn| cn.has_role(CellNodeRole::Chain))
        {
            let request =
                self.create_sync_request_for_range(sync_context, store, operation_id.., |op| {
                    if op.operation_id == operation_id {
                        OperationDetailsLevel::Full
                    } else {
                        OperationDetailsLevel::None
                    }
                })?;
            sync_context.push_pending_sync_request(cell_node.node().id().clone(), request);
        }

        Ok(())
    }

    /// Handles a sync request coming from a remote node. A request contains
    /// ranges of operation ids that need to be merged and/or compared to
    /// our local store. See `handle_incoming_sync_ranges` for more details on
    /// the merge / comparison.

    /// If we have any differences with remote node data, we send a request back
    /// with more data that will allow converging in the same stored data.
    pub fn handle_incoming_sync_request<F: FrameReader>(
        &mut self,
        from_node: &Node,
        sync_context: &mut SyncContext,
        store: &mut PS,
        request: TypedCapnpFrame<F, pending_sync_request::Owned>,
    ) -> Result<(), EngineError> {
        debug!("Got sync request from {}", from_node.id());

        let in_reader: pending_sync_request::Reader = request.get_reader()?;
        let operations_from_height = self.get_from_block_height(sync_context, Some(in_reader));
        let in_ranges = in_reader.get_ranges()?;

        if let Some(out_ranges) = self.handle_incoming_sync_ranges(
            sync_context,
            store,
            in_ranges.iter(),
            operations_from_height,
        )? {
            let mut sync_request_frame_builder =
                CapnpFrameBuilder::<pending_sync_request::Owned>::new();
            let mut sync_request_builder = sync_request_frame_builder.get_builder();
            sync_request_builder.set_from_block_height(operations_from_height.unwrap_or(0));

            let mut ranges_builder = sync_request_builder
                .reborrow()
                .init_ranges(out_ranges.ranges.len() as u32);
            for (i, range) in out_ranges.ranges.into_iter().enumerate() {
                let mut builder = ranges_builder.reborrow().get(i as u32);
                range.write_into_sync_range_builder(&mut builder)?;
            }

            sync_context
                .push_pending_sync_request(from_node.id().clone(), sync_request_frame_builder);
        }

        Ok(())
    }

    /// Handles the ranges coming from a sync request. For each range, we check
    /// if we have the same information locally, and take actions based on
    /// it.

    /// For each range, actions include:
    ///   * Doing nothing if both remote and local are equals
    ///   * Sending full operations if remote is empty while we have operations
    ///     locally
    ///   * Sending headers operations if we differences without any headers to
    ///     compared with
    ///   * Diffing our headers vs remote headers if headers are included.

    /// In any case, if the range includes operations, we always apply them
    /// first.
    fn handle_incoming_sync_ranges<'a, I>(
        &mut self,
        sync_context: &mut SyncContext,
        store: &mut PS,
        sync_range_iterator: I,
        operations_from_height: Option<BlockHeight>,
    ) -> Result<Option<SyncRangesBuilder>, EngineError>
    where
        I: Iterator<Item = pending_sync_range::Reader<'a>>,
    {
        let mut out_ranges_contains_changes = false;
        let mut out_ranges = SyncRangesBuilder::new(self.config);

        for sync_range_reader in sync_range_iterator {
            let ((bounds_from, bounds_to), from_numeric, to_numeric) =
                extract_sync_bounds(&sync_range_reader)?;
            let bounds_range = (bounds_from, bounds_to);
            if to_numeric != 0 && to_numeric < from_numeric {
                return Err(PendingSyncError::InvalidSyncRequest(format!(
                    "Request from={} > to={}",
                    from_numeric, to_numeric
                ))
                .into());
            }

            // first, we store operations for which we have data directly in the payload
            let mut included_operations = HashSet::<OperationId>::new();
            if sync_range_reader.has_operations_frames() {
                for operation_frame_res in sync_range_reader.get_operations_frames()?.iter() {
                    let operation_frame_data = operation_frame_res?;
                    let operation_frame =
                        crate::operation::read_operation_frame(operation_frame_data)?.to_owned();

                    let operation_frame_reader = operation_frame.get_reader()?;
                    let operation_id = operation_frame_reader.get_operation_id();
                    included_operations.insert(operation_id);

                    let new_operation = NewOperation::from_frame(operation_frame);
                    let existed = store.put_operation(new_operation)?;
                    if !existed {
                        sync_context.push_event(Event::NewPendingOperation(operation_id));
                    }
                }
            }

            // then check local store's range hash and count. if our local store data is the
            // same as the one described in the payload, we stop here since
            // everything is synchronized
            let (local_hash, local_count) =
                self.local_store_range_info(store, bounds_range, operations_from_height)?;
            let remote_hash = sync_range_reader.get_operations_hash()?;
            let remote_count = sync_range_reader.get_operations_count();
            if remote_hash == &local_hash[..] && local_count == remote_count as usize {
                // we are equal to remote, nothing to do
                out_ranges.push_range(SyncRangeBuilder::new_hashed(
                    bounds_range,
                    local_hash,
                    local_count as u32,
                ));
                continue;
            }

            // if we're here, remote's data is different from local data. we check what we
            // need to do
            out_ranges_contains_changes = true;
            out_ranges.push_new_range(bounds_from);

            let operations_iter =
                self.operations_iter_from_height(store, bounds_range, operations_from_height)?;
            if remote_count == 0 {
                // remote has no data, we sent everything
                for operation in operations_iter {
                    out_ranges.push_operation(operation, OperationDetailsLevel::Full);
                }
            } else if !sync_range_reader.has_operations_headers()
                && !sync_range_reader.has_operations_frames()
            {
                // remote has only sent us hash, we reply with headers
                for operation in operations_iter {
                    out_ranges.push_operation(operation, OperationDetailsLevel::Header);
                }
            } else {
                // remote and local has differences. We do a diff
                let remote_iter = sync_range_reader.get_operations_headers()?.iter();
                Self::diff_local_remote_range(
                    &mut out_ranges,
                    &mut included_operations,
                    remote_iter,
                    operations_iter,
                )?;
            }

            out_ranges.set_last_range_to_bound(bounds_to);
        }

        if out_ranges_contains_changes {
            Ok(Some(out_ranges))
        } else {
            Ok(None)
        }
    }

    /// Creates a sync request with the given details for the given range of
    /// operation IDs
    fn create_sync_request_for_range<R, F>(
        &self,
        sync_context: &SyncContext,
        store: &PS,
        range: R,
        operation_details: F,
    ) -> Result<CapnpFrameBuilder<pending_sync_request::Owned>, EngineError>
    where
        R: RangeBounds<OperationId> + Clone,
        F: Fn(&StoredOperation) -> OperationDetailsLevel,
    {
        let mut sync_ranges = SyncRangesBuilder::new(self.config);

        // create first range with proper starting bound
        match range.start_bound() {
            Bound::Unbounded => sync_ranges.push_new_range(Bound::Unbounded),
            Bound::Excluded(op_id) => sync_ranges.push_new_range(Bound::Excluded(*op_id)),
            Bound::Included(op_id) => sync_ranges.push_new_range(Bound::Included(*op_id)),
        }

        let operations_from_height = self.get_from_block_height(sync_context, None);
        let operations_iter =
            self.operations_iter_from_height(store, range.clone(), operations_from_height)?;
        for operation in operations_iter {
            let details = operation_details(&operation);
            sync_ranges.push_operation(operation, details);
        }

        // make sure last range has an infinite upper bound
        if let Bound::Unbounded = range.end_bound() {
            sync_ranges.set_last_range_to_bound(Bound::Unbounded);
        }

        let mut sync_request_frame_builder =
            CapnpFrameBuilder::<pending_sync_request::Owned>::new();
        let mut sync_request_builder = sync_request_frame_builder.get_builder();
        sync_request_builder.set_from_block_height(operations_from_height.unwrap_or(0));
        let mut ranges_builder = sync_request_builder
            .reborrow()
            .init_ranges(sync_ranges.ranges.len() as u32);
        for (i, range) in sync_ranges.ranges.into_iter().enumerate() {
            let mut builder = ranges_builder.reborrow().get(i as u32);
            range.write_into_sync_range_builder(&mut builder)?;
        }

        Ok(sync_request_frame_builder)
    }

    /// Hashes the operations of the store for the given range. This will be
    /// used to compare with the incoming sync request.
    fn local_store_range_info<R>(
        &self,
        store: &PS,
        range: R,
        operations_from_height: Option<BlockHeight>,
    ) -> Result<(Vec<u8>, usize), EngineError>
    where
        R: RangeBounds<OperationId>,
    {
        let mut frame_hasher = Sha3_256::default();
        let mut count = 0;

        let operations_iter =
            self.operations_iter_from_height(store, range, operations_from_height)?;
        for operation in operations_iter {
            frame_hasher.input_signed_frame(operation.frame.inner().inner());
            count += 1;
        }

        Ok((frame_hasher.result().into_bytes(), count))
    }

    /// Do a diff of the local and remote data based on the headers in the sync
    /// request payload.
    fn diff_local_remote_range<'a, 'b, RI, LI>(
        out_ranges: &mut SyncRangesBuilder,
        included_operations: &mut HashSet<u64>,
        remote_iter: RI,
        local_iter: LI,
    ) -> Result<(), EngineError>
    where
        LI: Iterator<Item = StoredOperation> + 'b,
        RI: Iterator<Item = chain_operation_header::Reader<'a>> + 'a,
    {
        let merged_iter = remote_iter.merge_join_by(local_iter, |remote_op, local_op| {
            remote_op.get_operation_id().cmp(&local_op.operation_id)
        });

        let mut diff_has_difference = false;
        for merge_res in merged_iter {
            match merge_res {
                EitherOrBoth::Left(_remote_op) => {
                    diff_has_difference = true;
                    // We are missing this operation in local store.
                    // Not including header will make remote sends it to us.
                }
                EitherOrBoth::Right(local_op) => {
                    // Make sure it's not because operations was given with full details
                    if !included_operations.contains(&local_op.operation_id) {
                        diff_has_difference = true;

                        // Remote is missing it, send full operation
                        out_ranges.push_operation(local_op, OperationDetailsLevel::Full);
                    } else {
                        // Else, it was included in operations, so we tell remote that we have it
                        // now
                        out_ranges.push_operation(local_op, OperationDetailsLevel::Header);
                    }
                }
                EitherOrBoth::Both(_remote_op, local_op) => {
                    out_ranges.push_operation(local_op, OperationDetailsLevel::Header);
                }
            }
        }
        if !diff_has_difference {
            return Err(PendingSyncError::InvalidSyncState(
                "Got into diff branch, but didn't result in any changes, which shouldn't have happened".to_string(),
            )
            .into());
        }

        Ok(())
    }

    fn get_or_create_node_info_mut(&mut self, node_id: &NodeId) -> &mut NodeSyncInfo {
        // early exit here to prevent cloning the node_id for .entry()
        if self.nodes_info.contains_key(node_id) {
            return self.nodes_info.get_mut(node_id).unwrap();
        }

        let config = self.config;
        let clock = self.clock.clone();
        self.nodes_info
            .entry(node_id.clone())
            .or_insert_with(move || NodeSyncInfo::new(&config, clock))
    }

    /// Returns operations from the pending store, but only if they are not
    /// committed, or committed after the given height.
    fn operations_iter_from_height<'store, R>(
        &self,
        store: &'store PS,
        range: R,
        from_block_height: Option<BlockHeight>,
    ) -> Result<impl Iterator<Item = StoredOperation> + 'store, EngineError>
    where
        R: RangeBounds<OperationId>,
    {
        let iter = store.operations_iter(range)?.filter(move |op| {
            match (op.commit_status, from_block_height) {
                (_, None) => true,
                (CommitStatus::Unknown, _) => true,
                (CommitStatus::Committed(_offset, op_height), Some(from_height)) => {
                    op_height >= from_height
                }
            }
        });

        Ok(iter)
    }

    /// Returns the block height at which we filter operations from pending
    /// store with. See `PendingSyncConfig`.`operations_included_depth`. The
    /// height from the request has priority, and then we fallback to the one in
    /// the sync_state.
    fn get_from_block_height(
        &self,
        sync_context: &SyncContext,
        incoming_request_reader: Option<pending_sync_request::Reader>,
    ) -> Option<BlockHeight> {
        incoming_request_reader
            .and_then(|request_reader| {
                let block_height = request_reader.get_from_block_height();
                if block_height != 0 {
                    Some(block_height)
                } else {
                    None
                }
            })
            .or_else(|| {
                sync_context
                    .sync_state
                    .pending_last_cleanup_block
                    .map(|(_offset, height)| height + self.config.operations_depth_after_cleanup)
            })
    }
}

/// Synchronization information about a remote node
struct NodeSyncInfo {
    request_tracker: request_tracker::RequestTracker,
}

impl NodeSyncInfo {
    fn new(config: &PendingSyncConfig, clock: Clock) -> NodeSyncInfo {
        NodeSyncInfo {
            request_tracker: request_tracker::RequestTracker::new_with_clock(
                clock,
                config.request_tracker_config,
            ),
        }
    }
}

/// Converts bounds from sync_request range to SyncBounds
fn extract_sync_bounds(
    sync_range_reader: &pending_sync_range::Reader,
) -> Result<SyncBounds, EngineError> {
    let (from, from_included, to, to_included) = (
        sync_range_reader.get_from_operation(),
        sync_range_reader.get_from_included(),
        sync_range_reader.get_to_operation(),
        sync_range_reader.get_to_included(),
    );

    let from_bound = match (from, from_included) {
        (0, false) => Bound::Unbounded,
        (bound, true) => Bound::Included(bound),
        (bound, false) => Bound::Excluded(bound),
    };
    let to_bound = match (to, to_included) {
        (0, false) => Bound::Unbounded,
        (bound, true) => Bound::Included(bound),
        (bound, false) => Bound::Excluded(bound),
    };

    Ok(((from_bound, to_bound), from, to))
}

type SyncBounds = (
    (Bound<OperationId>, Bound<OperationId>),
    OperationId,
    OperationId,
);

#[derive(Copy, Clone)]
enum OperationDetailsLevel {
    Header,
    Full,
    None,
}
