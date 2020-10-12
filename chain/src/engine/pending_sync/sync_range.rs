use super::{OperationDetailsLevel, PendingSyncConfig};
use crate::engine::EngineError;
use crate::operation::OperationId;
use crate::pending::StoredOperation;
use exocore_core::framing::FrameReader;
use exocore_core::protos::generated::data_transport_capnp::pending_sync_range;
use exocore_core::sec::hash::{MultihashDigest, MultihashDigestExt, Sha3_256};
use std::ops::Bound;

/// Collection of SyncRangeBuilder, taking into account maximum operations we
/// want per range.
pub struct SyncRangesBuilder {
    config: PendingSyncConfig,
    pub ranges: Vec<SyncRangeBuilder>,
}

impl SyncRangesBuilder {
    pub fn new(config: PendingSyncConfig) -> SyncRangesBuilder {
        SyncRangesBuilder {
            config,
            ranges: Vec::new(),
        }
    }

    /// Pushes the given operation to the latest range, or to a new range if the
    /// latest is full.
    pub(super) fn push_operation(
        &mut self,
        operation: StoredOperation,
        details: OperationDetailsLevel,
    ) {
        if self.ranges.is_empty() {
            self.push_new_range(Bound::Unbounded);
        } else {
            let last_range_size = self.ranges.last().map_or(0, |r| r.operations_count);
            if last_range_size > self.config.max_operations_per_range {
                let last_range_to = self.last_range_to_bound().expect("Should had a last range");

                // converted included into excluded for starting bound of next range since the
                // item is in current range, not next one
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

    pub fn push_new_range(&mut self, from_bound: Bound<OperationId>) {
        self.ranges
            .push(SyncRangeBuilder::new(from_bound, Bound::Unbounded));
    }

    pub fn push_range(&mut self, sync_range: SyncRangeBuilder) {
        self.ranges.push(sync_range);
    }

    pub fn set_last_range_to_bound(&mut self, to_bound: Bound<OperationId>) {
        if let Some(range) = self.ranges.last_mut() {
            range.to_operation = to_bound;
        }
    }

    fn last_range_to_bound(&self) -> Option<Bound<OperationId>> {
        self.ranges.last().map(|r| r.to_operation)
    }
}

/// Builder for pending_sync_range messages. A pending sync range represents a
/// range in the Pending Store to be synchronized against a remote node's own
/// store.
///
/// It can describe the operations in 3 ways:
///  * High level metadata (hash + count)
///  * Operations headers
///  * Operations full data
pub struct SyncRangeBuilder {
    pub from_operation: Bound<OperationId>,
    pub to_operation: Bound<OperationId>,

    pub operations: Vec<StoredOperation>,
    pub operations_headers: Vec<StoredOperation>,
    pub operations_count: u32,

    pub hasher: Option<Sha3_256>,
    pub hash: Option<Vec<u8>>,
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
            hasher: Some(Sha3_256::default()),
            hash: None,
        }
    }

    pub(crate) fn new_hashed(
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

    fn push_operation(&mut self, operation: StoredOperation, details: OperationDetailsLevel) {
        self.to_operation = Bound::Included(operation.operation_id);
        self.operations_count += 1;

        if let Some(hasher) = self.hasher.as_mut() {
            hasher.input_signed_frame(operation.frame.inner().inner())
        }

        match details {
            OperationDetailsLevel::Full => {
                self.operations.push(operation);
            }
            OperationDetailsLevel::Header => {
                self.operations_headers.push(operation);
            }
            OperationDetailsLevel::None => {
                // Only included in hash
            }
        }
    }

    pub(crate) fn write_into_sync_range_builder(
        self,
        range_msg_builder: &mut pending_sync_range::Builder,
    ) -> Result<(), EngineError> {
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
                .init_operations_frames(self.operations.len() as u32);
            for (i, operation) in self.operations.iter().enumerate() {
                operations_builder.set(i as u32, operation.frame.whole_data());
            }
        }

        match (self.hash, self.hasher) {
            (Some(hash), _) => {
                range_msg_builder.set_operations_hash(&hash);
            }
            (_, Some(hasher)) => {
                range_msg_builder.set_operations_hash(&hasher.result().into_bytes());
            }
            _ => {}
        }

        Ok(())
    }
}
