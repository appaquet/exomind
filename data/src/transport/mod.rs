use exocore_common::node::Node;
use exocore_common::serialization::msg::FramedOwnedMessage;
use tokio::prelude::*;

pub struct TransportContext {
    // TODO: Other nodes ? It's also in engine ...
}

// TODO: A-la-tokio Framed
pub trait Transport:
    Stream<Item = MessageType, Error = Error> + Sink<SinkItem = MessageType, SinkError = Error>
{
    fn send_message(node: &Node, message: MessageType);
}

pub struct OutMessage {
    to: Vec<Node>,
    message_type: MessageType,
    data: FramedOwnedMessage,
}

pub struct InMessage {
    from: Node,
    message_type: MessageType,
}

pub enum MessageType {
    PendingSyncEntries,
    ChainProposeBlock,
    ChainAcceptBlock,
    ChainGetSegmentsHeaders,
    ChainGetBlocksHeaders, // from, to
    ChainGetBlock,
}

pub enum Error {}

#[cfg(test)]
mod test {
    #[test]
    fn test_transport() {}
}
