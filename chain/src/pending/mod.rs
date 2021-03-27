use std::{ops::RangeBounds, sync::Arc, vec::Vec};

use crate::{
    block, operation,
    operation::{GroupId, OperationId},
};

pub mod error;
use bytes::Bytes;
pub use error::Error;
#[cfg(feature = "memory-pending")]
pub mod memory;

/// Pending operations store. This store contains operations that have just been
/// created and that aren't committed to the chain yet.
pub trait PendingStore: Send + Sync + 'static {
    /// Adds or replaces the given operation into the store.
    /// Returns true if the operation already exists and got overwritten.
    fn put_operation(&mut self, operation: operation::NewOperation) -> Result<bool, Error>;

    /// Updates the commit status of an operation. This information is not
    /// replicated, and is populated by the `CommitManager` so that the
    /// `PendingSynchronizer` and `Engine` can get the chain status of an
    /// operation without hitting the chain every time.

    /// Returns `Error::NotFound` if operation doesn't exist.
    fn update_operation_commit_status(
        &mut self,
        operation_id: OperationId,
        status: CommitStatus,
    ) -> Result<(), Error>;

    /// Returns the operation with given id.
    fn get_operation(&self, operation_id: OperationId) -> Result<Option<StoredOperation>, Error>;

    /// Returns all operations grouped under the given group id / operation id.
    /// An example of operation group is a block with its signatures /
    /// refusals operations. Entry operations are NOT stored in the block's
    /// group since they could get added into different blocks (but only one
    /// will be committed).
    fn get_group_operations(
        &self,
        group_id: GroupId,
    ) -> Result<Option<StoredOperationsGroup>, Error>;

    /// Iterates through all operations in the store within the given range.
    /// The iterator returns operations sorted by operation ids.
    fn operations_iter<R>(&self, range: R) -> Result<TimelineIterator, Error>
    where
        R: RangeBounds<OperationId>;

    /// Returns the number of operations in the store.
    fn operations_count(&self) -> usize;

    /// Deletes the operation with given id, or all operations grouped by this
    /// operation id if the operation was a group (ex: block with its
    /// signatures)
    fn delete_operation(&mut self, operation_id: OperationId) -> Result<(), Error>;
}

pub type TimelineIterator<'store> = Box<dyn Iterator<Item = StoredOperation> + 'store>;

/// An operation stored in the pending store.
#[derive(Clone)]
pub struct StoredOperation {
    pub group_id: GroupId,
    pub operation_id: OperationId,
    pub operation_type: operation::OperationType,
    pub commit_status: CommitStatus,
    pub frame: Arc<operation::OperationFrame<Bytes>>,
}

/// A group of operations related by their `group_id`.
/// Example: all operations related to a block proposal, approval and refusal
pub struct StoredOperationsGroup {
    pub group_id: GroupId,
    pub operations: Vec<StoredOperation>,
}

/// Chain status of an operation in the pending store, indicating if it's in the
/// chain or not, and at what height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitStatus {
    Unknown,
    Committed(block::BlockOffset, block::BlockHeight),
}
