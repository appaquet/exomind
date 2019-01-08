use exocore_common::data_chain_capnp::block;
use exocore_common::node::Node;
use exocore_common::serialization::msg::{FramedOwnedMessage, MessageBuilder};
use tokio::prelude::*;

pub struct TransportContext {
    // TODO: Other nodes ? It's also in engine ...
}

// TODO: A-la-tokio Framed
pub trait Transport:
    Stream<Item = MessageType, Error = Error>
    + Sink<SinkItem = MessageType, SinkError = Error>
    + Send
    + 'static
{
    fn send_message(node: &Node, message: MessageType);
}

pub struct OutMessage {
    to: Vec<Node>,
    message_type: MessageType,
    data: MessageBuilder<block::Owned>,
}

pub struct InMessage {
    from: Node,
    message_type: MessageType,
    data: FramedOwnedMessage,
}

pub enum MessageType {
    PendingSyncEntries, // from, to
    ChainSyncMeta,      // from, to
    ChainSyncData,      // from, to
}

#[derive(Debug)]
pub enum Error {
    Unknown,
}

#[cfg(test)]
mod test {
    #[test]
    fn test_transport() {}
}
