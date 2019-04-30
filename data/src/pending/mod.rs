use std::ops::RangeBounds;
use std::sync::Arc;
use std::vec::Vec;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::serialization::framed;

use crate::operation;
use crate::operation::{GroupID, OperationID};

pub mod memory;

///
/// Pending operations store. This store contains operations that have just been created and that
/// aren't committed to the chain yet.
///
pub trait PendingStore: Send + Sync + 'static {
    ///
    /// Add or replace the given operation into the store.
    /// Returns true if the operation already exists and got overwritten.
    ///
    fn put_operation(&mut self, operation: operation::NewOperation) -> Result<bool, Error>;

    fn get_operation(&self, operation_id: OperationID) -> Result<Option<StoredOperation>, Error>;

    fn get_group_operations(
        &self,
        group_id: GroupID,
    ) -> Result<Option<StoredOperationsGroup>, Error>;

    fn operations_iter<R>(&self, range: R) -> Result<TimelineIterator, Error>
    where
        R: RangeBounds<OperationID>;
}

pub type TimelineIterator<'store> = Box<dyn Iterator<Item = StoredOperation> + 'store>;

///
/// An operation stored in the pending store.
///
#[derive(Clone)]
pub struct StoredOperation {
    pub group_id: GroupID,
    pub operation_id: OperationID,
    pub operation_type: operation::OperationType,
    pub frame: Arc<framed::OwnedTypedFrame<pending_operation::Owned>>,
}

///
/// A group of operations related by their `group_id`.
/// Example: all operations related to a block proposal, approval and refusal
///
pub struct StoredOperationsGroup {
    pub group_id: GroupID,
    pub operations: Vec<StoredOperation>,
}

///
/// Error related to the pending store
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Operation related error: {:?}", _0)]
    Operation(#[fail(source)] operation::Error),
}

impl From<operation::Error> for Error {
    fn from(err: operation::Error) -> Self {
        Error::Operation(err)
    }
}
