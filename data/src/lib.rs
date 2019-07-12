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
pub mod engine;
pub mod operation;
pub mod pending;

#[cfg(any(test, feature = "tests_utils"))]
pub mod tests_utils;

///
/// Re-exports
///
pub use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig};
pub use crate::engine::{
    ChainSyncConfig, CommitManagerConfig, Config as EngineConfig, Engine, EngineHandle,
    EngineOperationStatus, PendingSyncConfig,
};
pub use crate::pending::memory::MemoryPendingStore;
