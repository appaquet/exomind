pub mod behaviour;
pub mod protocol;

use crate::layer::{TransportHandle, TransportLayer};
use crate::messages::{InMessage, OutMessage};
use crate::Error;
use behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage};
use exocore_common::cell::{Cell, CellID, CellNodes};
use exocore_common::node::LocalNode;
use exocore_common::serialization::framed::{TypedFrame, TypedSliceFrame};
use exocore_common::serialization::protos::data_transport_capnp::envelope;
use futures::prelude::*;
use futures::sync::mpsc;
use libp2p::{Multiaddr, PeerId, Swarm};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;
use tokio::timer::Interval;

/// libp2p transport configuration
#[derive(Clone)]
pub struct Config {
    pub listen_address: Option<Multiaddr>,
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub handles_to_behaviour_channel_size: usize,
    pub swarm_nodes_update_interval: Duration,
}

impl Config {
    fn listen_address(&self, local_node: &LocalNode) -> Result<Multiaddr, Error> {
        self
            .listen_address
            .as_ref()
            .cloned()
            .or_else(|| local_node.addresses().first().cloned())
             .ok_or_else(|| {
                 Error::Other("Local node has no addresses, and no listen address were specified in transport config".to_string())
             })
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_address: None,
            handle_in_channel_size: 1000,
            handle_out_channel_size: 1000,
            handles_to_behaviour_channel_size: 5000,
            swarm_nodes_update_interval: Duration::from_secs(1),
        }
    }
}

/// libp2p transport used by all layers of Exocore through handles. There is one handle
/// per cell per layer.
pub struct Libp2pTransport {
    local_node: LocalNode,
    config: Config,
    inner: Arc<RwLock<Inner>>,
}

struct Inner {
    handles: HashMap<(CellID, TransportLayer), InnerHandle>,
}

impl Inner {
    fn all_peers(&self) -> HashSet<(PeerId, Vec<Multiaddr>)> {
        let mut peers = HashSet::new();
        for inner_layer in self.handles.values() {
            for node in inner_layer.cell.nodes().iter().all() {
                peers.insert((node.peer_id().clone(), node.addresses()));
            }
        }
        peers
    }
}

struct InnerHandle {
    cell: Cell,
    in_sender: mpsc::Sender<InMessage>,
    out_receiver: Option<mpsc::Receiver<OutMessage>>,
}

impl Libp2pTransport {
    /// Creates a new transport for given node and config. The node is important here
    /// since all messages are authenticated using the node's private key thanks to secio
    pub fn new(local_node: LocalNode, config: Config) -> Libp2pTransport {
        let inner = Inner {
            handles: HashMap::new(),
        };

        Libp2pTransport {
            local_node,
            config,
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Creates sink and streams that can be used for a given Cell and Layer
    pub fn get_handle(
        &mut self,
        cell: Cell,
        layer: TransportLayer,
    ) -> Result<Libp2pTransportHandle, Error> {
        let (in_sender, in_receiver) = mpsc::channel(self.config.handle_in_channel_size);
        let (out_sender, out_receiver) = mpsc::channel(self.config.handle_out_channel_size);

        let mut inner = self.inner.write()?;
        let inner_layer = InnerHandle {
            cell: cell.clone(),
            in_sender,
            out_receiver: Some(out_receiver),
        };
        inner
            .handles
            .insert((cell.id().clone(), layer), inner_layer);

        Ok(Libp2pTransportHandle {
            cell_id: cell.id().clone(),
            layer,
            inner: Arc::downgrade(&self.inner),
            sink: Some(out_sender),
            stream: Some(in_receiver),
        })
    }

    /// Starts the engine by spawning different tasks onto the current Runtime
    fn start(&mut self) -> Result<(), Error> {
        let local_keypair = self.local_node.keypair().clone();
        let transport = libp2p::build_development_transport(local_keypair);

        let behaviour = ExocoreBehaviour::new();
        let mut swarm =
            libp2p::core::Swarm::new(transport, behaviour, self.local_node.peer_id().clone());

        let listen_address = self.config.listen_address(&self.local_node)?;
        Swarm::listen_on(&mut swarm, listen_address)?;

        // Spawn the swarm & receive message from a channel through which outgoing messages will go
        let (out_sender, mut out_receiver) =
            mpsc::channel::<OutMessage>(self.config.handles_to_behaviour_channel_size);

        // Add initial nodes to swarm
        {
            let inner = self.inner.read()?;
            for (peer_id, addresses) in inner.all_peers() {
                swarm.add_peer(peer_id, addresses);
            }
        }

        // Spawn the main Future which will take care of the swarm
        let inner = Arc::clone(&self.inner);
        let mut nodes_update_interval =
            Interval::new_interval(self.config.swarm_nodes_update_interval);

        tokio::spawn(futures::future::poll_fn(move || -> Result<_, ()> {
            // at interval, we update peers that we should be connected to
            if let Async::Ready(_) = nodes_update_interval
                .poll()
                .expect("Couldn't poll nodes update interval")
            {
                if let Ok(inner) = inner.read() {
                    for (peer_id, addresses) in inner.all_peers() {
                        swarm.add_peer(peer_id, addresses);
                    }
                }
            }

            // we drain all messages that need to be sent
            while let Async::Ready(Some(msg)) = out_receiver
                .poll()
                .expect("Couldn't poll behaviour channel")
            {
                // we don't need to sign the message since it's going through a authenticated channel (secio)
                match msg.envelope.as_owned_unsigned_framed() {
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

            // we poll the behaviour for incoming messages
            while let Async::Ready(Some(data)) = swarm.poll().expect("Couldn't poll swarm") {
                match data {
                    ExocoreBehaviourEvent::Message(msg) => {
                        if let Err(err) = Self::dispatch_message(&inner, &msg) {
                            warn!("Couldn't dispatch message from {}: {:?}", msg.source, err);
                        }

                        trace!("Got message from {}", msg.source,);
                    }
                }
            }

            Ok(Async::NotReady)
        }));

        // Sends each layer's outgoing messages to the behaviour's input channel
        {
            let mut inner = self.inner.write()?;
            for inner_layer in inner.handles.values_mut() {
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
        }

        Ok(())
    }

    /// Dispatches a received message from libp2p to corresponding handle
    fn dispatch_message(
        inner: &RwLock<Inner>,
        message: &ExocoreBehaviourMessage,
    ) -> Result<(), Error> {
        let frame = TypedSliceFrame::<envelope::Owned>::new(&message.data)?;
        let frame_reader: envelope::Reader = frame.get_typed_reader()?;
        let cell_id_bytes = frame_reader.get_cell_id()?;

        let mut inner = inner.write()?;

        let cell_id = CellID::from_bytes(&cell_id_bytes);
        let layer = TransportLayer::from_code(frame_reader.get_layer()).ok_or_else(|| {
            Error::Other(format!(
                "Message has invalid layer {}",
                frame_reader.get_layer()
            ))
        })?;

        let key = (cell_id, layer);
        let layer_stream = if let Some(layer_stream) = inner.handles.get_mut(&key) {
            layer_stream
        } else {
            return Err(Error::Other(format!(
                "Couldn't find transport for {:?}",
                key
            )));
        };

        let node_id = exocore_common::node::node_id_from_peer_id(&message.source);
        let cell_nodes = layer_stream.cell.nodes();
        let source_node = if let Some(source_node) = cell_nodes.get(&node_id) {
            source_node
        } else {
            return Err(Error::Other(format!(
                "Couldn't find node with id {} in local nodes",
                node_id
            )));
        };

        let msg = InMessage {
            from: source_node.clone(),
            envelope: frame.to_owned(),
        };

        layer_stream
            .in_sender
            .try_send(msg)
            .map_err(|err| Error::Other(format!("Couldn't send message to cell layer: {:?}", err)))
    }
}

impl Future for Libp2pTransport {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        // we are only polled once, and we return ready right away
        self.start().map_err(|err| {
            error!("Error starting transport: {:?}", err);
        })?;

        Ok(Async::Ready(()))
    }
}

/// Handle taken by a Cell layer to receive and send message for a given node & cell.
pub struct Libp2pTransportHandle {
    cell_id: CellID,
    layer: TransportLayer,
    inner: Weak<RwLock<Inner>>,

    sink: Option<mpsc::Sender<OutMessage>>,
    stream: Option<mpsc::Receiver<InMessage>>,
}

impl TransportHandle for Libp2pTransportHandle {
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

impl Future for Libp2pTransportHandle {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        Ok(Async::Ready(()))
    }
}

impl Drop for Libp2pTransportHandle {
    fn drop(&mut self) {
        // we have been dropped, we remove ourself from layers to communicate with
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.handles.remove(&(self.cell_id.clone(), self.layer));
            }
        }
    }
}

/// Wraps mpsc Stream channel to map Transport's error without having a convoluted type
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

/// Wraps mpsc Sink channel to map Transport's error without having a convoluted type
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

#[cfg(test)]
mod tests {
    use super::*;
    use exocore_common::cell::FullCell;
    use exocore_common::serialization::framed::FrameBuilder;
    use exocore_common::serialization::protos::data_chain_capnp::block_operation_header;
    use exocore_common::tests_utils::expect_eventually;
    use std::sync::Mutex;
    use std::time::Duration;
    use tokio::runtime::Runtime;

    #[test]
    fn test_integration() -> Result<(), failure::Error> {
        let mut rt = Runtime::new()?;

        let node1 = LocalNode::generate();
        node1.add_address("/ip4/127.0.0.1/tcp/3303".parse().unwrap());
        let node1_cell = FullCell::generate(node1.clone());

        let node2 = LocalNode::generate();
        node2.add_address("/ip4/127.0.0.1/tcp/3304".parse().unwrap());
        let node2_cell = node1_cell.clone_for_local_node(node2.clone());

        node1_cell.nodes_mut().add(node2.node().clone());
        node2_cell.nodes_mut().add(node1.node().clone());

        let mut transport1 = Libp2pTransport::new(node1.clone(), Config::default());
        let layer1 = transport1.get_handle(node1_cell.cell().clone(), TransportLayer::Data)?;
        let layer1_tester = LayerTransportTester::new(&mut rt, layer1);
        rt.spawn(transport1);

        let mut transport2 = Libp2pTransport::new(node2.clone(), Config::default());
        let layer2 = transport2.get_handle(node2_cell.cell().clone(), TransportLayer::Data)?;
        let layer2_tester = LayerTransportTester::new(&mut rt, layer2);
        rt.spawn(transport2);

        std::thread::sleep(Duration::from_secs(1));

        // create dummy frame
        let frame_builder = FrameBuilder::<block_operation_header::Owned>::new();
        let frame = frame_builder.as_owned_unsigned_framed()?;

        // send 1 to 2
        let to_nodes = vec![node2.node().clone()];
        let msg = OutMessage::from_framed_message(&node1_cell, to_nodes, frame.clone())?;
        layer1_tester.send(msg);
        expect_eventually(|| layer2_tester.received().len() == 1);

        // send 2 to twice 1 to test multiple nodes
        let to_nodes = vec![node1.node().clone(), node1.node().clone()];
        let msg = OutMessage::from_framed_message(&node2_cell, to_nodes, frame)?;
        layer2_tester.send(msg);
        expect_eventually(|| layer1_tester.received().len() == 2);

        Ok(())
    }

    #[test]
    fn listen_address_override() -> Result<(), failure::Error> {
        let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/1000".parse()?;
        let addr2: Multiaddr = "/ip4/127.0.0.1/tcp/1001".parse()?;

        // config always take precedence
        let node1 = LocalNode::generate();
        node1.add_address(addr1.clone());
        let config = Config {
            listen_address: Some(addr2.clone()),
            ..Config::default()
        };
        assert_eq!(addr2.clone(), config.listen_address(&node1)?);

        // fallback to node if not specified in config
        let node1 = LocalNode::generate();
        node1.add_address(addr1.clone());
        let config = Config::default();
        assert_eq!(addr1.clone(), config.listen_address(&node1)?);

        // error if no addresses found
        let node1 = LocalNode::generate();
        let config = Config::default();
        assert!(config.listen_address(&node1).is_err());

        Ok(())
    }

    struct LayerTransportTester {
        _transport: Libp2pTransportHandle,
        sender: mpsc::UnboundedSender<OutMessage>,
        received: Arc<Mutex<Vec<InMessage>>>,
    }

    impl LayerTransportTester {
        fn new(rt: &mut Runtime, mut transport: Libp2pTransportHandle) -> LayerTransportTester {
            let (sender, receiver) = mpsc::unbounded();
            rt.spawn(
                receiver
                    .forward(transport.get_sink().sink_map_err(|_| ()))
                    .map(|_| ())
                    .map_err(|_| ()),
            );

            let received = Arc::new(Mutex::new(Vec::new()));
            let received_weak = Arc::downgrade(&received);
            rt.spawn(
                transport
                    .get_stream()
                    .for_each(move |msg| {
                        let received = received_weak.upgrade().unwrap();
                        let mut received = received.lock().unwrap();
                        received.push(msg);
                        Ok(())
                    })
                    .map_err(|_| ()),
            );

            LayerTransportTester {
                _transport: transport,
                sender,
                received,
            }
        }

        fn send(&self, message: OutMessage) {
            self.sender.unbounded_send(message).unwrap();
        }

        fn received(&self) -> Vec<InMessage> {
            let received = self.received.lock().unwrap();
            received.clone()
        }
    }

}
