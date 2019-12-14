use crate::engine::{chain_sync, commit_manager, pending_sync};
use crate::{block, chain, operation, pending};
use exocore_common::capnp;
use exocore_transport::Error as TransportError;
use futures01::sync::mpsc;

///
/// Engine errors
///
#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Error in transport: {:?}", _0)]
    Transport(#[fail(cause)] TransportError),
    #[fail(display = "Error in pending store: {:?}", _0)]
    PendingStore(#[fail(cause)] pending::Error),
    #[fail(display = "Error in chain store: {:?}", _0)]
    ChainStore(#[fail(cause)] chain::Error),
    #[fail(display = "Error in pending synchronizer: {:?}", _0)]
    PendingSync(#[fail(cause)] pending_sync::PendingSyncError),
    #[fail(display = "Error in chain synchronizer: {:?}", _0)]
    ChainSync(#[fail(cause)] chain_sync::ChainSyncError),
    #[fail(display = "Error in commit manager: {:?}", _0)]
    CommitManager(#[fail(cause)] commit_manager::CommitManagerError),
    #[fail(display = "Got a block related error: {:?}", _0)]
    Block(#[fail(cause)] block::Error),
    #[fail(display = "Got an operation related error: {:?}", _0)]
    Operation(#[fail(cause)] operation::Error),
    #[fail(display = "Chain is not initialized")]
    UninitializedChain,
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),
    #[fail(display = "Item not found: {}", _0)]
    NotFound(String),
    #[fail(display = "Local node not found in nodes list")]
    MyNodeNotFound,
    #[fail(display = "Couldn't send message to a mpsc channel because its receiver was dropped")]
    MpscSendDropped,
    #[fail(display = "Inner was dropped or couldn't get locked")]
    InnerUpgrade,
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "A fatal error occurred: {}", _0)]
    Fatal(String),
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Error::ChainStore(inner) => inner.is_fatal(),
            Error::ChainSync(inner) => inner.is_fatal(),
            Error::MyNodeNotFound
            | Error::MpscSendDropped
            | Error::InnerUpgrade
            | Error::Poisoned
            | Error::Fatal(_) => true,
            _ => false,
        }
    }

    pub fn recover_non_fatal_error(self) -> Result<(), Error> {
        if !self.is_fatal() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl From<TransportError> for Error {
    fn from(err: TransportError) -> Self {
        Error::Transport(err)
    }
}

impl From<pending::Error> for Error {
    fn from(err: pending::Error) -> Self {
        Error::PendingStore(err)
    }
}

impl From<chain::Error> for Error {
    fn from(err: chain::Error) -> Self {
        Error::ChainStore(err)
    }
}

impl From<pending_sync::PendingSyncError> for Error {
    fn from(err: pending_sync::PendingSyncError) -> Self {
        Error::PendingSync(err)
    }
}

impl From<chain_sync::ChainSyncError> for Error {
    fn from(err: chain_sync::ChainSyncError) -> Self {
        Error::ChainSync(err)
    }
}

impl From<commit_manager::CommitManagerError> for Error {
    fn from(err: commit_manager::CommitManagerError) -> Self {
        Error::CommitManager(err)
    }
}

impl From<block::Error> for Error {
    fn from(err: block::Error) -> Self {
        Error::Block(err)
    }
}

impl From<operation::Error> for Error {
    fn from(err: operation::Error) -> Self {
        Error::Operation(err)
    }
}

impl<T> From<mpsc::SendError<T>> for Error {
    fn from(_err: mpsc::SendError<T>) -> Self {
        Error::MpscSendDropped
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}
