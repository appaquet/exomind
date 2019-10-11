pub mod behaviour;
pub mod common_transport;
pub mod protocol;

use crate::messages::{InMessage, OutMessage};
use crate::transport::{MpscHandleSink, MpscHandleStream};
use crate::Error;
use crate::{TransportHandle, TransportLayer};
use behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage};
use exocore_common::cell::{Cell, CellId, CellNodes};
use exocore_common::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_common::node::{LocalNode, NodeId};
use exocore_common::protos::common_capnp::envelope;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use futures::prelude::*;
use futures::sync::mpsc;
use futures::MapErr;
use libp2p_core::{Multiaddr, PeerId};
use libp2p_swarm::Swarm;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;
use tokio::timer::Interval;

///
/// libp2p transport configuration
///
#[derive(Clone)]
pub struct Libp2pTransportConfig {
    pub listen_address: Option<Multiaddr>,
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub handles_to_behaviour_channel_size: usize,
    pub swarm_nodes_update_interval: Duration,
}

impl Libp2pTransportConfig {
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

impl Default for Libp2pTransportConfig {
    fn default() -> Self {
        Libp2pTransportConfig {
            listen_address: None,
            handle_in_channel_size: 1000,
            handle_out_channel_size: 1000,
            handles_to_behaviour_channel_size: 5000,
            swarm_nodes_update_interval: Duration::from_secs(1),
        }
    }
}

/// Libp2p transport used by all layers of Exocore through handles. There is one handle
/// per cell per layer.
///
/// The transport itself is scheduled on an Executor, and its future will complete as soon
/// it's ready. Once all handles are dropped, all its scheduled tasks will be stopped too.
pub struct Libp2pTransport {
    local_node: LocalNode,
    config: Libp2pTransportConfig,
    start_notifier: CompletionNotifier<(), Error>,
    stop_listener: CompletionListener<(), Error>,
    inner: Arc<RwLock<InnerTransport>>,
}

struct InnerTransport {
    stop_notifier: CompletionNotifier<(), Error>,
    handles: HashMap<(CellId, TransportLayer), InnerHandle>,
}

impl InnerTransport {
    fn all_peers(&self) -> HashSet<(PeerId, Vec<Multiaddr>)> {
        let mut peers = HashSet::new();
        for inner_layer in self.handles.values() {
            for node in inner_layer.cell.nodes().iter().all() {
                peers.insert((node.peer_id().clone(), node.addresses()));
            }
        }
        peers
    }

    fn remove_handle(&mut self, cell_id: &CellId, layer: TransportLayer) {
        self.handles.remove(&(cell_id.clone(), layer));
        if self.handles.is_empty() {
            self.stop_notifier.complete(Ok(()));
        }
    }
}

struct InnerHandle {
    cell: Cell,
    in_sender: mpsc::Sender<InMessage>,
    out_receiver: Option<mpsc::Receiver<OutMessage>>,
}

impl Libp2pTransport {
    ///
    /// Creates a new transport for given node and config. The node is important here
    /// since all messages are authenticated using the node's private key thanks to secio
    ///
    pub fn new(local_node: LocalNode, config: Libp2pTransportConfig) -> Libp2pTransport {
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();

        let inner = InnerTransport {
            stop_notifier,
            handles: HashMap::new(),
        };

        Libp2pTransport {
            local_node,
            config,
            start_notifier: CompletionNotifier::new(),
            stop_listener,
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    ///
    /// Creates sink and streams that can be used for a given Cell and Layer
    ///
    pub fn get_handle(
        &mut self,
        cell: Cell,
        layer: TransportLayer,
    ) -> Result<Libp2pTransportHandle, Error> {
        let (in_sender, in_receiver) = mpsc::channel(self.config.handle_in_channel_size);
        let (out_sender, out_receiver) = mpsc::channel(self.config.handle_out_channel_size);

        let mut inner = self.inner.write()?;
        let start_listener = self.start_notifier.get_listener().map_err(|err| {
            Error::Other(format!(
                "Couldn't get listener on start notifier: {:?}",
                err
            ))
        })?;

        let stop_listener = inner.stop_notifier.get_listener().map_err(|err| {
            Error::Other(format!(
                "Couldn't get listener on start notifier: {:?}",
                err
            ))
        })?;

        let inner_layer = InnerHandle {
            cell: cell.clone(),
            in_sender,
            out_receiver: Some(out_receiver),
        };

        info!(
            "Registering transport for cell {} and layer {:?}",
            cell.id(),
            layer
        );
        let key = (cell.id().clone(), layer);
        inner.handles.insert(key, inner_layer);

        Ok(Libp2pTransportHandle {
            cell_id: cell.id().clone(),
            start_listener,
            layer,
            inner: Arc::downgrade(&self.inner),
            sink: Some(out_sender),
            stream: Some(in_receiver),
            stop_listener,
        })
    }

    ///
    /// Starts the engine by spawning different tasks onto the current Runtime
    ///
    fn start(&mut self) -> Result<(), Error> {
        let local_keypair = self.local_node.keypair().clone();
        let transport =
            common_transport::build_tcp_ws_secio_mplex_yamux(local_keypair.to_libp2p().clone());

        let behaviour = ExocoreBehaviour::new();
        let mut swarm = Swarm::new(transport, behaviour, self.local_node.peer_id().clone());

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
            {
                // check if we should still be running
                if let Ok(inner) = inner.read() {
                    if inner.handles.is_empty() {
                        info!("No more handles are running. Stopping transport");
                        return Ok(Async::Ready(()));
                    }
                }
            }

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

            // we drain all messages coming from handles that need to be sent
            while let Async::Ready(Some(msg)) = out_receiver
                .poll()
                .expect("Couldn't poll behaviour channel")
            {
                let frame_data = msg.envelope_builder.as_bytes();

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

            // we poll the behaviour for incoming messages to be dispatched to handles
            while let Async::Ready(Some(data)) = swarm.poll().expect("Couldn't poll swarm") {
                match data {
                    ExocoreBehaviourEvent::Message(msg) => {
                        trace!("Got message from {}", msg.source);

                        if let Err(err) = Self::dispatch_message(&inner, msg) {
                            warn!("Couldn't dispatch message: {}", err);
                        }
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

    ///
    /// Dispatches a received message from libp2p to corresponding handle
    ///
    fn dispatch_message(
        inner: &RwLock<InnerTransport>,
        message: ExocoreBehaviourMessage,
    ) -> Result<(), Error> {
        let frame = TypedCapnpFrame::<_, envelope::Owned>::new(message.data)?;
        let frame_reader: envelope::Reader = frame.get_reader()?;
        let cell_id_bytes = frame_reader.get_cell_id()?;

        let mut inner = inner.write()?;

        let cell_id = CellId::from_bytes(&cell_id_bytes);
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

        let node_id = NodeId::from_peer_id(&message.source);
        let cell_nodes = layer_stream.cell.nodes();
        let source_node = if let Some(source_node) = cell_nodes.get(&node_id) {
            source_node
        } else {
            return Err(Error::Other(format!(
                "Couldn't find node with id {} in local nodes",
                node_id
            )));
        };

        let msg = InMessage::from_node_and_frame(source_node.clone(), frame.to_owned())?;
        layer_stream
            .in_sender
            .try_send(msg)
            .map_err(|err| Error::Other(format!("Couldn't send message to cell layer: {}", err)))
    }
}

impl Future for Libp2pTransport {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.start_notifier.is_complete() {
            self.start()?;
            self.start_notifier.complete(Ok(()));
        }

        self.stop_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            _ => Error::Other("Error in completion error".to_string()),
        })
    }
}

///
/// Handle taken by a Cell layer to receive and send message for a given node & cell.
///
pub struct Libp2pTransportHandle {
    cell_id: CellId,
    layer: TransportLayer,
    start_listener: CompletionListener<(), Error>,
    inner: Weak<RwLock<InnerTransport>>,
    sink: Option<mpsc::Sender<OutMessage>>,
    stream: Option<mpsc::Receiver<InMessage>>,
    stop_listener: CompletionListener<(), Error>,
}

type StartFutureType = MapErr<CompletionListener<(), Error>, fn(CompletionError<Error>) -> Error>;

impl TransportHandle for Libp2pTransportHandle {
    type StartFuture = StartFutureType;
    type Sink = MpscHandleSink;
    type Stream = MpscHandleStream;

    fn on_start(&self) -> Self::StartFuture {
        self.start_listener
            .try_clone()
            .expect("Couldn't clone start listener")
            .map_err(|err| match err {
                CompletionError::UserError(err) => err,
                _ => Error::Other("Error in completion error".to_string()),
            })
    }

    fn get_sink(&mut self) -> Self::Sink {
        MpscHandleSink::new(self.sink.take().expect("Sink was already consumed"))
    }

    fn get_stream(&mut self) -> Self::Stream {
        MpscHandleStream::new(self.stream.take().expect("Stream was already consumed"))
    }
}

impl Future for Libp2pTransportHandle {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.stop_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            _ => Error::Other("Error in completion error".to_string()),
        })
    }
}

impl Drop for Libp2pTransportHandle {
    fn drop(&mut self) {
        debug!(
            "Transport handle for cell {} layer {:?} got dropped. Removing it from transport",
            self.cell_id, self.layer
        );

        // we have been dropped, we remove ourself from layers to communicate with
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.remove_handle(&self.cell_id, self.layer);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exocore_common::cell::FullCell;
    use exocore_common::framing::CapnpFrameBuilder;
    use exocore_common::protos::data_chain_capnp::block_operation_header;
    use exocore_common::tests_utils::expect_eventually;
    use std::sync::Mutex;
    use tokio::runtime::Runtime;

    #[test]
    fn test_integration() -> Result<(), failure::Error> {
        let mut rt = Runtime::new()?;

        let node1 = LocalNode::generate();
        node1.add_address("/ip4/127.0.0.1/tcp/3003".parse().unwrap());
        let node1_cell = FullCell::generate(node1.clone());

        let node2 = LocalNode::generate();
        node2.add_address("/ip4/127.0.0.1/tcp/3004".parse().unwrap());
        let node2_cell = node1_cell.clone_for_local_node(node2.clone());

        node1_cell.nodes_mut().add(node2.node().clone());
        node2_cell.nodes_mut().add(node1.node().clone());

        let mut transport1 = Libp2pTransport::new(node1.clone(), Libp2pTransportConfig::default());
        let handle1 = transport1.get_handle(node1_cell.cell().clone(), TransportLayer::Data)?;
        let handle1_tester = TransportHandlTester::new(&mut rt, handle1);
        rt.spawn(transport1.map(|_| ()).map_err(|_| ()));
        rt.block_on(handle1_tester.handle.on_start())?;

        let mut transport2 = Libp2pTransport::new(node2.clone(), Libp2pTransportConfig::default());
        let handle2 = transport2.get_handle(node2_cell.cell().clone(), TransportLayer::Data)?;
        let handle2_tester = TransportHandlTester::new(&mut rt, handle2);
        rt.spawn(transport2.map(|_| ()).map_err(|_| ()));
        rt.block_on(handle2_tester.handle.on_start())?;

        // give time for nodes to connect to each others
        std::thread::sleep(Duration::from_millis(500));

        // send 1 to 2
        let to_nodes = vec![node2.node().clone()];
        let mut frame_builder = CapnpFrameBuilder::<block_operation_header::Owned>::new();
        let _builder = frame_builder.get_builder();
        let msg =
            OutMessage::from_framed_message(&node1_cell, TransportLayer::Data, frame_builder)?
                .with_to_nodes(to_nodes);
        handle1_tester.send(msg);
        expect_eventually(|| handle2_tester.received().len() == 1);

        // send 2 to twice 1 to test multiple nodes
        let to_nodes = vec![node1.node().clone(), node1.node().clone()];
        let mut frame_builder = CapnpFrameBuilder::<block_operation_header::Owned>::new();
        let _builder = frame_builder.get_builder();
        let msg =
            OutMessage::from_framed_message(&node2_cell, TransportLayer::Data, frame_builder)?
                .with_to_nodes(to_nodes);
        handle2_tester.send(msg);
        expect_eventually(|| handle1_tester.received().len() == 2);

        Ok(())
    }

    #[test]
    fn listen_address_override() -> Result<(), failure::Error> {
        let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/1000".parse()?;
        let addr2: Multiaddr = "/ip4/127.0.0.1/tcp/1001".parse()?;

        // config always take precedence
        let node1 = LocalNode::generate();
        node1.add_address(addr1.clone());
        let config = Libp2pTransportConfig {
            listen_address: Some(addr2.clone()),
            ..Libp2pTransportConfig::default()
        };
        assert_eq!(addr2.clone(), config.listen_address(&node1)?);

        // fallback to node if not specified in config
        let node1 = LocalNode::generate();
        node1.add_address(addr1.clone());
        let config = Libp2pTransportConfig::default();
        assert_eq!(addr1.clone(), config.listen_address(&node1)?);

        // error if no addresses found
        let node1 = LocalNode::generate();
        let config = Libp2pTransportConfig::default();
        assert!(config.listen_address(&node1).is_err());

        Ok(())
    }

    #[test]
    fn handle_removal_and_transport_kill() -> Result<(), failure::Error> {
        let mut rt = Runtime::new()?;

        let node1 = LocalNode::generate();
        node1.add_address("/ip4/127.0.0.1/tcp/0".parse()?);
        let node1_cell = FullCell::generate(node1.clone());

        let node2 = LocalNode::generate();
        node2.add_address("/ip4/127.0.0.1/tcp/0".parse()?);
        let node2_cell = FullCell::generate(node2.clone());

        let mut transport = Libp2pTransport::new(node1.clone(), Libp2pTransportConfig::default());
        let inner_weak = Arc::downgrade(&transport.inner);

        // we create 2 handles
        let handle1 = transport.get_handle(node1_cell.cell().clone(), TransportLayer::Data)?;
        let handle1_tester = TransportHandlTester::new(&mut rt, handle1);

        let handle2 = transport.get_handle(node2_cell.cell().clone(), TransportLayer::Data)?;
        let handle2_tester = TransportHandlTester::new(&mut rt, handle2);

        rt.spawn(transport.map(|_| ()).map_err(|_| ()));
        rt.block_on(handle1_tester.handle.on_start())?;

        // we drop first handle, we expect inner to now contain its handle anymore
        {
            drop(handle1_tester);
            let inner = inner_weak.upgrade().unwrap();
            let inner = inner.read().unwrap();
            assert_eq!(1, inner.handles.len());
        }

        // we drop second handle, we expect inner to be dropped and therefor transport killed
        drop(handle2_tester);
        expect_eventually(|| inner_weak.upgrade().is_none());

        Ok(())
    }

    struct TransportHandlTester {
        handle: Libp2pTransportHandle,
        sender: mpsc::UnboundedSender<OutMessage>,
        received: Arc<Mutex<Vec<InMessage>>>,
    }

    impl TransportHandlTester {
        fn new(rt: &mut Runtime, mut handle: Libp2pTransportHandle) -> TransportHandlTester {
            let (sender, receiver) = mpsc::unbounded();
            rt.spawn(
                receiver
                    .forward(handle.get_sink().sink_map_err(|_| ()))
                    .map(|_| ())
                    .map_err(|_| ()),
            );

            let received = Arc::new(Mutex::new(Vec::new()));
            let received_weak = Arc::downgrade(&received);
            rt.spawn(
                handle
                    .get_stream()
                    .for_each(move |msg| {
                        let received = received_weak.upgrade().unwrap();
                        let mut received = received.lock().unwrap();
                        received.push(msg);
                        Ok(())
                    })
                    .map_err(|_| ()),
            );

            TransportHandlTester {
                handle,
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
