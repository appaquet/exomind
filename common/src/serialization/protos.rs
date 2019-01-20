pub mod common_capnp {
    const MSG_TYPE_BASE: u16 = 0;

    //    use crate::serialization::msg::MessageType;

    include!(concat!(env!("OUT_DIR"), "/proto/common_capnp.rs"));

}

pub mod data_chain_capnp {
    const MSG_TYPE_BASE: u16 = 100;

    use crate::serialization::msg::MessageType;

    include!(concat!(env!("OUT_DIR"), "/proto/data_chain_capnp.rs"));

    impl<'a> MessageType<'a> for self::entry::Owned {
        fn message_type() -> u16 {
            MSG_TYPE_BASE //+ 0
        }
    }

    impl<'a> MessageType<'a> for self::entry_header::Owned {
        fn message_type() -> u16 {
            MSG_TYPE_BASE + 1
        }
    }

    impl<'a> MessageType<'a> for self::block::Owned {
        fn message_type() -> u16 {
            MSG_TYPE_BASE + 2
        }
    }

    impl<'a> MessageType<'a> for self::block_signatures::Owned {
        fn message_type() -> u16 {
            MSG_TYPE_BASE + 3
        }
    }

    impl<'a> MessageType<'a> for self::block_signature::Owned {
        fn message_type() -> u16 {
            MSG_TYPE_BASE + 4
        }
    }
}

pub mod data_transport_capnp {
    const MSG_TYPE_BASE: u16 = 200;

    use crate::serialization::msg::MessageType;

    include!(concat!(env!("OUT_DIR"), "/proto/data_transport_capnp.rs"));

    impl<'a> MessageType<'a> for self::envelope::Owned {
        fn message_type() -> u16 {
            MSG_TYPE_BASE //+ 0
        }
    }
}
