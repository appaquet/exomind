#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate tempdir;

///
/// Modules
///
pub mod block;
pub mod chain;
pub mod engine;
pub mod operation;
pub mod pending;

///
/// Re-exports
///
pub use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig};
pub use crate::engine::{
    ChainSyncConfig, CommitManagerConfig, Config as EngineConfig, Engine, EngineOperationStatus,
    PendingSyncConfig,
};
pub use crate::pending::memory::MemoryPendingStore;
