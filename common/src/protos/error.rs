use protobuf::ProtobufError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Message type is not in registry")]
    NotInRegistry,
    #[fail(display = "Field doesn't exist")]
    NoSuchField,
    #[fail(display = "Invalid field type")]
    InvalidFieldType,
    #[fail(display = "Field type not supported")]
    NotSupported,
    #[fail(display = "Protobuf error: {}", _0)]
    Protobuf(ProtobufError),
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Self {
        Error::Protobuf(err)
    }
}
