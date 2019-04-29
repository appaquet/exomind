use std::sync::Arc;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::serialization::framed::TypedFrame;
use exocore_common::serialization::{capnp, framed};

pub trait Operation {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, framed::Error>;

    fn as_entry_data(&self) -> Result<&[u8], Error> {
        let frame_reader: pending_operation::Reader = self.get_operation_reader()?;
        match frame_reader.get_operation().which()? {
            pending_operation::operation::Entry(entry) => Ok(entry?.get_data()?),
            _ => Err(Error::NotAnEntry),
        }
    }
}

impl crate::operation::Operation for Arc<framed::OwnedTypedFrame<pending_operation::Owned>> {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, framed::Error> {
        self.get_typed_reader()
    }
}

impl crate::operation::Operation for framed::OwnedTypedFrame<pending_operation::Owned> {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, framed::Error> {
        self.get_typed_reader()
    }
}

impl<'a> crate::operation::Operation for framed::TypedSliceFrame<'a, pending_operation::Owned> {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, framed::Error> {
        self.get_typed_reader()
    }
}

///
/// Types of operations
///
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Type {
    Entry,
    BlockPropose,
    BlockSign,
    BlockRefuse,
    PendingIgnore,
}

///
/// Error related to operations
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
