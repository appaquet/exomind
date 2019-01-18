use exocore_common::data_transport_capnp::envelope;
use exocore_common::node::Node;
use exocore_common::serialization::msg::{FramedOwnedTypedMessage, MessageBuilder};
use tokio::prelude::*;

pub struct TransportContext {
    // TODO: Other nodes ? It's also in engine ...
}

pub trait Transport:
    Stream<Item = InMessage, Error = Error>
    + Sink<SinkItem = OutMessage, SinkError = Error>
    + Send
    + 'static
{
    fn send_message(node: &Node, message: MessageBuilder<envelope::Owned>);
}

pub struct OutMessage {
    to: Vec<Node>,
    data: MessageBuilder<envelope::Owned>,
}

pub struct InMessage {
    from: Node,
    data: FramedOwnedTypedMessage<envelope::Owned>,
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
