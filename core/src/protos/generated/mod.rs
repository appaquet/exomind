pub mod common_capnp;
pub mod data_chain_capnp;
pub mod data_transport_capnp;
pub mod index_transport_capnp;

pub mod types;
pub use types::*;

// Protobuf
pub mod exocore_index;
pub mod exocore_test;
pub const INDEX_FDSET: &[u8] = include_bytes!("./exocore_index.fd");
pub const TEST_FDSET: &[u8] = include_bytes!("./exocore_test.fd");
