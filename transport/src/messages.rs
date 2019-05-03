use exocore_common::data_transport_capnp::envelope;
use exocore_common::node::Node;
use exocore_common::serialization::framed::{
    FrameBuilder, MessageType, OwnedTypedFrame, TypedFrame,
};

use crate::{Error, TransportLayer};

pub struct OutMessage {
    pub to: Vec<Node>,
    pub envelope: FrameBuilder<envelope::Owned>,
}

impl OutMessage {
    pub fn from_framed_message<'n, R, T>(
        local_node: &'n Node,
        to_nodes: Vec<Node>,
        frame: R,
    ) -> Result<OutMessage, Error>
    where
        R: TypedFrame<T>,
        T: for<'a> MessageType<'a>,
    {
        let mut envelope_frame_builder = FrameBuilder::new();
        let mut envelope_message_builder: envelope::Builder =
            envelope_frame_builder.get_builder_typed();

        envelope_message_builder.set_layer(TransportLayer::Data.into());
        envelope_message_builder.set_type(frame.message_type());
        envelope_message_builder.set_from_node(&local_node.id());
        envelope_message_builder.set_data(frame.frame_data());

        Ok(OutMessage {
            to: to_nodes,
            envelope: envelope_frame_builder,
        })
    }

    pub fn to_in_message(&self, from_node: Node) -> InMessage {
        InMessage {
            from: from_node,
            envelope: self.envelope.as_owned_unsigned_framed().unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct InMessage {
    pub from: Node,
    pub envelope: OwnedTypedFrame<envelope::Owned>,
}
