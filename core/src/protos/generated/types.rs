/// Trait that needs to have an impl for each capnp generated message struct.
/// Used to identify a unique type id for each message and annotate each framed
/// message.
pub trait MessageType<'a>: capnp::traits::Owned<'a> {
    const MESSAGE_TYPE: u16;
}

/// Common messages
impl<'a> MessageType<'a> for super::common_capnp::envelope::Owned {
    const MESSAGE_TYPE: u16 = 0;
}

/// Messages related to the chain / operations storage
impl<'a> MessageType<'a> for super::data_chain_capnp::chain_operation::Owned {
    const MESSAGE_TYPE: u16 = 100;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::chain_operation_header::Owned {
    const MESSAGE_TYPE: u16 = 101;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::operation_entry::Owned {
    const MESSAGE_TYPE: u16 = 110;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::operation_block_propose::Owned {
    const MESSAGE_TYPE: u16 = 112;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::operation_block_sign::Owned {
    const MESSAGE_TYPE: u16 = 113;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::operation_block_refuse::Owned {
    const MESSAGE_TYPE: u16 = 114;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::block_header::Owned {
    const MESSAGE_TYPE: u16 = 130;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::block_signatures::Owned {
    const MESSAGE_TYPE: u16 = 131;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::block_signature::Owned {
    const MESSAGE_TYPE: u16 = 132;
}

impl<'a> MessageType<'a> for super::data_chain_capnp::block_operation_header::Owned {
    const MESSAGE_TYPE: u16 = 133;
}

/// Messages related to transport / messaging between nodes on chain layer
impl<'a> MessageType<'a> for super::data_transport_capnp::pending_sync_request::Owned {
    const MESSAGE_TYPE: u16 = 200;
}

impl<'a> MessageType<'a> for super::data_transport_capnp::pending_sync_range::Owned {
    const MESSAGE_TYPE: u16 = 201;
}

impl<'a> MessageType<'a> for super::data_transport_capnp::chain_sync_request::Owned {
    const MESSAGE_TYPE: u16 = 202;
}

impl<'a> MessageType<'a> for super::data_transport_capnp::chain_sync_response::Owned {
    const MESSAGE_TYPE: u16 = 203;
}

/// Messages related to transport / messaging between nodes on store layer
impl<'a> MessageType<'a> for super::store_transport_capnp::query_request::Owned {
    const MESSAGE_TYPE: u16 = 300;
}

impl<'a> MessageType<'a> for super::store_transport_capnp::query_response::Owned {
    const MESSAGE_TYPE: u16 = 301;
}

impl<'a> MessageType<'a> for super::store_transport_capnp::mutation_request::Owned {
    const MESSAGE_TYPE: u16 = 302;
}

impl<'a> MessageType<'a> for super::store_transport_capnp::mutation_response::Owned {
    const MESSAGE_TYPE: u16 = 303;
}

impl<'a> MessageType<'a> for super::store_transport_capnp::watched_query_request::Owned {
    const MESSAGE_TYPE: u16 = 304;
}

impl<'a> MessageType<'a> for super::store_transport_capnp::watched_query_response::Owned {
    const MESSAGE_TYPE: u16 = 305;
}

impl<'a> MessageType<'a> for super::store_transport_capnp::unwatch_query_request::Owned {
    const MESSAGE_TYPE: u16 = 306;
}
