///
/// Index related error
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error parsing schema: {}", _0)]
    Schema(String),
    #[fail(display = "Error in Tantivy: {}", _0)]
    Tantivy(tantivy::TantivyError),
    #[fail(display = "Error opening Tantivy directory: {:?}", _0)]
    TantivyOpenDirectoryError(tantivy::directory::error::OpenDirectoryError),
    #[fail(display = "Error parsing Tantivy query: {:?}", _0)]
    TantitvyQueryParsing(tantivy::query::QueryParserError),
    #[fail(display = "Serde json serialization/deserialization error: {}", _0)]
    SerdeJson(serde_json::Error),
    #[fail(display = "Data engine error: {}", _0)]
    DataEngine(#[fail(cause)] exocore_data::engine::Error),
    #[fail(display = "IO error of kind {:?}: {}", _0, _1)]
    IO(std::io::ErrorKind, String),
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "Other error occurred: {}", _0)]
    Other(String),
    #[fail(display = "A fatal error occurred: {}", _0)]
    Fatal(String),
}

impl From<tantivy::TantivyError> for Error {
    fn from(err: tantivy::TantivyError) -> Self {
        Error::Tantivy(err)
    }
}

impl From<tantivy::query::QueryParserError> for Error {
    fn from(err: tantivy::query::QueryParserError) -> Self {
        Error::TantitvyQueryParsing(err)
    }
}

impl From<tantivy::directory::error::OpenDirectoryError> for Error {
    fn from(err: tantivy::directory::error::OpenDirectoryError) -> Self {
        Error::TantivyOpenDirectoryError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}

impl From<exocore_data::engine::Error> for Error {
    fn from(err: exocore_data::engine::Error) -> Self {
        Error::DataEngine(err)
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
