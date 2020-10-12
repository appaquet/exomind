// Cap'n proto
pub mod common_capnp;
pub mod data_chain_capnp;
pub mod data_transport_capnp;
pub mod store_transport_capnp;
pub mod types;
pub use types::*;

// Protobuf
pub mod exocore_apps;
pub use self::exocore_apps as apps;
pub mod exocore_core;
pub use self::exocore_core as core;
pub mod exocore_store;
pub use self::exocore_store as store;
pub mod exocore_test;
pub use self::exocore_test as test;

pub const STORE_FDSET: &[u8] = include_bytes!("./exocore_store.fd");
pub const TEST_FDSET: &[u8] = include_bytes!("./exocore_test.fd");
