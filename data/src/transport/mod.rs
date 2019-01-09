use exocore_common::data_chain_capnp::block;
use exocore_common::node::Node;
use exocore_common::serialization::msg::{FramedOwnedMessage, MessageBuilder};
use tokio::prelude::*;

pub struct TransportContext {
    // TODO: Other nodes ? It's also in engine ...
}

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

// TODO: A-la-ampme
// TODO: from, to, entries, heads, hash
// TODO: entries could be full or just header too (so we don't send data)
// TODO: should send full if an object has been modified by us recently and we never sent to remote
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
