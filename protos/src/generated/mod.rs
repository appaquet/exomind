// Cap'n proto
#[path = "capn/common_capnp.rs"]
pub mod common_capnp;
#[path = "capn/data_chain_capnp.rs"]
pub mod data_chain_capnp;
#[path = "capn/data_transport_capnp.rs"]
pub mod data_transport_capnp;
#[path = "capn/store_transport_capnp.rs"]
pub mod store_transport_capnp;
pub mod types;
pub use types::*;

// Protobuf
#[path = "exocore.apps.rs"]
pub mod exocore_apps;
pub use self::exocore_apps as apps;
#[path = "exocore.core.rs"]
pub mod exocore_core;
pub use self::exocore_core as core;
#[path = "exocore.store.rs"]
pub mod exocore_store;
pub use self::exocore_store as store;
#[path = "exocore.test.rs"]
pub mod exocore_test;
pub use self::exocore_test as test;

pub const STORE_FDSET: &[u8] = include_bytes!("./exocore_store.fd");
pub const TEST_FDSET: &[u8] = include_bytes!("./exocore_test.fd");
