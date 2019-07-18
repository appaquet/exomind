use exocore_common::node::Node;
use exocore_common::protos::common_capnp::envelope;
use exocore_common::protos::MessageType;

use crate::{Error, TransportLayer};
use exocore_common::cell::Cell;
use exocore_common::framing::{CapnpFrameBuilder, FrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_common::time::ConsistentTimestamp;

/// Message to be sent to one or more other nodes
pub struct OutMessage {
    pub to: Vec<Node>,
    pub envelope_builder: CapnpFrameBuilder<envelope::Owned>,
}

impl OutMessage {
    pub fn from_framed_message<T>(
        cell: &Cell,
        to_layer: TransportLayer,
        frame: CapnpFrameBuilder<T>,
    ) -> Result<OutMessage, Error>
    where
        T: for<'a> MessageType<'a>,
    {
        let mut envelope_frame_builder = CapnpFrameBuilder::<envelope::Owned>::new();
        let mut envelope_message_builder = envelope_frame_builder.get_builder();
        envelope_message_builder.set_layer(to_layer.to_code());
        envelope_message_builder.set_type(T::MESSAGE_TYPE);
        envelope_message_builder.set_cell_id(cell.id().as_bytes());
        envelope_message_builder.set_from_node_id(&cell.local_node().id().to_str());
        envelope_message_builder.set_data(&frame.as_bytes());

        Ok(OutMessage {
            to: vec![],
            envelope_builder: envelope_frame_builder,
        })
    }

    pub fn with_to_node(mut self, to_node: Node) -> Self {
        self.to = vec![to_node];
        self
    }

    pub fn with_to_nodes(mut self, nodes: Vec<Node>) -> Self {
        self.to = nodes;
        self
    }

    pub fn with_follow_id(mut self, follow_id: ConsistentTimestamp) -> Self {
        let mut envelope_message_builder = self.envelope_builder.get_builder();
        envelope_message_builder.set_follow_id(follow_id);

        self
    }
}

/// Message receive from another node
#[derive(Clone)]
pub struct InMessage {
    pub from: Node,
    pub layer: TransportLayer,
    pub follow_id: Option<ConsistentTimestamp>,
    pub message_type: u16,
    pub envelope: TypedCapnpFrame<Vec<u8>, envelope::Owned>,
}

impl InMessage {
    pub fn from_node_and_frame<I: FrameReader<OwnedType = Vec<u8>>>(
        from: Node,
        envelope: TypedCapnpFrame<I, envelope::Owned>,
    ) -> Result<InMessage, Error> {
        let envelope_reader = envelope.get_reader()?;
        let follow_id = if envelope_reader.get_follow_id() != 0 {
            Some(envelope_reader.get_follow_id())
        } else {
            None
        };

        let layer_id = envelope_reader.get_layer();
        let layer = TransportLayer::from_code(layer_id).ok_or_else(|| {
            Error::Other(format!("Got message with invalid layer id: {}", layer_id))
        })?;

        let message_type = envelope_reader.get_type();

        Ok(InMessage {
            from,
            layer,
            follow_id,
            message_type,
            envelope: envelope.to_owned(),
        })
    }

    pub fn get_data(&self) -> Result<&[u8], Error> {
        let reader = self.envelope.get_reader()?;
        let data = reader.get_data()?;
        Ok(data)
    }

    pub fn get_data_as_framed_message<'d, T>(
        &'d self,
    ) -> Result<TypedCapnpFrame<&'d [u8], T>, Error>
    where
        T: for<'a> MessageType<'a>,
    {
        let reader = self.envelope.get_reader()?;
        let data = reader.get_data()?;
        let frame = TypedCapnpFrame::new(data)?;
        Ok(frame)
    }

    pub fn to_response_message<T>(
        &self,
        cell: &Cell,
        frame: CapnpFrameBuilder<T>,
    ) -> Result<OutMessage, Error>
    where
        T: for<'a> MessageType<'a>,
    {
        let out_message = OutMessage::from_framed_message(cell, self.layer, frame)?
            .with_to_node(self.from.clone());

        let follow_id = self.follow_id.ok_or_else(|| {
            Error::Other(format!(
                "Tried to respond to an InMessage without a follow id (message_type={} layer={:?})",
                self.message_type, self.layer
            ))
        })?;

        Ok(out_message.with_follow_id(follow_id))
    }
}
