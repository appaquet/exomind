// TODO: When Rust IntelliJ will support macro inclusion, we'll get back to the
// normal way pub mod common_capnp {
//    include!(concat!(env!("OUT_DIR"), "/proto/common_capnp.rs"));
//}
//
//pub mod data_chain_capnp {
//    include!(concat!(env!("OUT_DIR"), "/proto/data_chain_capnp.rs"));
//}
//
//pub mod data_transport_capnp {
//    include!(concat!(env!("OUT_DIR"), "/proto/data_transport_capnp.rs"));
//}

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
