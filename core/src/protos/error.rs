use protobuf::ProtobufError;
use std::sync::Arc;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Message type is not in registry: {0}")]
    NotInRegistry(String),

    #[error("Field doesn't exist")]
    NoSuchField,

    #[error("Invalid field type")]
    InvalidFieldType,

    #[error("Field type not supported")]
    NotSupported,

    #[error("Protobuf error: {0}")]
    StepanProtobuf(#[from] Arc<ProtobufError>),

    #[error("Protobuf encode error: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),

    #[error("Protobuf decode error: {0}")]
    ProstDecodeError(#[from] prost::DecodeError),
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Self {
        Error::StepanProtobuf(Arc::new(err))
    }
}
