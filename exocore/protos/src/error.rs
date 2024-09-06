use super::reflect::FieldId;

#[derive(Debug, thiserror::Error)]
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
    StepanProtobuf(#[from] protobuf::Error),

    #[error("Protobuf encode error: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),

    #[error("Protobuf decode error: {0}")]
    ProstDecodeError(#[from] prost::DecodeError),

    #[error("Error: {0}")]
    Other(anyhow::Error),
}
