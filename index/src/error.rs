use std::time::Duration;

use exocore_core::capnp;

#[derive(Debug, Fail, Clone)]
pub enum Error {
    #[fail(display = "Query parsing error: {}", _0)]
    QueryParsing(String),

    #[cfg(feature = "local_store")]
    #[fail(display = "Error in Tantivy: {}", _0)]
    Tantivy(std::sync::Arc<tantivy::TantivyError>),

    #[cfg(feature = "local_store")]
    #[fail(display = "Error opening Tantivy directory: {:?}", _0)]
    TantivyOpenDirectoryError(std::sync::Arc<tantivy::directory::error::OpenDirectoryError>),

    #[cfg(feature = "local_store")]
    #[fail(display = "Error parsing Tantivy query: {:?}", _0)]
    TantitvyQueryParsing(std::sync::Arc<tantivy::query::QueryParserError>),

    #[cfg(feature = "local_store")]
    #[fail(display = "Chain engine error: {}", _0)]
    ChainEngine(#[fail(cause)] exocore_chain::engine::EngineError),

    #[fail(display = "Transport error: {}", _0)]
    Transport(#[fail(cause)] exocore_transport::Error),

    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),

    #[fail(display = "Protobuf error: {}", _0)]
    Proto(#[fail(cause)] exocore_core::protos::Error),

    #[fail(display = "A protobuf field was expected, but was empty: {}", _0)]
    ProtoFieldExpected(&'static str),

    #[fail(display = "IO error of kind {:?}: {}", _0, _1)]
    IO(std::io::ErrorKind, String),

    #[fail(display = "Error from remote index: {}", _0)]
    Remote(String),

    #[fail(display = "Timeout error: {:?} > {:?}", _0, _1)]
    Timeout(Duration, Duration),

    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,

    #[fail(display = "Query or mutation got cancelled")]
    Cancelled,

    #[fail(display = "Dropped or couldn't get locked")]
    Dropped,

    #[fail(display = "Other error occurred: {}", _0)]
    Other(String),

    #[fail(display = "A fatal error occurred: {}", _0)]
    Fatal(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Error::Fatal(_) | Error::Poisoned | Error::Dropped | Error::IO(_, _) => true,

            #[cfg(feature = "local_store")]
            Error::TantivyOpenDirectoryError(_) => true,

            #[cfg(feature = "local_store")]
            Error::ChainEngine(err) if err.is_fatal() => true,

            _ => false,
        }
    }
}

#[cfg(feature = "local_store")]
impl From<tantivy::TantivyError> for Error {
    fn from(err: tantivy::TantivyError) -> Self {
        Error::Tantivy(std::sync::Arc::new(err))
    }
}

#[cfg(feature = "local_store")]
impl From<tantivy::query::QueryParserError> for Error {
    fn from(err: tantivy::query::QueryParserError) -> Self {
        Error::TantitvyQueryParsing(std::sync::Arc::new(err))
    }
}

#[cfg(feature = "local_store")]
impl From<tantivy::directory::error::OpenDirectoryError> for Error {
    fn from(err: tantivy::directory::error::OpenDirectoryError) -> Self {
        Error::TantivyOpenDirectoryError(std::sync::Arc::new(err))
    }
}

impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Self {
        Error::Proto(err.into())
    }
}

impl From<prost::EncodeError> for Error {
    fn from(err: prost::EncodeError) -> Self {
        Error::Proto(err.into())
    }
}

impl From<exocore_core::protos::Error> for Error {
    fn from(err: exocore_core::protos::Error) -> Self {
        Error::Proto(err)
    }
}

#[cfg(feature = "local_store")]
impl From<exocore_chain::engine::EngineError> for Error {
    fn from(err: exocore_chain::engine::EngineError) -> Self {
        Error::ChainEngine(err)
    }
}

impl From<exocore_transport::Error> for Error {
    fn from(err: exocore_transport::Error) -> Self {
        Error::Transport(err)
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err.kind(), err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}
