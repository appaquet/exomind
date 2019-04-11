use std::ops::RangeBounds;
use std::sync::Arc;
use std::vec::Vec;

use crate::chain;
use exocore_common::data_chain_capnp::{block, block_signature, pending_operation};
use exocore_common::security::signature::Signature;
use exocore_common::serialization::framed::{FrameBuilder, TypedFrame};
use exocore_common::serialization::protos::{GroupID, OperationID};
use exocore_common::serialization::{capnp, framed};

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
    fn put_operation(
        &mut self,
        operation: framed::OwnedTypedFrame<pending_operation::Owned>,
    ) -> Result<bool, Error>;

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
    pub operation_type: OperationType,
    pub frame: Arc<framed::OwnedTypedFrame<pending_operation::Owned>>,
}

pub struct StoredOperationsGroup {
    pub group_id: GroupID,
    pub operations: Vec<StoredOperation>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OperationType {
    Entry,
    BlockPropose,
    BlockSign,
    BlockRefuse,
    PendingIgnore,
}

///
/// Pending operation helper
///
pub struct PendingOperation;

impl PendingOperation {
    pub fn new_entry(
        operation_id: OperationID,
        node_id: &str,
        data: &[u8],
    ) -> FrameBuilder<pending_operation::Owned> {
        let mut frame_builder = FrameBuilder::new();

        let mut operation_builder: pending_operation::Builder = frame_builder.get_builder_typed();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(operation_id);
        operation_builder.set_node_id(node_id);

        let inner_operation_builder = operation_builder.init_operation();

        let mut new_entry_builder = inner_operation_builder.init_entry();
        new_entry_builder.set_data(data);

        frame_builder
    }

    pub fn new_block_proposal<B: chain::Block>(
        operation_id: OperationID,
        node_id: &str,
        block: &B,
    ) -> Result<FrameBuilder<pending_operation::Owned>, Error> {
        let mut frame_builder = FrameBuilder::new();

        let mut operation_builder: pending_operation::Builder = frame_builder.get_builder_typed();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(operation_id);
        operation_builder.set_node_id(node_id);

        let inner_operation_builder = operation_builder.init_operation();
        let mut new_block_builder = inner_operation_builder.init_block_propose();
        new_block_builder.set_block(&block.as_data_vec());

        Ok(frame_builder)
    }

    pub fn new_signature_for_block<B>(
        group_id: OperationID,
        operation_id: OperationID,
        node_id: &str,
        block: &B,
    ) -> Result<FrameBuilder<pending_operation::Owned>, Error>
    where
        B: TypedFrame<block::Owned>,
    {
        let mut frame_builder = FrameBuilder::new();

        let mut operation_builder: pending_operation::Builder = frame_builder.get_builder_typed();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(group_id);
        operation_builder.set_node_id(node_id);

        let inner_operation_builder = operation_builder.init_operation();
        let new_sig_builder = inner_operation_builder.init_block_sign();

        // TODO: Create signature for real
        let signature = Signature::empty();
        let _block_hash = block.signature_data().ok_or_else(|| {
            Error::Other("Tried to create a signature from a block without hash".to_string())
        })?;

        let mut sig_builder: block_signature::Builder = new_sig_builder.init_signature();
        sig_builder.set_node_id(node_id);
        sig_builder.set_node_signature(signature.get_bytes());

        Ok(frame_builder)
    }

    pub fn new_refusal(
        group_id: OperationID,
        operation_id: OperationID,
        node_id: &str,
    ) -> Result<FrameBuilder<pending_operation::Owned>, Error> {
        let mut frame_builder = FrameBuilder::new();

        let mut operation_builder: pending_operation::Builder = frame_builder.get_builder_typed();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(group_id);
        operation_builder.set_node_id(node_id);

        let inner_operation_builder = operation_builder.init_operation();
        let _new_refuse_builder = inner_operation_builder.init_block_refuse();

        Ok(frame_builder)
    }
}

///
/// Error related to the pending store
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error in message serialization")]
    Serialization(#[fail(cause)] framed::Error),
    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),
    #[fail(display = "Got an error: {}", _0)]
    Other(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        false
    }
}

impl From<framed::Error> for Error {
    fn from(err: framed::Error) -> Self {
        Error::Serialization(err)
    }
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}
