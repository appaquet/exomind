use std::collections::{HashMap, HashSet};
use std::ops::{Bound, RangeBounds};

use itertools::{EitherOrBoth, Itertools};

use crate::operation::OperationId;
use exocore_common::crypto::hash::{Digest, MultihashDigest, Sha3_256};
use exocore_common::node::{Node, NodeId};
use exocore_common::serialization::protos::data_chain_capnp::pending_operation_header;
use exocore_common::serialization::protos::data_transport_capnp::{
    pending_sync_range, pending_sync_request,
};

use crate::block::BlockDepth;
use crate::engine::{request_tracker, Event};
use crate::engine::{Error, SyncContext};
use crate::operation::{NewOperation, Operation};
use crate::pending::{CommitStatus, PendingStore, StoredOperation};
use exocore_common::cell::{Cell, CellNodes};
use exocore_common::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_common::time::Clock;

///
/// Synchronizes local pending store against remote nodes' pending stores. It does that by exchanging
/// PendingSyncRequest messages.
///
/// This PendingSyncRequest message contains information about ranges of operations (by `OperationID`) in a local pending store,
/// and is sent to be applied / compared to the remote pending store. If there are differences in the remote store, the remote
/// nodes reply with a request that represents the intent of the remote store. That intent could be to request missing data,
/// or send missing data.
///
/// The store could be compared as a whole (no boundaries), but that would result in excessive data transmission, because
/// a single difference would require the whole store to be compared. In order to minimize this, when building ranges, a node
/// tries to limit the number of operations by range. If a single range is not equal, only this range will be compared via
/// headers exchange and full operations exchange.
///
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

    ///
    /// Called at interval by the engine to make progress on synchronizing with other nodes. In theory, all changes are propagated
    /// in real-time when operations get added, but this periodic synchronization makes sure that we didn't lose anything.
    ///
    pub fn tick(&mut self, sync_context: &mut SyncContext, store: &PS) -> Result<(), Error> {
        debug!("Sync tick begins");

        let nodes = self.cell.nodes().to_owned();
        for node in nodes.iter().all_except_local() {
            let sync_info = self.get_or_create_node_info_mut(node.id());
            if sync_info.request_tracker.can_send_request() {
                sync_info.request_tracker.set_last_send_now();
                let request =
                    self.create_sync_request_for_range(sync_context, store, .., |_op| {
                        OperationDetails::None
                    })?;
                sync_context.push_pending_sync_request(node.id().clone(), request);
            }
        }

        debug!("Sync tick ended");
        Ok(())
    }

    ///
    /// Handles a new operation coming from our own node, to be added to the pending store.
    /// This will add it to local pending store, and create a request message to be sent to other nodes.
    ///
    pub fn handle_new_operation(
        &mut self,
        sync_context: &mut SyncContext,
        store: &mut PS,
        operation: NewOperation,
    ) -> Result<(), Error> {
        let operation_id = operation.get_id()?;
        store.put_operation(operation)?;
        sync_context.push_event(Event::PendingOperationNew(operation_id));

        // create a sync request for which we send full detail for new op, but none for other ops
        let nodes = self.cell.nodes();
        for node in nodes.iter().all_except_local() {
            let request =
                self.create_sync_request_for_range(sync_context, store, operation_id.., |op| {
                    if op.operation_id == operation_id {
                        OperationDetails::Full
                    } else {
                        OperationDetails::None
                    }
                })?;
            sync_context.push_pending_sync_request(node.id().clone(), request);
        }

        Ok(())
    }

    ///
    /// Handles a sync request coming from a remote node. A request contains ranges of operation ids that need to be merged and/or
    /// compared to our local store. See `handle_incoming_sync_ranges` for more details on the merge / comparison.
    ///
    /// If we have any differences with remote node data, we send a request back with more data that will allow converging in the
    /// same stored data.
    ///
    pub fn handle_incoming_sync_request<F: FrameReader>(
        &mut self,
        from_node: &Node,
        sync_context: &mut SyncContext,
        store: &mut PS,
        request: TypedCapnpFrame<F, pending_sync_request::Owned>,
    ) -> Result<(), Error> {
        debug!("Got sync request from {}", from_node.id());

        let in_reader: pending_sync_request::Reader = request.get_reader()?;
        let operations_from_depth = self.get_from_block_depth(sync_context, Some(in_reader));
        let in_ranges = in_reader.get_ranges()?;

        if let Some(out_ranges) = self.handle_incoming_sync_ranges(
            sync_context,
            store,
            in_ranges.iter(),
            operations_from_depth,
        )? {
            let mut sync_request_frame_builder =
                CapnpFrameBuilder::<pending_sync_request::Owned>::new();
            let mut sync_request_builder = sync_request_frame_builder.get_builder();
            sync_request_builder.set_from_block_depth(operations_from_depth.unwrap_or(0));

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

    ///
    /// Handles the ranges coming from a sync request. For each range, we check if we have
    /// the same information locally, and take actions based on it.
    ///
    /// For each range, actions include:
    ///   * Doing nothing if both remote and local are equals
    ///   * Sending full operations if remote is empty while we have operations locally
    ///   * Sending headers operations if we differences without any headers to compared with
    ///   * Diffing our headers vs remote headers if headers are included.
    ///
    /// In any case, if the range includes operations, we always apply them first.
    ///
    fn handle_incoming_sync_ranges<'a, I>(
        &mut self,
        sync_context: &mut SyncContext,
        store: &mut PS,
        sync_range_iterator: I,
        operations_from_depth: Option<BlockDepth>,
    ) -> Result<Option<SyncRangesBuilder>, Error>
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
            if sync_range_reader.has_operations() {
                for operation_frame_res in sync_range_reader.get_operations()?.iter() {
                    let operation_frame_data = operation_frame_res?;
                    let operation_frame =
                        crate::operation::read_operation_frame(operation_frame_data)?.to_owned();

                    let operation_frame_reader = operation_frame.get_reader()?;
                    let operation_id = operation_frame_reader.get_operation_id();
                    included_operations.insert(operation_id);

                    let new_operation = NewOperation::from_frame(operation_frame);
                    let existed = store.put_operation(new_operation)?;
                    if !existed {
                        sync_context.push_event(Event::PendingOperationNew(operation_id));
                    }
                }
            }

            // then check local store's range hash and count. if our local store data is the same as the one described in the
            // payload, we stop here since everything is synchronized
            let (local_hash, local_count) =
                self.local_store_range_info(store, bounds_range, operations_from_depth)?;
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

            // if we're here, remote's data is different from local data. we check what we need to do
            out_ranges_contains_changes = true;
            out_ranges.push_new_range(bounds_from);

            let operations_iter =
                self.operations_iter_from_depth(store, bounds_range, operations_from_depth)?;
            if remote_count == 0 {
                // remote has no data, we sent everything
                for operation in operations_iter {
                    out_ranges.push_operation(operation, OperationDetails::Full);
                }
            } else if !sync_range_reader.has_operations_headers()
                && !sync_range_reader.has_operations()
            {
                // remote has only sent us hash, we reply with headers
                for operation in operations_iter {
                    out_ranges.push_operation(operation, OperationDetails::Header);
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

    ///
    /// Creates a sync request with the given details for the given range of operation IDs
    ///
    fn create_sync_request_for_range<R, F>(
        &self,
        sync_context: &SyncContext,
        store: &PS,
        range: R,
        operation_details: F,
    ) -> Result<CapnpFrameBuilder<pending_sync_request::Owned>, Error>
    where
        R: RangeBounds<OperationId> + Clone,
        F: Fn(&StoredOperation) -> OperationDetails,
    {
        let mut sync_ranges = SyncRangesBuilder::new(self.config);

        // create first range with proper starting bound
        match range.start_bound() {
            Bound::Unbounded => sync_ranges.push_new_range(Bound::Unbounded),
            Bound::Excluded(op_id) => sync_ranges.push_new_range(Bound::Excluded(*op_id)),
            Bound::Included(op_id) => sync_ranges.push_new_range(Bound::Included(*op_id)),
        }

        let operations_from_depth = self.get_from_block_depth(sync_context, None);
        let operations_iter =
            self.operations_iter_from_depth(store, range.clone(), operations_from_depth)?;
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
        sync_request_builder.set_from_block_depth(operations_from_depth.unwrap_or(0));
        let mut ranges_builder = sync_request_builder
            .reborrow()
            .init_ranges(sync_ranges.ranges.len() as u32);
        for (i, range) in sync_ranges.ranges.into_iter().enumerate() {
            let mut builder = ranges_builder.reborrow().get(i as u32);
            range.write_into_sync_range_builder(&mut builder)?;
        }

        Ok(sync_request_frame_builder)
    }

    ///
    /// Hashes the operations of the store for the given range. This will be used to compare with the
    /// incoming sync request.
    ///
    fn local_store_range_info<R>(
        &self,
        store: &PS,
        range: R,
        operations_from_depth: Option<BlockDepth>,
    ) -> Result<(Vec<u8>, usize), Error>
    where
        R: RangeBounds<OperationId>,
    {
        let mut frame_hasher = Sha3_256::new();
        let mut count = 0;

        let operations_iter =
            self.operations_iter_from_depth(store, range, operations_from_depth)?;
        for operation in operations_iter {
            frame_hasher.input_signed_frame(operation.frame.inner().inner());
            count += 1;
        }

        Ok((frame_hasher.into_multihash_bytes(), count))
    }

    ///
    /// Do a diff of the local and remote data based on the headers in the sync request payload.
    ///
    fn diff_local_remote_range<'a, 'b, RI, LI>(
        out_ranges: &mut SyncRangesBuilder,
        included_operations: &mut HashSet<u64>,
        remote_iter: RI,
        local_iter: LI,
    ) -> Result<(), Error>
    where
        LI: Iterator<Item = StoredOperation> + 'b,
        RI: Iterator<Item = pending_operation_header::Reader<'a>> + 'a,
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
                        out_ranges.push_operation(local_op, OperationDetails::Full);
                    } else {
                        // Else, it was included in operations, so we tell remote that we have it now
                        out_ranges.push_operation(local_op, OperationDetails::Header);
                    }
                }
                EitherOrBoth::Both(_remote_op, local_op) => {
                    out_ranges.push_operation(local_op, OperationDetails::Header);
                }
            }
        }
        if !diff_has_difference {
            return Err(PendingSyncError::InvalidSyncState("Got into diff branch, but didn't result in any changes, which shouldn't have happened".to_string()).into());
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

    ///
    /// Returns operations from the pending store, but only if they are not committed, or
    /// committed after the given depth.
    ///
    fn operations_iter_from_depth<'store, R>(
        &self,
        store: &'store PS,
        range: R,
        from_block_depth: Option<BlockDepth>,
    ) -> Result<impl Iterator<Item = StoredOperation> + 'store, Error>
    where
        R: RangeBounds<OperationId>,
    {
        let iter = store.operations_iter(range)?.filter(move |op| {
            match (op.commit_status, from_block_depth) {
                (_, None) => true,
                (CommitStatus::Unknown, _) => true,
                (CommitStatus::Committed(_offset, op_depth), Some(from_depth)) => {
                    op_depth >= from_depth
                }
            }
        });

        Ok(iter)
    }

    ///
    /// Returns the block depth at which we filter operations from pending store with. See `PendingSyncConfig`.`operations_included_depth`.
    /// The depth from the request has priority, and then we fallback to the one in the sync_state.
    ///
    fn get_from_block_depth(
        &self,
        sync_context: &SyncContext,
        incoming_request_reader: Option<pending_sync_request::Reader>,
    ) -> Option<BlockDepth> {
        incoming_request_reader
            .and_then(|request_reader| {
                let block_depth = request_reader.get_from_block_depth();
                if block_depth != 0 {
                    Some(block_depth)
                } else {
                    None
                }
            })
            .or_else(|| {
                sync_context
                    .sync_state
                    .pending_last_cleanup_block
                    .map(|(_offset, depth)| depth + self.config.operations_depth_after_cleanup)
            })
    }
}

///
/// Synchronizer's configuration
///
#[derive(Copy, Clone, Debug)]
pub struct PendingSyncConfig {
    pub max_operations_per_range: u32,

    pub request_tracker_config: request_tracker::RequestTrackerConfig,

    ///
    /// Related to `CommitManagerConfig`.`operations_cleanup_after_block_depth`.
    /// This indicates how many blocks after the last cleaned up block we should include by
    /// default when doing sync requests, so that we don't request for operations that may
    /// have been cleaned up on other nodes.
    ///
    /// The `CommitManager` does cleanup at interval, and sets the last block that got cleaned
    /// in the `SyncState` up from the `PendingStore` because it was committed for more than
    /// `CommitManagerConfig`.`operations_cleanup_after_block_depth` of depth.
    ///
    /// This value is added to the `SyncState` last cleanup block depth to make sure we don't
    /// ask or include operations that got cleaned up.
    ///
    pub operations_depth_after_cleanup: BlockDepth,
}

impl Default for PendingSyncConfig {
    fn default() -> Self {
        PendingSyncConfig {
            max_operations_per_range: 30,
            request_tracker_config: request_tracker::RequestTrackerConfig::default(),
            operations_depth_after_cleanup: 2,
        }
    }
}

///
/// Synchronization information about a remote node
///
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

///
/// Converts bounds from sync_request range to SyncBounds
///
fn extract_sync_bounds(
    sync_range_reader: &pending_sync_range::Reader,
) -> Result<SyncBounds, Error> {
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

///
/// Collection of SyncRangeBuilder, taking into account maximum operations we want per range.
///
struct SyncRangesBuilder {
    config: PendingSyncConfig,
    ranges: Vec<SyncRangeBuilder>,
}

impl SyncRangesBuilder {
    fn new(config: PendingSyncConfig) -> SyncRangesBuilder {
        SyncRangesBuilder {
            config,
            ranges: Vec::new(),
        }
    }

    ///
    /// Pushes the given operation to the latest range, or to a new range if the latest is full.
    ///
    fn push_operation(&mut self, operation: StoredOperation, details: OperationDetails) {
        if self.ranges.is_empty() {
            self.push_new_range(Bound::Unbounded);
        } else {
            let last_range_size = self.ranges.last().map_or(0, |r| r.operations_count);
            if last_range_size > self.config.max_operations_per_range {
                let last_range_to = self.last_range_to_bound().expect("Should had a last range");

                // converted included into excluded for starting bound of next range since the item is in current range, not next one
                if let Bound::Included(to) = last_range_to {
                    self.push_new_range(Bound::Excluded(to));
                } else {
                    panic!("Expected current range end bound to be included");
                }
            }
        }

        let last_range = self
            .ranges
            .last_mut()
            .expect("Ranges should have had at least one range");
        last_range.push_operation(operation, details);
    }

    fn push_new_range(&mut self, from_bound: Bound<OperationId>) {
        self.ranges
            .push(SyncRangeBuilder::new(from_bound, Bound::Unbounded));
    }

    fn push_range(&mut self, sync_range: SyncRangeBuilder) {
        self.ranges.push(sync_range);
    }

    fn last_range_to_bound(&self) -> Option<Bound<OperationId>> {
        self.ranges.last().map(|r| r.to_operation)
    }

    fn set_last_range_to_bound(&mut self, to_bound: Bound<OperationId>) {
        if let Some(range) = self.ranges.last_mut() {
            range.to_operation = to_bound;
        }
    }
}

///
/// Builder for pending_sync_range messages. A pending sync range represents a range in the Pending Store to be synchronized
/// against a remote node's own store.
///
/// It can describe the operations in 3 ways:
///  * High level metadata (hash + count)
///  * Operations headers
///  * Operations full data
///
struct SyncRangeBuilder {
    from_operation: Bound<OperationId>,
    to_operation: Bound<OperationId>,

    operations: Vec<StoredOperation>,
    operations_headers: Vec<StoredOperation>,
    operations_count: u32,

    hasher: Option<Sha3_256>,
    hash: Option<Vec<u8>>,
}

#[derive(Copy, Clone)]
enum OperationDetails {
    Header,
    Full,
    None,
}

impl SyncRangeBuilder {
    fn new(
        from_operation: Bound<OperationId>,
        to_operation: Bound<OperationId>,
    ) -> SyncRangeBuilder {
        SyncRangeBuilder {
            from_operation,
            to_operation,
            operations: Vec::new(),
            operations_headers: Vec::new(),
            operations_count: 0,
            hasher: Some(Sha3_256::new()),
            hash: None,
        }
    }

    fn new_hashed(
        operations_range: (Bound<OperationId>, Bound<OperationId>),
        operations_hash: Vec<u8>,
        operations_count: u32,
    ) -> SyncRangeBuilder {
        SyncRangeBuilder {
            from_operation: operations_range.0,
            to_operation: operations_range.1,
            operations: Vec::new(),
            operations_headers: Vec::new(),
            operations_count,
            hasher: None,
            hash: Some(operations_hash),
        }
    }

    fn push_operation(&mut self, operation: StoredOperation, details: OperationDetails) {
        self.to_operation = Bound::Included(operation.operation_id);
        self.operations_count += 1;

        if let Some(hasher) = self.hasher.as_mut() {
            hasher.input_signed_frame(operation.frame.inner().inner())
        }

        match details {
            OperationDetails::Full => {
                self.operations.push(operation);
            }
            OperationDetails::Header => {
                self.operations_headers.push(operation);
            }
            OperationDetails::None => {
                // Only included in hash
            }
        }
    }

    fn write_into_sync_range_builder(
        self,
        range_msg_builder: &mut pending_sync_range::Builder,
    ) -> Result<(), Error> {
        match self.from_operation {
            Bound::Included(bound) => {
                range_msg_builder.set_from_included(true);
                range_msg_builder.set_from_operation(bound);
            }
            Bound::Excluded(bound) => {
                range_msg_builder.set_from_included(false);
                range_msg_builder.set_from_operation(bound);
            }
            Bound::Unbounded => {
                range_msg_builder.set_from_included(false);
                range_msg_builder.set_from_operation(0);
            }
        }

        match self.to_operation {
            Bound::Included(bound) => {
                range_msg_builder.set_to_included(true);
                range_msg_builder.set_to_operation(bound);
            }
            Bound::Excluded(bound) => {
                range_msg_builder.set_to_included(false);
                range_msg_builder.set_to_operation(bound);
            }
            Bound::Unbounded => {
                range_msg_builder.set_to_included(false);
                range_msg_builder.set_to_operation(0);
            }
        }

        range_msg_builder.set_operations_count(self.operations_count);

        if !self.operations_headers.is_empty() {
            let mut operations_headers_builder = range_msg_builder
                .reborrow()
                .init_operations_headers(self.operations_headers.len() as u32);
            for (i, operation) in self.operations_headers.iter().enumerate() {
                let mut op_header_builder = operations_headers_builder.reborrow().get(i as u32);
                op_header_builder.set_group_id(operation.group_id);
                op_header_builder.set_operation_id(operation.operation_id);

                let signature_data = operation.frame.inner().inner().multihash_bytes();
                op_header_builder.set_operation_signature(&signature_data);
            }
        }

        if !self.operations.is_empty() {
            let mut operations_builder = range_msg_builder
                .reborrow()
                .init_operations(self.operations.len() as u32);
            for (i, operation) in self.operations.iter().enumerate() {
                operations_builder.set(i as u32, operation.frame.whole_data());
            }
        }

        match (self.hash, self.hasher) {
            (Some(hash), _) => {
                range_msg_builder.set_operations_hash(&hash);
            }
            (_, Some(hasher)) => {
                range_msg_builder.set_operations_hash(&hasher.into_multihash_bytes());
            }
            _ => {}
        }

        Ok(())
    }
}

///
/// Pending Synchronization Error
///
#[derive(Clone, Debug, Fail)]
pub enum PendingSyncError {
    #[fail(display = "Got into an invalid synchronization state: {}", _0)]
    InvalidSyncState(String),
    #[fail(display = "Got an invalid sync request: {}", _0)]
    InvalidSyncRequest(String),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use exocore_common::serialization::protos::data_chain_capnp::{
        pending_operation, pending_operation_header,
    };

    use crate::engine::testing::create_dummy_new_entry_op;
    use crate::engine::testing::*;
    use crate::engine::{SyncContextMessage, SyncState};
    use crate::operation::{NewOperation, OperationType};

    use super::*;
    use crate::pending::CommitStatus;
    use crate::MemoryPendingStore;
    use exocore_common::framing::{CapnpFrameBuilder, FrameBuilder};
    use exocore_common::node::LocalNode;
    use std::time::{Duration, Instant};

    #[test]
    fn tick_send_to_other_nodes() -> Result<(), failure::Error> {
        // only one node, shouldn't send to ourself
        let mut cluster = EngineTestCluster::new(1);
        let mut sync_context = SyncContext::new(SyncState::default());
        cluster.pending_stores_synchronizer[0]
            .tick(&mut sync_context, &cluster.pending_stores[0])?;
        assert_eq!(sync_context.messages.len(), 0);

        // two nodes should send to other node
        let mut cluster = EngineTestCluster::new(2);
        let mut sync_context = SyncContext::new(SyncState::default());
        cluster.pending_stores_synchronizer[0]
            .tick(&mut sync_context, &cluster.pending_stores[0])?;
        assert_eq!(sync_context.messages.len(), 1);

        Ok(())
    }

    #[test]
    fn create_sync_range_request() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.pending_generate_dummy(0, 0, 100);

        let mut sync_context = SyncContext::new(SyncState::default());
        cluster.pending_stores_synchronizer[0]
            .tick(&mut sync_context, &cluster.pending_stores[0])?;
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
    fn create_sync_range_request_with_depth() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.clocks[0].set_fixed_instant(Instant::now());

        let config_depth_offset = cluster.pending_stores_synchronizer[0]
            .config
            .operations_depth_after_cleanup;

        // we update operations status as if they were all committed at depth 10
        let operations_id = cluster.pending_generate_dummy(0, 0, 100);
        for operation_id in operations_id {
            let status = CommitStatus::Committed(10, 10);
            cluster.pending_stores[0].update_operation_commit_status(operation_id, status)?;
        }

        // no filter should generate multiple ranges
        let mut sync_context = SyncContext::new(SyncState::default());
        cluster.pending_stores_synchronizer[0]
            .tick(&mut sync_context, &cluster.pending_stores[0])?;
        let (_to_node, sync_request_frame) = extract_request_from_result(&sync_context);
        let sync_request_reader: pending_sync_request::Reader = sync_request_frame.get_reader()?;
        assert_eq!(0, sync_request_reader.get_from_block_depth());
        let ranges = sync_request_reader.get_ranges()?;
        assert!(ranges.len() > 1);

        // filter with depth of 1000 should generate only 1 empty range since it matches no operations
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
        let mut sync_context = SyncContext::new(SyncState::default());
        sync_context.sync_state.pending_last_cleanup_block = Some((0, 1000));
        cluster.pending_stores_synchronizer[0]
            .tick(&mut sync_context, &cluster.pending_stores[0])?;
        let (_to_node, sync_request_frame) = extract_request_from_result(&sync_context);
        let sync_request_reader: pending_sync_request::Reader = sync_request_frame.get_reader()?;
        assert_eq!(
            1000 + config_depth_offset,
            sync_request_reader.get_from_block_depth()
        );
        let ranges = sync_request_reader.get_ranges()?;
        assert_eq!(ranges.len(), 1);
        assert_eq!(0, ranges.get(0).get_operations_count());

        Ok(())
    }

    #[test]
    fn new_operation_after_last_operation() -> Result<(), failure::Error> {
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
    fn new_operation_among_current_operations() -> Result<(), failure::Error> {
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
    fn handle_sync_equals() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.pending_generate_dummy(0, 0, 100);
        cluster.pending_generate_dummy(1, 0, 100);

        let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
        assert_eq!(count_a_to_b, 1);
        assert_eq!(count_b_to_a, 0);

        Ok(())
    }

    #[test]
    fn handle_sync_empty_to_many() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.pending_generate_dummy(0, 0, 100);

        let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
        assert_eq!(count_a_to_b, 2);
        assert_eq!(count_b_to_a, 1);

        Ok(())
    }

    #[test]
    fn handle_sync_many_to_empty() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.pending_generate_dummy(1, 1, 100);

        let (count_a_to_b, count_b_to_a) = sync_nodes(&mut cluster, 0, 1)?;
        assert_eq!(count_a_to_b, 1);
        assert_eq!(count_b_to_a, 1);

        Ok(())
    }

    #[test]
    fn handle_sync_full_to_some() -> Result<(), failure::Error> {
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
    fn handle_sync_some_to_all() -> Result<(), failure::Error> {
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
    fn handle_sync_different_some_to_different_some() -> Result<(), failure::Error> {
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
    fn handle_sync_cleaned_up_depth() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.clocks[0].set_fixed_instant(Instant::now());

        // we generate operations on node 0 spread in 10 blocks
        let operations_id = cluster.pending_generate_dummy(0, 0, 100);
        for operation_id in operations_id {
            let depth = operation_id / 10;
            let status = CommitStatus::Committed(depth, depth);

            cluster.pending_stores[0].update_operation_commit_status(operation_id, status)?;
        }

        // syncing 0 to 1 without depth filter should sync all operations
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
        sync_nodes(&mut cluster, 0, 1)?;
        assert_eq!(100, cluster.pending_stores[1].operations_count());

        // clear node 1 operations
        cluster.pending_stores[1].clear();

        // we mark node 0 as cleaned up up to block with depth 3
        // syncing should not sync non-matching operations to node 1
        cluster.sync_states[0].pending_last_cleanup_block = Some((3, 3));
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
        sync_nodes(&mut cluster, 0, 1)?;
        assert_eq!(51, cluster.pending_stores[1].operations_count());

        // syncing 1 to 0 without depth should not revive cleaned up operations
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(30));
        sync_nodes(&mut cluster, 1, 0)?;
        assert_eq!(51, cluster.pending_stores[1].operations_count());

        Ok(())
    }

    #[test]
    fn should_extract_from_block_offset() -> Result<(), failure::Error> {
        let cluster = EngineTestCluster::new(1);

        let pending_store = &cluster.pending_stores_synchronizer[0];

        let mut req_frame_builder = CapnpFrameBuilder::<pending_sync_request::Owned>::new();
        {
            let mut req_builder: pending_sync_request::Builder = req_frame_builder.get_builder();
            req_builder.set_from_block_depth(0);
        }

        // 0 in sync request means none
        let sync_context = SyncContext::new(SyncState::default());
        let frame = req_frame_builder.as_owned_frame();
        let frame_reader = frame.get_reader()?;
        let depth = pending_store.get_from_block_depth(&sync_context, Some(frame_reader));
        assert_eq!(None, depth);

        // in sync state
        let mut sync_context = SyncContext::new(SyncState::default());
        sync_context.sync_state.pending_last_cleanup_block = Some((0, 10));
        let frame = req_frame_builder.as_owned_frame();
        let frame_reader = frame.get_reader()?;
        let depth = pending_store.get_from_block_depth(&sync_context, Some(frame_reader));
        assert_eq!(Some(12), depth); // 10 + 2

        // request has priority
        let mut sync_context = SyncContext::new(SyncState::default());
        sync_context.sync_state.pending_last_cleanup_block = Some((0, 10));
        {
            let mut req_builder: pending_sync_request::Builder = req_frame_builder.get_builder();
            req_builder.set_from_block_depth(20);
        }
        let frame = req_frame_builder.as_owned_frame();
        let frame_reader = frame.get_reader()?;
        let depth = pending_store.get_from_block_depth(&sync_context, Some(frame_reader));
        assert_eq!(Some(20), depth);

        Ok(())
    }

    #[test]
    fn operations_iter_filtered_depth() -> Result<(), failure::Error> {
        let cluster = EngineTestCluster::new(1);

        let local_node = &cluster.nodes[0];
        let pending_store = &cluster.pending_stores_synchronizer[0];

        let mut store = MemoryPendingStore::new();
        store.put_operation(create_dummy_new_entry_op(&local_node, 100, 100))?;
        store.put_operation(create_dummy_new_entry_op(&local_node, 101, 101))?;
        store.put_operation(create_dummy_new_entry_op(&local_node, 102, 102))?;

        let res = pending_store
            .operations_iter_from_depth(&store, .., None)?
            .collect_vec();
        assert_eq!(3, res.len());

        // should return everything since they are all `Unknown` status
        let res = pending_store
            .operations_iter_from_depth(&store, .., Some(2))?
            .collect_vec();
        assert_eq!(3, res.len());

        // should return not committed
        store.update_operation_commit_status(100, CommitStatus::Unknown)?;
        let res = pending_store
            .operations_iter_from_depth(&store, .., Some(2))?
            .collect_vec();
        assert_eq!(3, res.len());

        // should return equal depth
        store.update_operation_commit_status(101, CommitStatus::Committed(0, 2))?;
        let res = pending_store
            .operations_iter_from_depth(&store, .., Some(2))?
            .collect_vec();
        assert_eq!(3, res.len());

        // should not return smaller depth
        let res = pending_store
            .operations_iter_from_depth(&store, .., Some(3))?
            .collect_vec();
        assert_eq!(2, res.len());

        Ok(())
    }

    #[test]
    fn sync_ranges_push_operation() {
        let local_node = LocalNode::generate();
        let mut sync_ranges = SyncRangesBuilder::new(PendingSyncConfig::default());
        for operation in stored_ops_generator(&local_node, 90) {
            sync_ranges.push_operation(operation, OperationDetails::None);
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
    fn sync_range_to_frame_builder_with_hash() -> Result<(), failure::Error> {
        let local_node = LocalNode::generate();
        let frames_builder = build_sync_ranges_frames(&local_node, 90, OperationDetails::None);
        assert_eq!(frames_builder.len(), 3);

        let frame0 = frames_builder[0].as_owned_frame();
        let frame0_reader: pending_sync_range::Reader = frame0.get_reader()?;
        let frame0_hash = frame0_reader.reborrow().get_operations_hash().unwrap();
        assert_eq!(frame0_reader.has_operations(), false);
        assert_eq!(frame0_reader.has_operations_headers(), false);

        let frame1 = frames_builder[1].as_owned_frame();
        let frame1_reader: pending_sync_range::Reader = frame1.get_reader()?;
        let frame1_hash = frame1_reader.reborrow().get_operations_hash()?;
        assert_eq!(frame1_reader.has_operations(), false);
        assert_eq!(frame1_reader.has_operations_headers(), false);

        assert_ne!(frame0_hash, frame1_hash);

        Ok(())
    }

    #[test]
    fn sync_range_to_frame_builder_with_headers() -> Result<(), failure::Error> {
        let local_node = LocalNode::generate();
        let frames_builder = build_sync_ranges_frames(&local_node, 90, OperationDetails::Header);

        let frame0 = frames_builder[0].as_owned_frame();
        let frame0_reader: pending_sync_range::Reader = frame0.get_reader()?;
        assert_eq!(frame0_reader.has_operations(), false);
        assert_eq!(frame0_reader.has_operations_headers(), true);

        let operations = frame0_reader.get_operations_headers()?;
        let operation0_header: pending_operation_header::Reader = operations.get(0);
        assert_eq!(operation0_header.get_group_id(), 2);

        Ok(())
    }

    #[test]
    fn sync_range_to_frame_builder_with_data() -> Result<(), failure::Error> {
        let local_node = LocalNode::generate();
        let frames_builder = build_sync_ranges_frames(&local_node, 90, OperationDetails::Full);

        let frame0 = frames_builder[0].as_owned_frame();
        let frame0_reader: pending_sync_range::Reader = frame0.get_reader()?;
        assert_eq!(frame0_reader.has_operations(), true);
        assert_eq!(frame0_reader.has_operations_headers(), false);

        let operations = frame0_reader.get_operations()?;
        let operation0_data = operations.get(0)?;
        let operation0_frame = crate::operation::read_operation_frame(operation0_data)?;

        let operation0_reader: pending_operation::Reader = operation0_frame.get_reader()?;
        let operation0_inner_reader = operation0_reader.get_operation();
        assert!(operation0_inner_reader.has_entry());

        Ok(())
    }

    fn sync_nodes(
        cluster: &mut EngineTestCluster,
        node_id_a: usize,
        node_id_b: usize,
    ) -> Result<(usize, usize), failure::Error> {
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
    ) -> Result<(usize, usize), failure::Error> {
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
        details: OperationDetails,
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

    fn print_sync_request<F: FrameReader>(
        request: &TypedCapnpFrame<F, pending_sync_request::Owned>,
    ) {
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

            if range.has_operations() {
                debug!("    Operations={}", range.get_operations().unwrap().len());
            } else {
                debug!("    Operations=None");
            }
        }
    }
}
