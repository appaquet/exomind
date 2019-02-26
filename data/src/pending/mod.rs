use std::ops::RangeBounds;
use std::sync::Arc;
use std::vec::Vec;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::security::hash::Multihash;
use exocore_common::serialization::framed;
use exocore_common::serialization::protos::{OperationID, PendingID};

pub mod memory;

pub trait Store: Send + 'static {
    fn put_operation(
        &mut self,
        operation: framed::OwnedTypedFrame<pending_operation::Owned>,
    ) -> Result<(), Error>;

    fn get_entry_operations(
        &self,
        entry_id: PendingID,
    ) -> Result<Option<StoredEntryOperations>, Error>;

    fn operations_iter<'store, R>(
        &'store self,
        range: R,
    ) -> Result<Box<dyn Iterator<Item = StoredOperation> + 'store>, Error>
    where
        R: RangeBounds<OperationID>;

    fn operations_range_summary<R>(&self, range: R) -> Result<StoredRangeSummary, Error>
    where
        R: RangeBounds<OperationID>;
}

pub type TimelineIterator<'store> = Box<dyn Iterator<Item = StoredOperation> + 'store>;

pub struct StoredOperation {
    pub entry_id: PendingID,
    pub operation_id: OperationID,
}

pub struct StoredEntryOperations {
    pub entry_id: PendingID,
    pub operations: Vec<Arc<framed::OwnedTypedFrame<pending_operation::Owned>>>,
}

pub struct StoredRangeSummary {
    pub count: u32,
    pub hash: Multihash,
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error in message serialization")]
    Serialization(#[fail(cause)] framed::Error),
}

impl From<framed::Error> for Error {
    fn from(err: framed::Error) -> Self {
        Error::Serialization(err)
    }
}
