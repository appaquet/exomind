pub mod data_chain_capnp {
    include!(concat!(env!("OUT_DIR"), "/proto/data_chain_capnp.rs"));

    impl<'a> crate::serialization::msg::MessageType<'a> for crate::data_chain_capnp::block::Owned {
        fn message_type() -> u16 {
            1
        }
    }

    impl<'a> crate::serialization::msg::MessageType<'a>
        for crate::data_chain_capnp::block_entry::Owned
    {
        fn message_type() -> u16 {
            2
        }
    }

    impl<'a> crate::serialization::msg::MessageType<'a>
        for crate::data_chain_capnp::block_signatures::Owned
    {
        fn message_type() -> u16 {
            3
        }
    }

    impl<'a> crate::serialization::msg::MessageType<'a>
        for crate::data_chain_capnp::block_signature::Owned
    {
        fn message_type() -> u16 {
            4
        }
    }
}

pub mod data_transport_capnp {
    include!(concat!(env!("OUT_DIR"), "/proto/data_transport_capnp.rs"));
}
