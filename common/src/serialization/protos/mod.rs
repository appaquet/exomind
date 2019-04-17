// TODO: When Rust IntelliJ will support macro inclusion, we'll get back to the normal way
//pub mod common_capnp {
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

use crate::serialization::framed::MessageType;

pub mod common_capnp;
pub mod data_chain_capnp;
pub mod data_transport_capnp;

pub type GroupID = u64;
pub type OperationID = u64;

///
/// Messages related to the chain / operations storage
///
impl<'a> MessageType<'a> for self::data_chain_capnp::pending_operation::Owned {
    const MESSAGE_TYPE: u16 = 100;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::pending_operation_header::Owned {
    const MESSAGE_TYPE: u16 = 101;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_entry::Owned {
    const MESSAGE_TYPE: u16 = 110;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_block_propose::Owned {
    const MESSAGE_TYPE: u16 = 112;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_block_sign::Owned {
    const MESSAGE_TYPE: u16 = 113;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_block_refuse::Owned {
    const MESSAGE_TYPE: u16 = 114;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_pending_ignore::Owned {
    const MESSAGE_TYPE: u16 = 115;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::block::Owned {
    const MESSAGE_TYPE: u16 = 130;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::block_signatures::Owned {
    const MESSAGE_TYPE: u16 = 131;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::block_signature::Owned {
    const MESSAGE_TYPE: u16 = 132;
}

impl<'a> MessageType<'a> for self::data_chain_capnp::block_operation_header::Owned {
    const MESSAGE_TYPE: u16 = 133;
}

///
/// Messages related to transport / messaging between nodes
///
impl<'a> MessageType<'a> for self::data_transport_capnp::envelope::Owned {
    const MESSAGE_TYPE: u16 = 200;
}

impl<'a> MessageType<'a> for self::data_transport_capnp::pending_sync_request::Owned {
    const MESSAGE_TYPE: u16 = 201;
}

impl<'a> MessageType<'a> for self::data_transport_capnp::pending_sync_range::Owned {
    const MESSAGE_TYPE: u16 = 202;
}

impl<'a> MessageType<'a> for self::data_transport_capnp::chain_sync_request::Owned {
    const MESSAGE_TYPE: u16 = 203;
}

impl<'a> MessageType<'a> for self::data_transport_capnp::chain_sync_response::Owned {
    const MESSAGE_TYPE: u16 = 204;
}
