use prost::{DecodeError, EncodeError};
use protobuf::ProtobufError;
use std::sync::Arc;

#[derive(Debug, Clone, Fail)]
pub enum Error {
    #[fail(display = "Message type is not in registry: {}", _0)]
    NotInRegistry(String),
    #[fail(display = "Field doesn't exist")]
    NoSuchField,
    #[fail(display = "Invalid field type")]
    InvalidFieldType,
    #[fail(display = "Field type not supported")]
    NotSupported,
    #[fail(display = "Protobuf error: {}", _0)]
    StepanProtobuf(Arc<ProtobufError>),
    #[fail(display = "Protobuf encode error: {}", _0)]
    ProstEncodeError(prost::EncodeError),
    #[fail(display = "Protobuf decode error: {}", _0)]
    ProstDecodeError(prost::DecodeError),
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Self {
        Error::StepanProtobuf(Arc::new(err))
    }
}

impl From<EncodeError> for Error {
    fn from(err: EncodeError) -> Self {
        Error::ProstEncodeError(err)
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self {
        Error::ProstDecodeError(err)
    }
}
