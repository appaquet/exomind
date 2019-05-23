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
/// Re-exports
///
pub use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig};
pub use crate::engine::{ChainSyncConfig, CommitManagerConfig, PendingSyncConfig};
pub use crate::engine::{Config as EngineConfig, Engine, EngineOperationStatus};
pub use crate::pending::memory::MemoryPendingStore;

///
/// Modules
///
pub mod block;
pub mod chain;
pub mod engine;
pub mod operation;
pub mod pending;
