/// Engine errors
#[derive(Clone, Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Error in transport: {0:?}")]
    Transport(#[from] exocore_transport::Error),

    #[error("Error in pending store: {0:?}")]
    PendingStore(#[from] crate::pending::Error),

    #[error("Error in chain store: {0:?}")]
    ChainStore(#[from] crate::chain::Error),

    #[error("Error in pending synchronizer: {0:?}")]
    PendingSync(#[from] crate::engine::pending_sync::PendingSyncError),

    #[error("Error in chain synchronizer: {0:?}")]
    ChainSync(#[from] crate::engine::chain_sync::ChainSyncError),

    #[error("Error in commit manager: {0:?}")]
    CommitManager(#[from] crate::engine::commit_manager::CommitManagerError),

    #[error("Got a block related error: {0:?}")]
    Block(#[from] crate::block::Error),

    #[error("Got an operation related error: {0:?}")]
    Operation(#[from] crate::operation::Error),

    #[error("Chain is not initialized")]
    UninitializedChain,

    #[error("Error in capnp serialization: {0}")]
    Serialization(#[from] exocore_core::capnp::Error),

    #[error("Field is not in capnp schema: code={0}")]
    SerializationNotInSchema(u16),

    #[error("Local node not found in nodes list")]
    MyNodeNotFound,

    #[error("Inner was dropped or couldn't get locked")]
    InnerUpgrade,

    #[error("Try to lock a mutex that was poisoned")]
    Poisoned,

    #[error("A fatal error occurred: {0}")]
    Fatal(String),

    #[error("An error occurred: {0}")]
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

impl<T> From<std::sync::PoisonError<T>> for EngineError {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        EngineError::Poisoned
    }
}

impl From<exocore_core::capnp::NotInSchema> for EngineError {
    fn from(err: exocore_core::capnp::NotInSchema) -> Self {
        EngineError::SerializationNotInSchema(err.0)
    }
}
