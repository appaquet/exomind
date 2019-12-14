use exocore_common::capnp;
use exocore_schema::schema::{SchemaFieldId, SchemaRecordId};
use std::sync::Arc;
use std::time::Duration;

///
/// Index related error
///
#[derive(Debug, Fail, Clone)]
pub enum Error {
    #[fail(display = "Error parsing schema: {}", _0)]
    Schema(String),

    #[fail(display = "Data integrity error: {}", _0)]
    DataIntegrity(String),

    #[fail(display = "Field id {} of record id {} didn't have a value", _0, _1)]
    FieldEmptyValue(SchemaRecordId, SchemaFieldId),

    #[fail(display = "Record field invalid type error: {}", _0)]
    FieldInvalidType(String),

    #[fail(display = "Field named {} was not in schema", _0)]
    NamedFieldNotInSchema(String),

    #[fail(display = "Query parsing error: {}", _0)]
    QueryParsing(String),

    #[cfg(feature = "local_store")]
    #[fail(display = "Error in Tantivy: {}", _0)]
    Tantivy(Arc<tantivy::TantivyError>),

    #[cfg(feature = "local_store")]
    #[fail(display = "Error opening Tantivy directory: {:?}", _0)]
    TantivyOpenDirectoryError(Arc<tantivy::directory::error::OpenDirectoryError>),

    #[cfg(feature = "local_store")]
    #[fail(display = "Error parsing Tantivy query: {:?}", _0)]
    TantitvyQueryParsing(Arc<tantivy::query::QueryParserError>),

    #[fail(display = "Serde json serialization/deserialization error: {}", _0)]
    SerdeJson(Arc<serde_json::Error>),

    #[cfg(feature = "local_store")]
    #[fail(display = "Data engine error: {}", _0)]
    DataEngine(#[fail(cause)] exocore_data::engine::Error),

    #[fail(display = "Transport error: {}", _0)]
    Transport(#[fail(cause)] exocore_transport::Error),

    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),

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
            Error::DataEngine(err) if err.is_fatal() => true,

            _ => false,
        }
    }
}

#[cfg(feature = "local_store")]
impl From<tantivy::TantivyError> for Error {
    fn from(err: tantivy::TantivyError) -> Self {
        Error::Tantivy(Arc::new(err))
    }
}

#[cfg(feature = "local_store")]
impl From<tantivy::query::QueryParserError> for Error {
    fn from(err: tantivy::query::QueryParserError) -> Self {
        Error::TantitvyQueryParsing(Arc::new(err))
    }
}

#[cfg(feature = "local_store")]
impl From<tantivy::directory::error::OpenDirectoryError> for Error {
    fn from(err: tantivy::directory::error::OpenDirectoryError) -> Self {
        Error::TantivyOpenDirectoryError(Arc::new(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(Arc::new(err))
    }
}

#[cfg(feature = "local_store")]
impl From<exocore_data::engine::Error> for Error {
    fn from(err: exocore_data::engine::Error) -> Self {
        Error::DataEngine(err)
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
