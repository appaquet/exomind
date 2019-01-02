#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate tempdir;

pub mod cell;
pub mod node;
pub mod range;
pub mod security;
pub mod serialization;
pub mod simple_store;

pub mod chain_block_capnp {
    include!(concat!(env!("OUT_DIR"), "/proto/chain_block_capnp.rs"));

    impl<'a> crate::serialization::msg::MessageType<'a> for crate::chain_block_capnp::block::Owned {
        fn message_type() -> u16 {
            1
        }
    }

    impl<'a> crate::serialization::msg::MessageType<'a>
        for crate::chain_block_capnp::block_entry::Owned
    {
        fn message_type() -> u16 {
            2
        }
    }

    impl<'a> crate::serialization::msg::MessageType<'a>
        for crate::chain_block_capnp::block_signatures::Owned
    {
        fn message_type() -> u16 {
            3
        }
    }

    impl<'a> crate::serialization::msg::MessageType<'a>
        for crate::chain_block_capnp::block_signature::Owned
    {
        fn message_type() -> u16 {
            4
        }
    }
}
