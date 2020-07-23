use super::reflect::FieldId;
use std::sync::Arc;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Message type is not in registry: {0}")]
    NotInRegistry(String),

    #[error("Field doesn't exist: {0}")]
    NoSuchField(FieldId),

    #[error("Invalid field type")]
    InvalidFieldType,

    #[error("Field type not supported")]
    NotSupported,

    #[error("Protobuf error: {0}")]
    StepanProtobuf(#[source] Arc<protobuf::ProtobufError>),

    #[error("Protobuf encode error: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),

    #[error("Protobuf decode error: {0}")]
    ProstDecodeError(#[from] prost::DecodeError),
}

impl From<protobuf::ProtobufError> for Error {
    fn from(err: protobuf::ProtobufError) -> Self {
        Error::StepanProtobuf(Arc::new(err))
    }
}
