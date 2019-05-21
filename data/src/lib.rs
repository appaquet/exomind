#![deny(bare_trait_objects)]

extern crate byteorder;
extern crate exocore_common;
extern crate exocore_transport;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate futures;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate memmap;
#[cfg(test)]
extern crate tempdir;
extern crate tokio;
extern crate tokio_io;

///
/// Re-exports
///
pub use crate::chain::directory::{DirectoryChainStore, DirectoryChainStoreConfig};
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
