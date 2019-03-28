use std::ops::RangeBounds;
use std::sync::Arc;
use std::vec::Vec;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::security::hash::Multihash;
use exocore_common::serialization::framed;
use exocore_common::serialization::protos::{GroupID, OperationID};

pub mod memory;

pub trait Store: Send + Sync + 'static {
    fn put_operation(
        &mut self,
        operation: framed::OwnedTypedFrame<pending_operation::Owned>,
    ) -> Result<(), Error>;

    fn get_group_operations(
        &self,
        group_id: GroupID,
    ) -> Result<Option<StoredGroupOperations>, Error>;

    fn operations_iter<R>(&self, range: R) -> Result<TimelineIterator, Error>
    where
        R: RangeBounds<OperationID>;

    fn operations_range_summary<R>(&self, range: R) -> Result<StoredRangeSummary, Error>
    where
        R: RangeBounds<OperationID>;
}

pub type TimelineIterator<'store> = Box<dyn Iterator<Item = StoredOperation> + 'store>;

#[derive(Clone)]
pub struct StoredOperation {
    pub group_id: GroupID,
    pub operation_id: OperationID,
    pub operation: Arc<framed::OwnedTypedFrame<pending_operation::Owned>>,
}

pub struct StoredGroupOperations {
    pub group_id: GroupID,
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

#[cfg(test)]
pub mod tests {
    use exocore_common::serialization::framed::{FrameBuilder, MultihashFrameSigner};

    use super::*;

    pub fn create_new_entry_op(
        operation_id: OperationID,
        group_id: GroupID,
    ) -> framed::OwnedTypedFrame<pending_operation::Owned> {
        let mut msg_builder = FrameBuilder::<pending_operation::Owned>::new();

        {
            let mut op_builder: pending_operation::Builder = msg_builder.get_builder_typed();
            op_builder.set_group_id(group_id);
            op_builder.set_operation_id(operation_id);

            let inner_op_builder = op_builder.init_operation();
            let new_entry_builder = inner_op_builder.init_entry_new();
            let mut entry_header_builder = new_entry_builder.init_entry_header();
            entry_header_builder.set_id(1234);
        }

        let frame_signer = MultihashFrameSigner::new_sha3256();
        msg_builder.as_owned_framed(frame_signer).unwrap()
    }
}
