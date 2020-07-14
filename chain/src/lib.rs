#![deny(bare_trait_objects)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(any(test, feature = "tests-utils"))]
#[macro_use]
extern crate anyhow;

/// Modules
pub mod block;
pub mod chain;
#[cfg(feature = "engine")]
pub mod engine;
pub mod operation;
pub mod pending;

#[cfg(feature = "tests-utils")]
pub mod tests_utils;

/// Re-exports
#[cfg(feature = "directory-chain")]
pub use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig};
#[cfg(feature = "engine")]
pub use crate::engine::{
    ChainSyncConfig, CommitManagerConfig, Engine, EngineConfig, EngineHandle,
    EngineOperationStatus, PendingSyncConfig,
};
#[cfg(feature = "memory-pending")]
pub use crate::pending::memory::MemoryPendingStore;
