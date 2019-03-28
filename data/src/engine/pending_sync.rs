#![allow(dead_code)]

use std::collections::HashSet;
use std::ops::Bound;
use std::ops::RangeBounds;

use itertools::{EitherOrBoth, Itertools};

use exocore_common::security::hash::{Sha3Hasher, StreamHasher};
use exocore_common::serialization::capnp;
use exocore_common::serialization::framed;
use exocore_common::serialization::framed::*;
use exocore_common::serialization::protos::data_chain_capnp::{
    pending_operation, pending_operation_header,
};
use exocore_common::serialization::protos::data_transport_capnp::{
    pending_sync_range, pending_sync_request,
};
use exocore_common::serialization::protos::OperationID;

use crate::pending;
use crate::pending::{Store, StoredOperation};

const MAX_OPERATIONS_PER_RANGE: u32 = 30;

///
/// Synchronizes local pending store against remote nodes' pending stores. It does that by exchanging
/// PendingSyncRequest messages.
///
/// This PendingSyncRequest message contains information about ranges of operations (by `OperationID`) in a local pending store,
/// and is sent to be applied / compared to the remote pending store. If there are differences in the remote store, the remote
/// nodes reply with a request that represents the intent of the remote store. That intent could be to request missing data,
/// or send missing data.
///
/// The whole store could be compared as a while (no boundaries), but that would result in excessive data transmission, because
/// a single difference would require the whole store to be compared. In order to minimize this, when building ranges, a node
/// tries to limit the number of operations by range. If a single range is not equal, only this range will be compared via
/// headers exchange and full operations exchange.
///
pub struct Synchronizer<PS: Store> {
    phantom: std::marker::PhantomData<PS>,
}

impl<PS: Store> Synchronizer<PS> {
    pub fn new() -> Synchronizer<PS> {
        Synchronizer {
            phantom: std::marker::PhantomData,
        }
    }

    pub fn create_sync_request(
        &self,
        store: &PS,
    ) -> Result<FrameBuilder<pending_sync_request::Owned>, Error> {
        let mut sync_ranges = SyncRangesBuilder::new();
        for operation in store.operations_iter(..)? {
            sync_ranges.push_operation(operation, OperationDetails::None);
        }

        // make sure we have at least one range
        if sync_ranges.ranges.is_empty() {
            sync_ranges.create_new_range(0);
        }

        // make sure last range has an infinite upper bound
        sync_ranges.set_last_range_to(0);

        let mut sync_request_frame_builder = FrameBuilder::<pending_sync_request::Owned>::new();
        let mut sync_request_builder = sync_request_frame_builder.get_builder_typed();
        let mut ranges_builder = sync_request_builder
            .reborrow()
            .init_ranges(sync_ranges.ranges.len() as u32);
        for (i, range) in sync_ranges.ranges.into_iter().enumerate() {
            let mut builder = ranges_builder.reborrow().get(i as u32);
            range.write_into_sync_range_builder(&mut builder)?;
        }

        Ok(sync_request_frame_builder)
    }

    pub fn handle_incoming_sync_request(
        &mut self,
        store: &mut PS,
        request: OwnedTypedFrame<pending_sync_request::Owned>,
    ) -> Result<Option<FrameBuilder<pending_sync_request::Owned>>, Error> {
        let in_reader: pending_sync_request::Reader = request.get_typed_reader()?;
        let in_ranges = in_reader.get_ranges()?;

        if let Some(out_ranges) = self.handle_incoming_sync_ranges(store, in_ranges.iter())? {
            let mut sync_request_frame_builder = FrameBuilder::<pending_sync_request::Owned>::new();
            let mut sync_request_builder = sync_request_frame_builder.get_builder_typed();

            let mut ranges_builder = sync_request_builder
                .reborrow()
                .init_ranges(out_ranges.ranges.len() as u32);
            for (i, range) in out_ranges.ranges.into_iter().enumerate() {
                let mut builder = ranges_builder.reborrow().get(i as u32);
                range.write_into_sync_range_builder(&mut builder)?;
            }

            Ok(Some(sync_request_frame_builder))
        } else {
            Ok(None)
        }
    }

    fn handle_incoming_sync_ranges<'a, I>(
        &mut self,
        store: &mut PS,
        sync_range_iterator: I,
    ) -> Result<Option<SyncRangesBuilder>, Error>
    where
        I: Iterator<Item = pending_sync_range::Reader<'a>>,
    {
        let mut out_ranges_contains_changes = false;
        let mut out_ranges = SyncRangesBuilder::new();

        for sync_range_reader in sync_range_iterator {
            let (bounds, bounds_from, bounds_to) = Self::extract_sync_2ounds(&sync_range_reader)?;
            if bounds_to < bounds_from && bounds_to != 0 {
                return Err(Error::InvalidSyncRequest(format!(
                    "Request from={} > to={}",
                    bounds_from, bounds_to
                )));
            }

            // first, apply all operations
            let mut included_operations = HashSet::<OperationID>::new();
            if sync_range_reader.has_operations() {
                for operation_frame_res in sync_range_reader.get_operations()?.iter() {
                    let operation_frame_data = operation_frame_res?;
                    let operation_frame =
                        TypedSliceFrame::<pending_operation::Owned>::new(operation_frame_data)?
                            .to_owned();

                    let operation_frame_reader = operation_frame.get_typed_reader()?;
                    let operation_id = operation_frame_reader.get_operation_id();
                    included_operations.insert(operation_id);

                    store.put_operation(operation_frame)?;
                }
            }

            // then check local store's range hash and count
            let (local_hash, local_count) = Self::local_store_range_info(store, bounds)?;
            let remote_hash = sync_range_reader.get_operations_hash()?;
            let remote_count = sync_range_reader.get_operations_count();

            if remote_hash == &local_hash[..] && local_count == remote_count as usize {
                // we are equal to remote, nothing to do
                out_ranges.push_range(SyncRangeBuilder::new_hashed(
                    bounds_from,
                    bounds_to,
                    local_hash,
                    local_count as u32,
                ));
            } else if remote_count == 0 {
                // remote has no data, we sent everything
                out_ranges_contains_changes = true;
                out_ranges.create_new_range(bounds_from);
                for operation in store.operations_iter(bounds)? {
                    out_ranges.push_operation(operation, OperationDetails::Full);
                }
                out_ranges.set_last_range_to(bounds_to);
            } else if !sync_range_reader.has_operations_headers()
                && !sync_range_reader.has_operations()
            {
                // remote has only sent us hash, we reply with headers
                out_ranges_contains_changes = true;
                out_ranges.create_new_range(bounds_from);
                for operation in store.operations_iter(bounds)? {
                    out_ranges.push_operation(operation, OperationDetails::Header);
                }
                out_ranges.set_last_range_to(bounds_to);
            } else {
                // remote and local has differences. We do a diff
                out_ranges_contains_changes = true;
                out_ranges.create_new_range(bounds_from);

                let remote_iter = sync_range_reader.get_operations_headers()?.iter();
                let local_iter = store.operations_iter(bounds)?;
                Self::diff_local_remote_range(
                    &mut out_ranges,
                    &mut included_operations,
                    remote_iter,
                    local_iter,
                )?;

                out_ranges.set_last_range_to(bounds_to);
            }
        }

        if out_ranges_contains_changes {
            Ok(Some(out_ranges))
        } else {
            Ok(None)
        }
    }

    fn local_store_range_info<R>(store: &PS, range: R) -> Result<(Vec<u8>, usize), Error>
    where
        R: RangeBounds<OperationID>,
    {
        let mut frame_hasher = FramesHasher::new();
        let mut count = 0;
        for operation in store.operations_iter(range)? {
            frame_hasher.consume_frame(operation.operation.as_ref());
            count += 1;
        }

        Ok((frame_hasher.into_multihash_bytes(), count))
    }

    fn extract_sync_2ounds(
        sync_range_reader: &pending_sync_range::Reader,
    ) -> Result<
        (
            (Bound<OperationID>, Bound<OperationID>),
            OperationID,
            OperationID,
        ),
        Error,
    > {
        let (from, to) = (
            sync_range_reader.get_from_operation(),
            sync_range_reader.get_to_operation(),
        );

        let bounds = match (
            sync_range_reader.get_from_operation(),
            sync_range_reader.get_to_operation(),
        ) {
            (0, 0) => (Bound::Unbounded, Bound::Unbounded),
            (0, up) => (Bound::Unbounded, Bound::Included(up)),
            (low, 0) => (Bound::Excluded(low), Bound::Unbounded),
            (low, up) => (Bound::Excluded(low), Bound::Included(up)),
        };

        Ok((bounds, from, to))
    }

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
                    }
                }
                EitherOrBoth::Both(_remote_op, local_op) => {
                    out_ranges.push_operation(local_op, OperationDetails::Header);
                }
            }
        }
        if !diff_has_difference {
            return Err(Error::InvalidSyncState("Got into diff branch, but didn't result in any changes, which shouldn't have happened".to_string()));
        }

        Ok(())
    }
}

///
/// Collection of SyncRangeBuilder, taking into account maximum operations we want per range.
///
struct SyncRangesBuilder {
    ranges: Vec<SyncRangeBuilder>,
}

impl SyncRangesBuilder {
    fn new() -> SyncRangesBuilder {
        SyncRangesBuilder { ranges: Vec::new() }
    }

    fn push_operation(&mut self, operation: StoredOperation, details: OperationDetails) {
        if self.ranges.is_empty() {
            self.create_new_range(0);
        } else {
            let last_range_size = self.ranges.last().map(|r| r.operations_count).unwrap_or(0);
            if last_range_size > MAX_OPERATIONS_PER_RANGE {
                let last_range_to = self.last_range_to().expect("Should had a last range");
                self.create_new_range(last_range_to);
            }
        }

        let last_range = self
            .ranges
            .last_mut()
            .expect("Ranges should have had at least one range");
        last_range.push_operation(operation, details);
    }

    fn create_new_range(&mut self, from_operation_id: OperationID) {
        self.ranges
            .push(SyncRangeBuilder::new(from_operation_id, 0));
    }

    fn push_range(&mut self, sync_range: SyncRangeBuilder) {
        self.ranges.push(sync_range);
    }

    fn last_range_to(&self) -> Option<OperationID> {
        self.ranges.last().map(|r| r.to_operation)
    }

    fn set_last_range_to(&mut self, operation_id: OperationID) {
        if let Some(range) = self.ranges.last_mut() {
            range.to_operation = operation_id;
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
    from_operation: OperationID,
    to_operation: OperationID,

    operations: Vec<StoredOperation>,
    operations_headers: Vec<StoredOperation>,
    operations_count: u32,

    hasher: Option<FramesHasher>,
    hash: Option<Vec<u8>>,
}

#[derive(Copy, Clone)]
enum OperationDetails {
    Header,
    Full,
    None,
}

impl SyncRangeBuilder {
    fn new(from_operation: OperationID, to_operation: OperationID) -> SyncRangeBuilder {
        SyncRangeBuilder {
            from_operation,
            to_operation,
            operations: Vec::new(),
            operations_headers: Vec::new(),
            operations_count: 0,
            hasher: Some(FramesHasher::new()),
            hash: None,
        }
    }

    fn new_hashed(
        from_operation: OperationID,
        to_operation: OperationID,
        operations_hash: Vec<u8>,
        operations_count: u32,
    ) -> SyncRangeBuilder {
        SyncRangeBuilder {
            from_operation,
            to_operation,
            operations: Vec::new(),
            operations_headers: Vec::new(),
            operations_count,
            hasher: None,
            hash: Some(operations_hash),
        }
    }

    fn push_operation(&mut self, operation: StoredOperation, details: OperationDetails) {
        self.to_operation = operation.operation_id;
        self.operations_count += 1;

        if let Some(hasher) = self.hasher.as_mut() {
            hasher.consume_frame(operation.operation.as_ref())
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
        range_msg_builder.set_from_operation(self.from_operation);
        range_msg_builder.set_to_operation(self.to_operation);
        range_msg_builder.set_operations_count(self.operations_count);

        if !self.operations_headers.is_empty() {
            let mut operations_headers_builder = range_msg_builder
                .reborrow()
                .init_operations_headers(self.operations_headers.len() as u32);
            for (i, operation) in self.operations_headers.iter().enumerate() {
                let mut op_header_builder = operations_headers_builder.reborrow().get(i as u32);
                op_header_builder.set_group_id(operation.group_id);
                op_header_builder.set_operation_id(operation.operation_id);

                let signature_data = operation
                    .operation
                    .signature_data()
                    .expect("The frame didn't have a signature");
                op_header_builder.set_operation_signature(&signature_data);
            }
        }

        if !self.operations.is_empty() {
            let mut operations_builder = range_msg_builder
                .reborrow()
                .init_operations(self.operations.len() as u32);
            for (i, operation) in self.operations.iter().enumerate() {
                operations_builder.set(i as u32, operation.operation.frame_data());
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
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error in pending store: {:?}", _0)]
    Store(#[fail(cause)] pending::Error),
    #[fail(display = "Error in framing serialization: {:?}", _0)]
    Framing(#[fail(cause)] framed::Error),
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),
    #[fail(display = "Got into an invalid synchronization state: {}", _0)]
    InvalidSyncState(String),
    #[fail(display = "Got an invalid sync request: {}", _0)]
    InvalidSyncRequest(String),
}

impl From<pending::Error> for Error {
    fn from(err: pending::Error) -> Self {
        Error::Store(err)
    }
}

impl From<framed::Error> for Error {
    fn from(err: framed::Error) -> Self {
        Error::Framing(err)
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}

///
///
///
struct FramesHasher {
    hasher: Sha3Hasher,
}

impl FramesHasher {
    fn new() -> FramesHasher {
        FramesHasher {
            hasher: Sha3Hasher::new_256(),
        }
    }

    fn consume_frame<F: SignedFrame>(&mut self, frame: &F) {
        let signature_data = frame
            .signature_data()
            .expect("The frame didn't have a signature");
        self.hasher.consume(signature_data);
    }

    fn into_multihash_bytes(self) -> Vec<u8> {
        self.hasher.into_multihash_bytes()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use exocore_common::serialization::protos::data_chain_capnp::{
        pending_operation, pending_operation_header,
    };

    use crate::pending::tests::create_new_entry_op;

    use super::*;

    #[test]
    fn create_sync_range_request() {
        let mut store = crate::pending::memory::MemoryStore::new();
        for operation in pending_ops_generator(100) {
            store.put_operation(operation).unwrap();
        }

        let synchronizer = Synchronizer::new();
        let sync_request = synchronizer.create_sync_request(&store).unwrap();

        let sync_request_frame = sync_request.as_owned_framed(NullFrameSigner).unwrap();
        let sync_request_reader: pending_sync_request::Reader =
            sync_request_frame.get_typed_reader().unwrap();

        let ranges = sync_request_reader.get_ranges().unwrap();
        assert_eq!(ranges.len(), 4);

        let range0: pending_sync_range::Reader = ranges.get(0);
        assert_eq!(range0.get_from_operation(), 0);

        let range1: pending_sync_range::Reader = ranges.get(1);
        assert_eq!(range0.get_to_operation(), range1.get_from_operation());

        let range3: pending_sync_range::Reader = ranges.get(3);
        assert_eq!(range3.get_to_operation(), 0);
    }

    #[test]
    fn handle_sync_equals() {
        let mut store_1 = crate::pending::memory::MemoryStore::new();
        let mut store_2 = crate::pending::memory::MemoryStore::new();
        for operation in pending_ops_generator(100) {
            store_1.put_operation(operation.clone()).unwrap();
            store_2.put_operation(operation).unwrap();
        }

        let mut sync_1 = Synchronizer::new();
        let mut sync_2 = Synchronizer::new();

        let (count_a_to_b, count_b_to_a) =
            test_sync_stores(&mut store_1, &mut sync_1, &mut store_2, &mut sync_2);

        assert_eq!(count_a_to_b, 1);
        assert_eq!(count_b_to_a, 0);
    }

    #[test]
    fn handle_sync_empty_to_many() {
        let mut store_1 = crate::pending::memory::MemoryStore::new();
        let mut sync_1 = Synchronizer::new();
        for operation in pending_ops_generator(100) {
            store_1.put_operation(operation).unwrap();
        }

        let mut store_2 = crate::pending::memory::MemoryStore::new();
        let mut sync_2 = Synchronizer::new();

        let (count_a_to_b, count_b_to_a) =
            test_sync_stores(&mut store_1, &mut sync_1, &mut store_2, &mut sync_2);

        assert_eq!(count_a_to_b, 2);
        assert_eq!(count_b_to_a, 1);
    }

    #[test]
    fn handle_sync_many_to_empty() {
        let mut store_1 = crate::pending::memory::MemoryStore::new();
        let mut sync_1 = Synchronizer::new();

        let mut store_2 = crate::pending::memory::MemoryStore::new();
        let mut sync_2 = Synchronizer::new();
        for operation in pending_ops_generator(100) {
            store_2.put_operation(operation).unwrap();
        }

        let (count_a_to_b, count_b_to_a) =
            test_sync_stores(&mut store_1, &mut sync_1, &mut store_2, &mut sync_2);

        assert_eq!(count_a_to_b, 1);
        assert_eq!(count_b_to_a, 1);
    }

    #[test]
    fn handle_sync_1ll_to_some() {
        let mut store_1 = crate::pending::memory::MemoryStore::new();
        let mut sync_1 = Synchronizer::new();
        for operation in pending_ops_generator(100) {
            store_1.put_operation(operation).unwrap();
        }

        let mut store_2 = crate::pending::memory::MemoryStore::new();
        let mut sync_2 = Synchronizer::new();
        for operation in pending_ops_generator(100) {
            let reader = operation.get_typed_reader().unwrap();
            if reader.get_operation_id() % 2 == 0 {
                store_2.put_operation(operation).unwrap();
            }
        }

        let (count_a_to_b, count_b_to_a) =
            test_sync_stores(&mut store_1, &mut sync_1, &mut store_2, &mut sync_2);

        assert_eq!(count_a_to_b, 2);
        assert_eq!(count_b_to_a, 1);
    }

    #[test]
    fn handle_sync_some_to_all() {
        let mut store_1 = crate::pending::memory::MemoryStore::new();
        let mut sync_1 = Synchronizer::new();
        for operation in pending_ops_generator(100) {
            let reader = operation.get_typed_reader().unwrap();
            if reader.get_operation_id() % 2 == 0 {
                store_1.put_operation(operation).unwrap();
            }
        }

        let mut store_2 = crate::pending::memory::MemoryStore::new();
        let mut sync_2 = Synchronizer::new();
        for operation in pending_ops_generator(100) {
            store_2.put_operation(operation).unwrap();
        }

        let (count_a_to_b, count_b_to_a) =
            test_sync_stores(&mut store_1, &mut sync_1, &mut store_2, &mut sync_2);

        assert_eq!(count_a_to_b, 2);
        assert_eq!(count_b_to_a, 2);
    }

    #[test]
    fn sync_ranges_push_operation() {
        let mut sync_ranges = SyncRangesBuilder::new();
        for operation in stored_ops_generator(90) {
            sync_ranges.push_operation(operation, OperationDetails::None);
        }

        assert_eq!(sync_ranges.ranges.len(), 3);
        assert_eq!(
            sync_ranges.ranges.first().map(|r| r.from_operation),
            Some(0)
        );

        // check continuity of ranges
        let mut last_range_to: Option<OperationID> = None;
        for range in sync_ranges.ranges.iter() {
            assert_eq!(range.from_operation, last_range_to.unwrap_or(0));
            last_range_to = Some(range.to_operation);
        }

        assert_eq!(last_range_to, Some(90));
    }

    #[test]
    fn sync_range_to_frame_builder_with_hash() {
        let frames_builder = build_sync_ranges_frames(90, OperationDetails::None);
        assert_eq!(frames_builder.len(), 3);

        let frame0 = frames_builder[0].as_owned_unsigned_framed().unwrap();
        let frame0_reader: pending_sync_range::Reader = frame0.get_typed_reader().unwrap();
        let frame0_hash = frame0_reader.reborrow().get_operations_hash().unwrap();
        assert_eq!(frame0_reader.has_operations(), false);
        assert_eq!(frame0_reader.has_operations_headers(), false);

        let frame1 = frames_builder[1].as_owned_unsigned_framed().unwrap();
        let frame1_reader: pending_sync_range::Reader = frame1.get_typed_reader().unwrap();
        let frame1_hash = frame1_reader.reborrow().get_operations_hash().unwrap();
        assert_eq!(frame1_reader.has_operations(), false);
        assert_eq!(frame1_reader.has_operations_headers(), false);

        assert_ne!(frame0_hash, frame1_hash);
    }

    #[test]
    fn sync_range_to_frame_builder_with_headers() {
        let frames_builder = build_sync_ranges_frames(90, OperationDetails::Header);

        let frame0 = frames_builder[0].as_owned_unsigned_framed().unwrap();
        let frame0_reader: pending_sync_range::Reader = frame0.get_typed_reader().unwrap();
        assert_eq!(frame0_reader.has_operations(), false);
        assert_eq!(frame0_reader.has_operations_headers(), true);

        let operations = frame0_reader.get_operations_headers().unwrap();
        let operation0_header: pending_operation_header::Reader = operations.get(0);
        assert_eq!(operation0_header.get_group_id(), 2);
    }

    #[test]
    fn sync_range_to_frame_builder_with_data() {
        let frames_builder = build_sync_ranges_frames(90, OperationDetails::Full);

        let frame0 = frames_builder[0].as_owned_unsigned_framed().unwrap();
        let frame0_reader: pending_sync_range::Reader = frame0.get_typed_reader().unwrap();
        assert_eq!(frame0_reader.has_operations(), true);
        assert_eq!(frame0_reader.has_operations_headers(), false);

        let operations = frame0_reader.get_operations().unwrap();
        let operation0_data = operations.get(0).unwrap();
        let operation0_frame =
            TypedSliceFrame::<pending_operation::Owned>::new(operation0_data).unwrap();

        let operation0_reader: pending_operation::Reader =
            operation0_frame.get_typed_reader().unwrap();
        let operation0_inner_reader = operation0_reader.get_operation();
        assert!(operation0_inner_reader.has_entry_new());
    }

    fn test_sync_stores<PS>(
        store_1: &mut PS,
        sync_1: &mut Synchronizer<PS>,
        store_2: &mut PS,
        sync_2: &mut Synchronizer<PS>,
    ) -> (usize, usize)
    where
        PS: Store,
    {
        let mut count_1_to_2 = 0;
        let mut count_2_to_1 = 0;

        let mut next_request = sync_1
            .create_sync_request(&store_1)
            .unwrap()
            .as_owned_unsigned_framed()
            .unwrap();
        print_sync_request(&next_request);

        loop {
            count_1_to_2 += 1;
            let resp = sync_2
                .handle_incoming_sync_request(store_2, next_request)
                .unwrap();
            if resp.is_none() {
                break;
            }

            count_2_to_1 += 1;
            let request = resp.unwrap().as_owned_unsigned_framed().unwrap();
            print_sync_request(&request);
            let resp = sync_1
                .handle_incoming_sync_request(store_1, request)
                .unwrap();
            match resp {
                Some(resp) => next_request = resp.as_owned_unsigned_framed().unwrap(),
                None => break,
            }
            print_sync_request(&next_request);
        }

        (count_1_to_2, count_2_to_1)
    }

    fn build_sync_ranges_frames(
        count: usize,
        details: OperationDetails,
    ) -> Vec<FrameBuilder<pending_sync_range::Owned>> {
        let mut sync_ranges = SyncRangesBuilder::new();
        for operation in stored_ops_generator(count) {
            sync_ranges.push_operation(operation, details);
        }
        sync_ranges
            .ranges
            .into_iter()
            .map(|range| {
                let mut range_frame_builder = FrameBuilder::<pending_sync_range::Owned>::new();
                let mut range_msg_builder = range_frame_builder.get_builder_typed();
                range
                    .write_into_sync_range_builder(&mut range_msg_builder)
                    .unwrap();
                range_frame_builder
            })
            .collect()
    }

    fn pending_ops_generator(
        count: usize,
    ) -> impl Iterator<Item = OwnedTypedFrame<pending_operation::Owned>> {
        (1..=count).map(|i| {
            let (group_id, operation_id) = ((i % 10 + 1) as u64, i as u64);
            create_new_entry_op(operation_id, group_id)
        })
    }

    fn stored_ops_generator(count: usize) -> impl Iterator<Item = StoredOperation> {
        (1..=count).map(|i| {
            let (group_id, operation_id) = ((i % 10 + 1) as u64, i as u64);
            let operation = Arc::new(create_new_entry_op(operation_id, group_id));

            StoredOperation {
                group_id,
                operation_id,
                operation,
            }
        })
    }

    fn print_sync_request(request: &OwnedTypedFrame<pending_sync_request::Owned>) {
        let reader: pending_sync_request::Reader = request.get_typed_reader().unwrap();
        let ranges = reader.get_ranges().unwrap();

        debug!("Request ------------");
        for range in ranges.iter() {
            debug!(
                " Range {} to {}",
                range.get_from_operation(),
                range.get_to_operation()
            );
            debug!("   Hash={:?}", range.get_operations_hash().unwrap());
            debug!("   Count={}", range.get_operations_count());

            if range.has_operations_headers() {
                debug!(
                    "   Headers={}",
                    range.get_operations_headers().unwrap().len()
                );
            } else {
                debug!("   Headers=None");
            }

            if range.has_operations() {
                debug!("   Operations={}", range.get_operations().unwrap().len());
            } else {
                debug!("   Operations=None");
            }
        }

        debug!("-------------------");
    }
}
