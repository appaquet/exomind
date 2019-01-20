use tokio::prelude::*;

use exocore_common::data_transport_capnp::envelope;
use exocore_common::node::Node;
use exocore_common::serialization::msg::{FramedOwnedTypedMessage, MessageBuilder};

pub mod mock;

pub trait Transport: Future<Item = (), Error = Error> + Send {
    type Sink: Sink<SinkItem = OutMessage, SinkError = Error> + Send + 'static;
    type Stream: Stream<Item = InMessage, Error = Error> + Send + 'static;

    fn get_sink(&mut self) -> Self::Sink;
    fn get_stream(&mut self) -> Self::Stream;
}

pub struct OutMessage {
    to: Vec<Node>,
    data: MessageBuilder<envelope::Owned>,
}
impl OutMessage {
    fn to_in_message(&self, from_node: Node) -> InMessage {
        InMessage {
            from: from_node,
            data: self.data.as_owned_framed().unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct InMessage {
    from: Node,
    data: FramedOwnedTypedMessage<envelope::Owned>,
}

#[derive(Debug)]
pub enum Error {
    Unknown,
}
