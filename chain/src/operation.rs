use exocore_core::capnp;
use exocore_core::cell::{LocalNode, NodeId};
use exocore_core::crypto::hash::Sha3_256;
use exocore_core::crypto::signature::Signature;
use exocore_core::framing::{
    CapnpFrameBuilder, FrameBuilder, FrameReader, MultihashFrame, MultihashFrameBuilder,
    SizedFrame, SizedFrameBuilder, TypedCapnpFrame,
};
use exocore_core::protos::generated::data_chain_capnp::{block_signature, chain_operation};

use crate::block::Block;

pub type GroupId = u64;
pub type OperationId = u64;

pub type OperationFrame<I> =
    TypedCapnpFrame<MultihashFrame<Sha3_256, SizedFrame<I>>, chain_operation::Owned>;

pub type OperationFrameBuilder =
    SizedFrameBuilder<MultihashFrameBuilder<Sha3_256, CapnpFrameBuilder<chain_operation::Owned>>>;

/// Wraps an operation that is stored either in the pending store, or in the
/// the chain.
pub trait Operation {
    fn get_operation_reader(&self) -> Result<chain_operation::Reader, Error>;

    fn as_entry_data(&self) -> Result<&[u8], Error> {
        let frame_reader = self.get_operation_reader()?;
        match frame_reader.get_operation().which()? {
            chain_operation::operation::Entry(entry) => Ok(entry?.get_data()?),
            _ => Err(Error::NotAnEntry),
        }
    }

    fn get_type(&self) -> Result<OperationType, Error> {
        let operation_reader = self.get_operation_reader()?;
        Ok(match operation_reader.get_operation().which()? {
            chain_operation::operation::Which::BlockSign(_) => OperationType::BlockSign,
            chain_operation::operation::Which::BlockPropose(_) => OperationType::BlockPropose,
            chain_operation::operation::Which::BlockRefuse(_) => OperationType::BlockRefuse,
            chain_operation::operation::Which::Entry(_) => OperationType::Entry,
        })
    }

    fn get_id(&self) -> Result<OperationId, Error> {
        let operation_reader = self.get_operation_reader()?;
        Ok(operation_reader.get_operation_id())
    }

    fn get_group_id(&self) -> Result<OperationId, Error> {
        let operation_reader = self.get_operation_reader()?;
        Ok(operation_reader.get_group_id())
    }
}

/// Types of operations
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OperationType {
    Entry,
    BlockPropose,
    BlockSign,
    BlockRefuse,
}

/// Chain operation frame building helper
pub struct OperationBuilder {
    pub frame_builder: CapnpFrameBuilder<chain_operation::Owned>,
}

impl OperationBuilder {
    pub fn new_entry(operation_id: OperationId, node_id: &NodeId, data: &[u8]) -> OperationBuilder {
        let mut frame_builder = CapnpFrameBuilder::new();

        let mut operation_builder: chain_operation::Builder = frame_builder.get_builder();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(operation_id);
        operation_builder.set_node_id(&node_id.to_string());

        let inner_operation_builder = operation_builder.init_operation();

        let mut new_entry_builder = inner_operation_builder.init_entry();
        new_entry_builder.set_data(data);

        OperationBuilder { frame_builder }
    }

    pub fn new_block_proposal<B: Block>(
        operation_id: OperationId,
        node_id: &NodeId,
        block: &B,
    ) -> Result<OperationBuilder, Error> {
        let mut frame_builder = CapnpFrameBuilder::new();

        let mut operation_builder: chain_operation::Builder = frame_builder.get_builder();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(operation_id);
        operation_builder.set_node_id(&node_id.to_string());

        let inner_operation_builder = operation_builder.init_operation();
        let mut new_block_builder = inner_operation_builder.init_block_propose();
        new_block_builder.set_block(&block.as_data_vec());

        Ok(OperationBuilder { frame_builder })
    }

    pub fn new_signature_for_block<I: FrameReader>(
        group_id: OperationId,
        operation_id: OperationId,
        node_id: &NodeId,
        _header: &crate::block::BlockHeaderFrame<I>,
    ) -> Result<OperationBuilder, Error> {
        let mut frame_builder = CapnpFrameBuilder::new();

        let mut operation_builder: chain_operation::Builder = frame_builder.get_builder();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(group_id);
        operation_builder.set_node_id(&node_id.to_string());

        let inner_operation_builder = operation_builder.init_operation();
        let new_sig_builder = inner_operation_builder.init_block_sign();

        // TODO: Signature ticket: https://github.com/appaquet/exocore/issues/46
        //       Create signature for real
        let signature = Signature::empty();

        let mut sig_builder: block_signature::Builder = new_sig_builder.init_signature();
        sig_builder.set_node_id(&node_id.to_string());
        sig_builder.set_node_signature(signature.get_bytes());

        Ok(OperationBuilder { frame_builder })
    }

    pub fn new_refusal(
        group_id: OperationId,
        operation_id: OperationId,
        node_id: &NodeId,
    ) -> Result<OperationBuilder, Error> {
        let mut frame_builder = CapnpFrameBuilder::new();

        let mut operation_builder: chain_operation::Builder = frame_builder.get_builder();
        operation_builder.set_operation_id(operation_id);
        operation_builder.set_group_id(group_id);
        operation_builder.set_node_id(&node_id.to_string());

        let inner_operation_builder = operation_builder.init_operation();
        let _new_refuse_builder = inner_operation_builder.init_block_refuse();

        Ok(OperationBuilder { frame_builder })
    }

    pub fn sign_and_build(self, _local_node: &LocalNode) -> Result<NewOperation, Error> {
        // TODO: Signature ticket: https://github.com/appaquet/exocore/issues/46
        //       Include signature, not just hash.
        let msg_frame = self.frame_builder.as_bytes();
        let signed_frame_builder = MultihashFrameBuilder::<Sha3_256, _>::new(msg_frame);
        let sized_frame_builder = SizedFrameBuilder::new(signed_frame_builder);
        let final_frame = read_operation_frame(sized_frame_builder.as_bytes())?;

        Ok(NewOperation::from_frame(final_frame))
    }
}

pub fn read_operation_frame<I: FrameReader>(inner: I) -> Result<OperationFrame<I>, Error> {
    let sized_frame = SizedFrame::new(inner)?;
    let multihash_frame = MultihashFrame::<Sha3_256, _>::new(sized_frame)?;
    let frame = TypedCapnpFrame::new(multihash_frame)?;
    Ok(frame)
}

/// Operation to be added or replaced in the store
#[derive(Clone)]
pub struct NewOperation {
    pub frame: OperationFrame<Vec<u8>>,
}

impl NewOperation {
    pub fn from_frame(frame: OperationFrame<Vec<u8>>) -> NewOperation {
        NewOperation { frame }
    }
}

impl crate::operation::Operation for NewOperation {
    fn get_operation_reader(&self) -> Result<chain_operation::Reader, Error> {
        Ok(self.frame.get_reader()?)
    }
}

/// Operations related error
#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("The operation is not any entry operation")]
    NotAnEntry,

    #[error("Framing error: {0}")]
    Framing(#[from] exocore_core::framing::Error),

    #[error("Error in capnp serialization: {0}")]
    Serialization(#[from] capnp::Error),

    #[error("Field is not in capnp schema: code={0}")]
    SerializationNotInSchema(u16),

    #[error("Other operation error: {0}")]
    Other(String),
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}
