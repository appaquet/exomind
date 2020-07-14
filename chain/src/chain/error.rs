#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Block related error: {0}")]
    Block(#[from] crate::block::Error),

    #[error("The store is in an unexpected state: {0}")]
    UnexpectedState(String),

    #[error("The store has an integrity problem: {0}")]
    Integrity(String),

    #[error("A segment has reached its full capacity")]
    SegmentFull,

    #[error("Error in capnp serialization: {0}")]
    Serialization(#[from] exocore_core::capnp::Error),

    #[error("An offset is out of the chain data: {0}")]
    OutOfBound(String),

    #[error("IO error of kind {0}: {1}")]
    IO(std::sync::Arc<std::io::Error>, String),

    #[cfg(feature = "directory-chain")]
    #[error("Error in directory chain store: {0}")]
    DirectoryError(#[from] super::directory::DirectoryError),

    #[error("Try to lock a mutex that was poisoned")]
    Poisoned,

    #[error("An error occurred: {0}")]
    Other(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Error::UnexpectedState(_) | Error::Integrity(_) | Error::IO(_, _) => true,
            _ => false,
        }
    }

    pub fn new_io<S: Into<String>>(io: std::io::Error, msg: S) -> Error {
        Error::IO(std::sync::Arc::new(io), msg.into())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}
