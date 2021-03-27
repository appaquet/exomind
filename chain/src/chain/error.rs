use crate::block::BlockOffset;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Block related error: {0}")]
    Block(#[from] crate::block::Error),

    #[error("The store is in an unexpected state: {0}")]
    UnexpectedState(#[source] anyhow::Error),

    #[error("The store has an integrity problem: {0}")]
    Integrity(#[source] anyhow::Error),

    #[error("Tried to write a block at offset {offset}, but next offset was {expected_offset}")]
    InvalidNextBlock {
        offset: BlockOffset,
        expected_offset: BlockOffset,
    },

    #[error("Error in capnp serialization: {0}")]
    Serialization(#[from] exocore_protos::capnp::Error),

    #[error("An offset is out of the chain data: {0}")]
    OutOfBound(#[source] anyhow::Error),

    #[error("IO error of kind {0}: {1}")]
    Io(std::io::Error, String),

    #[cfg(feature = "directory-chain")]
    #[error("Error in directory chain store: {0}")]
    DirectoryError(#[from] super::directory::DirectoryError),

    #[error("Try to lock a mutex that was poisoned")]
    Poisoned,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Error::UnexpectedState(_) | Error::Integrity(_) | Error::Io(_, _)
        )
    }

    pub fn new_io<S: Into<String>>(io: std::io::Error, msg: S) -> Error {
        Error::Io(io, msg.into())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}
