use crate::engine::{chain_sync, commit_manager, pending_sync};
use crate::{block, chain, operation, pending};
use exocore_core::capnp;
use exocore_transport::Error as TransportError;

/// Engine errors
#[derive(Clone, Debug, Fail)]
pub enum EngineError {
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
    #[fail(display = "Inner was dropped or couldn't get locked")]
    InnerUpgrade,
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "A fatal error occurred: {}", _0)]
    Fatal(String),
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl EngineError {
    pub fn is_fatal(&self) -> bool {
        match self {
            EngineError::ChainStore(inner) => inner.is_fatal(),
            EngineError::ChainSync(inner) => inner.is_fatal(),
            EngineError::MyNodeNotFound
            | EngineError::InnerUpgrade
            | EngineError::Poisoned
            | EngineError::Fatal(_) => true,
            _ => false,
        }
    }

    pub fn recover_non_fatal_error(self) -> Result<(), EngineError> {
        if !self.is_fatal() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl From<TransportError> for EngineError {
    fn from(err: TransportError) -> Self {
        EngineError::Transport(err)
    }
}

impl From<pending::Error> for EngineError {
    fn from(err: pending::Error) -> Self {
        EngineError::PendingStore(err)
    }
}

impl From<chain::Error> for EngineError {
    fn from(err: chain::Error) -> Self {
        EngineError::ChainStore(err)
    }
}

impl From<pending_sync::PendingSyncError> for EngineError {
    fn from(err: pending_sync::PendingSyncError) -> Self {
        EngineError::PendingSync(err)
    }
}

impl From<chain_sync::ChainSyncError> for EngineError {
    fn from(err: chain_sync::ChainSyncError) -> Self {
        EngineError::ChainSync(err)
    }
}

impl From<commit_manager::CommitManagerError> for EngineError {
    fn from(err: commit_manager::CommitManagerError) -> Self {
        EngineError::CommitManager(err)
    }
}

impl From<block::Error> for EngineError {
    fn from(err: block::Error) -> Self {
        EngineError::Block(err)
    }
}

impl From<operation::Error> for EngineError {
    fn from(err: operation::Error) -> Self {
        EngineError::Operation(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for EngineError {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        EngineError::Poisoned
    }
}

impl From<capnp::Error> for EngineError {
    fn from(err: capnp::Error) -> Self {
        EngineError::Serialization(err.kind, err.description)
    }
}

impl From<capnp::NotInSchema> for EngineError {
    fn from(err: capnp::NotInSchema) -> Self {
        EngineError::SerializationNotInSchema(err.0)
    }
}
