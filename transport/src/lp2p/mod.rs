pub mod behaviour;
pub mod protocol;

use crate::layer::{Layer, LayerStreams};
use crate::messages::{InMessage, OutMessage};
use crate::{Error};
use behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage};
use exocore_common::cell::{CellID, Cell};
use exocore_common::node::{LocalNode, Node};
use exocore_common::serialization::framed::{TypedFrame, TypedSliceFrame};
use exocore_common::serialization::protos::data_transport_capnp::envelope;
use futures::prelude::*;
use futures::sink::SinkMapErr;
use futures::sync::{mpsc, oneshot};
use libp2p::core::swarm::{
    ExpandedSwarm, NetworkBehaviour, NetworkBehaviourAction, PollParameters,
};
use libp2p::identity::Keypair;
use libp2p::{identity, Multiaddr, PeerId, Swarm, Transport};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use tokio::prelude::*;

///
/// Transport
///
pub struct NodeTransport {
    local_node: LocalNode,
    config: Config,
    inner: Arc<RwLock<Inner>>,
    started: bool,
}

struct Inner {
    layers: HashMap<(CellID, Layer), InnerLayer>,
}

struct InnerLayer {
    in_sender: mpsc::Sender<InMessage>,
    out_receiver: Option<mpsc::Receiver<OutMessage>>,
}

impl NodeTransport {
    pub fn new(cell: Cell, config: Config) -> NodeTransport {
        let inner = Inner {
            layers: HashMap::new(),
        };

        NodeTransport {
            local_node: cell.nodes().local_node().clone(), // TODO: fixme
            config,
            inner: Arc::new(RwLock::new(inner)),
            started: false,
        }
    }

    pub fn get_cell_layer_transport(
        &mut self,
        cell_id: CellID,
        layer: Layer,
    ) -> Result<CellLayerTransport, Error> {
        let (in_sender, in_receiver) = mpsc::channel(self.config.layer_stream_in_channel_size);
        let (out_sender, out_receiver) = mpsc::channel(self.config.layer_stream_out_channel_size);

        let mut inner = self.inner.write()?;
        let inner_layer = InnerLayer {
            in_sender,
            out_receiver: Some(out_receiver),
        };
        inner.layers.insert((cell_id.clone(), layer), inner_layer);

        Ok(CellLayerTransport {
            cell_id,
            layer,
            inner: Arc::downgrade(&self.inner),
            sink: Some(out_sender),
            stream: Some(in_receiver),
        })
    }

    fn start(&mut self) -> Result<(), Error> {
        self.started = true;

        let local_keypair = self.local_node.keypair().clone();
        let transport = libp2p::build_development_transport(local_keypair);

        let behaviour = ExocoreBehaviour::new();
        let mut swarm =
            libp2p::core::Swarm::new(transport, behaviour, self.local_node.peer_id().clone());
        Swarm::listen_on(&mut swarm, self.config.listen_address.clone())?;

        // Spawn the swarm & receive message from a channel through which outgoing messages will go
        let (out_sender, mut out_receiver) =
            mpsc::channel::<OutMessage>(self.config.layers_to_behaviour_channel_size);
        let local_node = self.local_node.clone();
        let inner = Arc::clone(&self.inner); // TODO: Should be weak, but performance issue... Hashmap could be just moved here, and we always try to send even if dropped
        tokio::spawn(futures::future::poll_fn(move || -> Result<_, ()> {
            while let Async::Ready(Some(msg)) = out_receiver
                .poll()
                .expect("Error polling layer to behaviour channel")
            {
                // TODO: Is it really worth it to sign ? We are already going through a secio that authenticate messages with peer's signature
                match msg.envelope.as_owned_framed(local_node.frame_signer()) {
                    Ok(frame) => {
                        let frame_data = frame.frame_data().to_vec();

                        // prevent cloning frame if we only send to 1 node
                        if msg.to.len() == 1 {
                            let to_node = msg.to.first().unwrap();
                            swarm.send_message(to_node.peer_id().clone(), frame_data);
                        } else {
                            for to_node in msg.to {
                                swarm.send_message(to_node.peer_id().clone(), frame_data.clone());
                            }
                        }
                    }
                    Err(err) => {
                        error!("Couldn't serialize frame to data: {:?}", err);
                    }
                }
            }

            while let Async::Ready(Some(data)) = swarm.poll().expect("Error while polling swarm") {
                match data {
                    ExocoreBehaviourEvent::Message(msg) => {
                        if let Err(err) = Self::dispatch_message(&inner, &msg) {
                            error!("Couldn't dispatch message from {}: {:?}", msg.source, err);
                        }

                        debug!(
                            "Got message from {}: {}",
                            msg.source,
                            String::from_utf8_lossy(&msg.data)
                        );
                    }
                }
            }

            Ok(Async::NotReady)
        }));

        // sends each layer's outgoing messages to the behaviour's input channel
        let mut inner = self.inner.write()?;
        for inner_layer in inner.layers.values_mut() {
            let out_receiver = inner_layer
                .out_receiver
                .take()
                .expect("Out receiver of one layer was already consummed");

            tokio::spawn(
                out_receiver
                    .forward(out_sender.clone().sink_map_err(|_| ()))
                    .map(|_| ()),
            );
        }

        Ok(())
    }

    fn dispatch_message(
        inner: &RwLock<Inner>,
        message: &ExocoreBehaviourMessage,
    ) -> Result<(), Error> {
        let frame = TypedSliceFrame::<envelope::Owned>::new(&message.data).map_err(|err| {
            // TODO:
            Error::Other("Couldn't parse message".to_string())
        })?;
        let frame_reader: envelope::Reader = frame.get_typed_reader().map_err(|err| {
            // TODO:
            Error::Other("Couldn't parse message".to_string())
        })?;

        let mut inner = inner.write()?;

        let cell_id_bytes = frame_reader.get_cell_id().map_err(|err| {
            // TODO:
            Error::Other("Couldn't parse message".to_string())
        })?;

        let cell_id = CellID::from_bytes(&cell_id_bytes);
        let layer = Layer::from_code(frame_reader.get_layer()).ok_or_else(|| {
            Error::Other(format!(
                "Got message with invalid layer: {}",
                frame_reader.get_layer()
            ))
        })?;

        if let Some(layer_stream) = inner.layers.get_mut(&(cell_id, layer)) {
            // TODO: fix me
            let node = Node::new("".to_string());
            let msg = InMessage {
                from: node,
                envelope: frame.to_owned(),
            };

            if let Err(err) = layer_stream.in_sender.try_send(msg) {
                warn!("Couldn't send message to layer stream: {:?}", err);
            }
        }

        Ok(())
    }
}

impl Future for NodeTransport {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.started {
            self.start().map_err(|err| {
                error!("Error starting transport: {:?}", err);
                ()
            })?;
        }

        // TODO: Check it it has been dropped

        Ok(Async::Ready(()))
    }
}

///
///
///
#[derive(Clone)]
pub struct Config {
    pub listen_address: Multiaddr,
    pub layer_stream_in_channel_size: usize,
    pub layer_stream_out_channel_size: usize,
    pub layers_to_behaviour_channel_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_address: "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
            layer_stream_in_channel_size: 1000,
            layer_stream_out_channel_size: 1000,
            layers_to_behaviour_channel_size: 5000,
        }
    }
}

///
///
///
pub struct CellLayerTransport {
    cell_id: CellID,
    layer: Layer,
    inner: Weak<RwLock<Inner>>,

    sink: Option<mpsc::Sender<OutMessage>>,
    stream: Option<mpsc::Receiver<InMessage>>,
}

impl LayerStreams for CellLayerTransport {
    type Sink = MpscLayerSink;
    type Stream = MpscLayerStream;

    fn get_sink(&mut self) -> Self::Sink {
        MpscLayerSink {
            sender: self.sink.take().expect("Sink was already consumed"),
        }
    }

    fn get_stream(&mut self) -> Self::Stream {
        MpscLayerStream {
            receiver: self.stream.take().expect("Stream was already consumed"),
        }
    }
}

impl Future for CellLayerTransport {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

impl Drop for CellLayerTransport {
    fn drop(&mut self) {
        // TODO: unregister ourself
    }
}

///
///
///
pub struct MpscLayerStream {
    receiver: mpsc::Receiver<InMessage>,
}

impl Stream for MpscLayerStream {
    type Item = InMessage;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.receiver.poll().map_err(|err| {
            error!(
                "Error receiving from incoming stream in MockTransportStream: {:?}",
                err
            );
            Error::Other(format!("Error receiving from incoming stream: {:?}", err))
        })
    }
}

///
///
///
pub struct MpscLayerSink {
    sender: mpsc::Sender<OutMessage>,
}

impl Sink for MpscLayerSink {
    type SinkItem = OutMessage;
    type SinkError = Error;

    fn start_send(&mut self, item: OutMessage) -> StartSend<OutMessage, Error> {
        self.sender.start_send(item).map_err(|err| {
            Error::Other(format!(
                "Error calling 'start_send' to in_channel: {:?}",
                err
            ))
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Error> {
        self.sender.poll_complete().map_err(|err| {
            Error::Other(format!(
                "Error calling 'poll_complete' to in_channel: {:?}",
                err
            ))
        })
    }

    fn close(&mut self) -> Poll<(), Error> {
        self.sender
            .close()
            .map_err(|err| Error::Other(format!("Error calling 'close' to in_channel: {:?}", err)))
    }
}
