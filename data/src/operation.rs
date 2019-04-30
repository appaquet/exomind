use crate::block::Block;
use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::security::signature::Signature;
use exocore_common::serialization::framed::{FrameBuilder, FrameSigner, TypedFrame};
use exocore_common::serialization::protos::data_chain_capnp::{block, block_signature};
use exocore_common::serialization::{capnp, framed};

pub type GroupID = u64;
pub type OperationID = u64;

///
/// Wraps an operation that is stored either in the pending store, or in the
/// the chain.
///
pub trait Operation {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, Error>;

    fn as_entry_data(&self) -> Result<&[u8], Error> {
        let frame_reader = self.get_operation_reader()?;
        match frame_reader.get_operation().which()? {
            pending_operation::operation::Entry(entry) => Ok(entry?.get_data()?),
            _ => Err(Error::NotAnEntry),
        }
    }

    fn get_type(&self) -> Result<OperationType, Error> {
        let operation_reader = self.get_operation_reader()?;
        Ok(match operation_reader.get_operation().which()? {
            pending_operation::operation::Which::BlockSign(_) => OperationType::BlockSign,
            pending_operation::operation::Which::BlockPropose(_) => OperationType::BlockPropose,
            pending_operation::operation::Which::BlockRefuse(_) => OperationType::BlockRefuse,
            pending_operation::operation::Which::PendingIgnore(_) => OperationType::PendingIgnore,
            pending_operation::operation::Which::Entry(_) => OperationType::Entry,
        })
    }

    fn get_id(&self) -> Result<OperationID, Error> {
        let operation_reader = self.get_operation_reader()?;
        Ok(operation_reader.get_operation_id())
    }

    fn get_group_id(&self) -> Result<OperationID, Error> {
        let operation_reader = self.get_operation_reader()?;
        Ok(operation_reader.get_group_id())
    }
}

///
/// Types of operations
///
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
pub struct OperationBuilder {
    pub frame_builder: FrameBuilder<pending_operation::Owned>,
}

impl OperationBuilder {
    pub fn new_entry(operation_id: OperationID, node_id: &str, data: &[u8]) -> OperationBuilder {
        let mut frame_builder = FrameBuilder::new();

        let mut operation_builder: pending_operation::Builder = frame_builder.get_builder_typed();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(operation_id);
        operation_builder.set_node_id(node_id);

        let inner_operation_builder = operation_builder.init_operation();

        let mut new_entry_builder = inner_operation_builder.init_entry();
        new_entry_builder.set_data(data);

        OperationBuilder { frame_builder }
    }

    pub fn new_block_proposal<B: Block>(
        operation_id: OperationID,
        node_id: &str,
        block: &B,
    ) -> Result<OperationBuilder, Error> {
        let mut frame_builder = FrameBuilder::new();

        let mut operation_builder: pending_operation::Builder = frame_builder.get_builder_typed();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(operation_id);
        operation_builder.set_node_id(node_id);

        let inner_operation_builder = operation_builder.init_operation();
        let mut new_block_builder = inner_operation_builder.init_block_propose();
        new_block_builder.set_block(&block.as_data_vec());

        Ok(OperationBuilder { frame_builder })
    }

    pub fn new_signature_for_block<B: TypedFrame<block::Owned>>(
        group_id: OperationID,
        operation_id: OperationID,
        node_id: &str,
        block: &B,
    ) -> Result<OperationBuilder, Error> {
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

        Ok(OperationBuilder { frame_builder })
    }

    pub fn new_refusal(
        group_id: OperationID,
        operation_id: OperationID,
        node_id: &str,
    ) -> Result<OperationBuilder, Error> {
        let mut frame_builder = FrameBuilder::new();

        let mut operation_builder: pending_operation::Builder = frame_builder.get_builder_typed();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(group_id);
        operation_builder.set_node_id(node_id);

        let inner_operation_builder = operation_builder.init_operation();
        let _new_refuse_builder = inner_operation_builder.init_block_refuse();

        Ok(OperationBuilder { frame_builder })
    }

    pub fn sign_and_build<S: FrameSigner>(self, frame_signer: S) -> Result<NewOperation, Error> {
        let frame = self.frame_builder.as_owned_framed(frame_signer)?;
        Ok(NewOperation::from_frame(frame))
    }
}

///
/// Operation to be added or replaced in the store
///
#[derive(Clone)]
pub struct NewOperation {
    pub frame: framed::OwnedTypedFrame<pending_operation::Owned>,
}

impl NewOperation {
    pub fn from_frame(frame: framed::OwnedTypedFrame<pending_operation::Owned>) -> NewOperation {
        NewOperation { frame }
    }
}

impl crate::operation::Operation for NewOperation {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, Error> {
        Ok(self.frame.get_typed_reader()?)
    }
}

///
/// Operations related error
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "The operation is not any entry operation")]
    NotAnEntry,
    #[fail(display = "Error in message serialization")]
    Framing(#[fail(cause)] framed::Error),
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),
    #[fail(display = "Other operation error: {}", _0)]
    Other(String),
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
