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

impl<'a> MessageType<'a> for self::data_chain_capnp::pending_operation::Owned {
    fn message_type() -> u16 {
        100
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::pending_operation_header::Owned {
    fn message_type() -> u16 {
        101
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_entry_new::Owned {
    fn message_type() -> u16 {
        110
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_block_propose::Owned {
    fn message_type() -> u16 {
        112
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_block_sign::Owned {
    fn message_type() -> u16 {
        113
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::operation_block_refuse::Owned {
    fn message_type() -> u16 {
        114
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::entry::Owned {
    fn message_type() -> u16 {
        120
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::entry_header::Owned {
    fn message_type() -> u16 {
        121
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::block::Owned {
    fn message_type() -> u16 {
        122
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::block_signatures::Owned {
    fn message_type() -> u16 {
        123
    }
}

impl<'a> MessageType<'a> for self::data_chain_capnp::block_signature::Owned {
    fn message_type() -> u16 {
        124
    }
}

///
///
impl<'a> MessageType<'a> for self::data_transport_capnp::envelope::Owned {
    fn message_type() -> u16 {
        200
    }
}

impl<'a> MessageType<'a> for self::data_transport_capnp::engine_message::Owned {
    fn message_type() -> u16 {
        201
    }
}

impl<'a> MessageType<'a> for self::data_transport_capnp::pending_sync_request::Owned {
    fn message_type() -> u16 {
        202
    }
}

impl<'a> MessageType<'a> for self::data_transport_capnp::pending_sync_range::Owned {
    fn message_type() -> u16 {
        203
    }
}
