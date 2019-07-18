#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;

///
/// Modules
///
pub mod block;
pub mod chain;
#[cfg(feature = "engine")]
pub mod engine;
pub mod operation;
pub mod pending;

#[cfg(feature = "tests_utils")]
pub mod tests_utils;

///
/// Re-exports
///
#[cfg(feature = "directory_chain")]
pub use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig};
#[cfg(feature = "engine")]
pub use crate::engine::{
    ChainSyncConfig, CommitManagerConfig, Config as EngineConfig, Engine, EngineHandle,
    EngineOperationStatus, PendingSyncConfig,
};
#[cfg(feature = "memory_pending")]
pub use crate::pending::memory::MemoryPendingStore;
