use tokio::prelude::*;

use exocore_common::data_transport_capnp::envelope;
use exocore_common::node::Node;
use exocore_common::serialization::framed::{FrameBuilder, OwnedTypedFrame};

pub mod mock;

pub trait Transport: Future<Item = (), Error = Error> + Send {
    type Sink: Sink<SinkItem = OutMessage, SinkError = Error> + Send + 'static;
    type Stream: Stream<Item = InMessage, Error = Error> + Send + 'static;

    fn get_sink(&mut self) -> Self::Sink;
    fn get_stream(&mut self) -> Self::Stream;
}

pub struct OutMessage {
    to: Vec<Node>,
    data: FrameBuilder<envelope::Owned>,
}
impl OutMessage {
    fn to_in_message(&self, from_node: Node) -> InMessage {
        InMessage {
            from: from_node,
            data: self.data.as_owned_unsigned_framed().unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct InMessage {
    pub from: Node,
    pub data: OwnedTypedFrame<envelope::Owned>,
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}
