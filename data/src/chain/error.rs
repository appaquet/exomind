use super::directory;
use crate::block;
use exocore_common::serialization::capnp;

#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Block related error: {}", _0)]
    Block(#[fail(cause)] block::Error),
    #[fail(display = "The store is in an unexpected state: {}", _0)]
    UnexpectedState(String),
    #[fail(display = "The store has an integrity problem: {}", _0)]
    Integrity(String),
    #[fail(display = "A segment has reached its full capacity")]
    SegmentFull,
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "An offset is out of the chain data: {}", _0)]
    OutOfBound(String),
    #[fail(display = "IO error of kind {:?}: {}", _0, _1)]
    IO(std::io::ErrorKind, String),
    #[fail(display = "Error in directory chain store: {:?}", _0)]
    DirectoryError(#[fail(cause)] directory::DirectoryError),
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Error::UnexpectedState(_) | Error::Integrity(_) | Error::IO(_, _) => true,
            _ => false,
        }
    }
}

impl From<block::Error> for Error {
    fn from(err: block::Error) -> Self {
        Error::Block(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

impl From<directory::DirectoryError> for Error {
    fn from(err: directory::DirectoryError) -> Self {
        Error::DirectoryError(err)
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}
