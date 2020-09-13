use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll};
use std::time::Duration;

use futures::channel::mpsc;
use futures::channel::mpsc::SendError;
use futures::prelude::*;
use futures::sink::SinkMapErr;
use futures::{FutureExt, SinkExt, StreamExt};
use libp2p::core::{Multiaddr, PeerId};
use libp2p::swarm::Swarm;

use behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage};
use exocore_core::cell::{Cell, CellId, CellNodes};
use exocore_core::cell::{LocalNode, Node, NodeId};
use exocore_core::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_core::protos::generated::common_capnp::envelope;
use exocore_core::utils::handle_set::{Handle, HandleSet};

use crate::messages::InMessage;
use crate::transport::{ConnectionStatus, InEvent, OutEvent, TransportHandleOnStart};
use crate::Error;
use crate::{lp2p::behaviour::PeerStatus, transport::ConnectionID};
use crate::{TransportHandle, TransportLayer};

mod behaviour;
mod protocol;

/// Libp2p transport used by all layers of Exocore through handles. There is one
/// handle per cell per layer.
///
/// The transport itself is scheduled on an Executor, and its future will
/// complete as soon it's ready. Once all handles are dropped, all its scheduled
/// tasks will be stopped too.
pub struct Libp2pTransport {
    local_node: LocalNode,
    config: Libp2pTransportConfig,
    handles: Arc<RwLock<Handles>>,
    handle_set: HandleSet,
}

impl Libp2pTransport {
    /// Creates a new transport for given node and config. The node is important
    /// here since all messages are authenticated using the node's private
    /// key thanks to secio
    pub fn new(local_node: LocalNode, config: Libp2pTransportConfig) -> Libp2pTransport {
        let inner = Handles {
            handles: HashMap::new(),
        };

        Libp2pTransport {
            local_node,
            config,
            handles: Arc::new(RwLock::new(inner)),
            handle_set: HandleSet::new(),
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

        // Register new handle and its streams
        let mut handles = self.handles.write()?;
        let inner_layer = HandleChannels {
            cell: cell.clone(),
            in_sender,
            out_receiver: Some(out_receiver),
        };
        info!(
            "Registering transport for cell {} and layer {:?}",
            cell, layer
        );
        let key = (cell.id().clone(), layer);
        handles.handles.insert(key, inner_layer);

        Ok(Libp2pTransportHandle {
            cell_id: cell.id().clone(),
            layer,
            inner: Arc::downgrade(&self.handles),
            sink: Some(out_sender),
            stream: Some(in_receiver),
            handle: self.handle_set.get_handle(),
        })
    }

    /// Runs the transport to completion.
    pub async fn run(self) -> Result<(), Error> {
        let behaviour = ExocoreBehaviour::new();

        #[cfg(all(feature = "libp2p-web", target_arch = "wasm32"))]
        let mut swarm = {
            use libp2p::{wasm_ext::ffi::websocket_transport, wasm_ext::ExtTransport, Transport};

            let noise_keys = libp2p::noise::Keypair::<libp2p::noise::X25519Spec>::new()
                .into_authentic(self.local_node.keypair().to_libp2p())
                .map_err(|err| {
                    Error::Other(format!(
                        "Signing libp2p-noise static DH keypair failed: {}",
                        err
                    ))
                })?;

            let transport = ExtTransport::new(websocket_transport())
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(libp2p::noise::NoiseConfig::xx(noise_keys).into_authenticated())
                .multiplex(libp2p::core::upgrade::SelectUpgrade::new(
                    libp2p::yamux::Config::default(),
                    libp2p::mplex::MplexConfig::new(),
                ))
                .map(|(peer, muxer), _| (peer, libp2p::core::muxing::StreamMuxerBox::new(muxer)));
            Swarm::new(transport, behaviour, self.local_node.peer_id().clone())
        };

        #[cfg(feature = "libp2p-full")]
        let mut swarm = {
            let transport = libp2p::build_tcp_ws_noise_mplex_yamux(
                self.local_node.keypair().to_libp2p().clone(),
            )?;

            // Create our own libp2p executor since by default it spawns its own thread pool
            // to spawn tcp related futures, but Tokio requires to be spawn from
            // within its runtime.
            struct CoreExecutor;
            impl libp2p::core::Executor for CoreExecutor {
                fn exec(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) {
                    exocore_core::futures::spawn_future(f)
                }
            }

            libp2p::swarm::SwarmBuilder::new(
                transport,
                behaviour,
                self.local_node.peer_id().clone(),
            )
            .executor(Box::new(CoreExecutor))
            .build()
        };

        let listen_addresses = self.config.listen_addresses(&self.local_node)?;
        for listen_address in listen_addresses {
            Swarm::listen_on(&mut swarm, listen_address)?;
        }

        // Spawn the swarm & receive message from a channel through which outgoing
        // messages will go
        let (out_sender, mut out_receiver) =
            mpsc::channel::<OutEvent>(self.config.handles_to_behaviour_channel_size);

        // Add initial nodes to swarm
        {
            let inner = self.handles.read()?;
            for node in inner.all_peer_nodes().values() {
                swarm.add_node_peer(node);
            }
        }

        let mut nodes_update_interval =
            exocore_core::futures::interval(self.config.swarm_nodes_update_interval);

        // Spawn the main Future which will take care of the swarm
        let inner = Arc::clone(&self.handles);
        let swarm_task = future::poll_fn(move |cx: &mut Context| -> Poll<()> {
            if let Poll::Ready(_) = nodes_update_interval.poll_next_unpin(cx) {
                if let Ok(inner) = inner.read() {
                    for node in inner.all_peer_nodes().values() {
                        swarm.add_node_peer(node);
                    }
                }
            }

            // we drain all messages coming from handles that need to be sent
            while let Poll::Ready(Some(event)) = out_receiver.poll_next_unpin(cx) {
                match event {
                    OutEvent::Message(msg) => {
                        let frame_data = msg.envelope_builder.as_bytes(); // TODO: Should be to an Arc

                        let connection =
                            if let Some(ConnectionID::Libp2p(connection)) = msg.connection {
                                Some(connection)
                            } else {
                                None
                            };

                        // prevent cloning frame if we only send to 1 node
                        if msg.to.len() == 1 {
                            let to_node = msg.to.first().unwrap();
                            swarm.send_message(
                                to_node.peer_id().clone(),
                                msg.expiration,
                                connection,
                                frame_data,
                            );
                        } else {
                            for to_node in msg.to {
                                swarm.send_message(
                                    to_node.peer_id().clone(),
                                    msg.expiration,
                                    connection,
                                    frame_data.clone(),
                                );
                            }
                        }
                    }
                }
            }

            // we poll the behaviour for incoming messages to be dispatched to handles
            while let Poll::Ready(Some(data)) = swarm.poll_next_unpin(cx) {
                match data {
                    ExocoreBehaviourEvent::Message(msg) => {
                        trace!("Got message from {}", msg.source);

                        if let Err(err) = Self::dispatch_message(&inner, msg) {
                            warn!("Couldn't dispatch message: {}", err);
                        }
                    }
                    ExocoreBehaviourEvent::PeerStatus(peer_id, status) => {
                        if let Err(err) = Self::dispatch_node_status(&inner, peer_id, status) {
                            warn!("Couldn't dispatch node status: {}", err);
                        }
                    }
                }
            }

            Poll::Pending
        });

        // Sends handles' outgoing messages to the behaviour's input channel
        let handles_dispatcher = {
            let mut inner = self.handles.write()?;
            let mut futures = Vec::new();
            for inner_layer in inner.handles.values_mut() {
                let out_receiver = inner_layer
                    .out_receiver
                    .take()
                    .expect("Out receiver of one layer was already consumed");

                let mut out_sender = out_sender.clone();
                futures.push(async move {
                    let mut out_receiver = out_receiver;
                    while let Some(event) = out_receiver.next().await {
                        let _ = out_sender.send(event).await;
                    }
                });
            }
            futures::future::join_all(futures)
        };

        info!("Libp2p transport now running");
        futures::select! {
            _ = swarm_task.fuse() => (),
            _ = handles_dispatcher.fuse() => (),
            _ = self.handle_set.on_handles_dropped().fuse() => (),
        };
        info!("Libp2p transport is done");

        Ok(())
    }

    /// Dispatches a received message from libp2p to corresponding handle
    fn dispatch_message(
        inner: &RwLock<Handles>,
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
        let handle_channels = if let Some(layer_stream) = inner.handles.get_mut(&key) {
            layer_stream
        } else {
            return Err(Error::Other(format!(
                "Couldn't find transport for {:?}",
                key
            )));
        };

        let source_node = Self::get_node_by_peer(&handle_channels.cell, message.source)?;
        let mut msg = InMessage::from_node_and_frame(source_node, frame.to_owned())?;
        msg.connection = Some(ConnectionID::Libp2p(message.connection));

        handle_channels
            .in_sender
            .try_send(InEvent::Message(msg))
            .map_err(|err| Error::Other(format!("Couldn't send message to cell layer: {}", err)))
    }

    /// Dispatches a node status change.
    fn dispatch_node_status(
        inner: &RwLock<Handles>,
        peer_id: PeerId,
        peer_status: PeerStatus,
    ) -> Result<(), Error> {
        let mut inner = inner.write()?;

        let status = match peer_status {
            PeerStatus::Connected => ConnectionStatus::Connected,
            PeerStatus::Disconnected => ConnectionStatus::Disconnected,
        };

        for handle in inner.handles.values_mut() {
            if let Ok(node) = Self::get_node_by_peer(&handle.cell, peer_id.clone()) {
                handle
                    .in_sender
                    .try_send(InEvent::NodeStatus(node.id().clone(), status))
                    .map_err(|err| {
                        Error::Other(format!("Couldn't send message to cell layer: {}", err))
                    })?;
            }
        }

        Ok(())
    }

    fn get_node_by_peer(cell: &Cell, peer_id: PeerId) -> Result<Node, Error> {
        let node_id = NodeId::from_peer_id(peer_id);
        let cell_nodes = cell.nodes();

        if let Some(source_node) = cell_nodes.get(&node_id) {
            Ok(source_node.node().clone())
        } else {
            Err(Error::Other(format!(
                "Couldn't find node with id {} in local nodes",
                node_id
            )))
        }
    }
}

/// `Libp2pTransport` configuration.
#[derive(Clone)]
pub struct Libp2pTransportConfig {
    pub listen_addresses: Vec<Multiaddr>,
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub handles_to_behaviour_channel_size: usize,
    pub swarm_nodes_update_interval: Duration,
}

impl Libp2pTransportConfig {
    fn listen_addresses(&self, local_node: &LocalNode) -> Result<Vec<Multiaddr>, Error> {
        let mut conf_addresses = self.listen_addresses.clone();
        let mut node_addresses = local_node.addresses();

        node_addresses.append(&mut conf_addresses);

        Ok(node_addresses)
    }
}

impl Default for Libp2pTransportConfig {
    fn default() -> Self {
        Libp2pTransportConfig {
            listen_addresses: Vec::new(),
            handle_in_channel_size: 1000,
            handle_out_channel_size: 1000,
            handles_to_behaviour_channel_size: 5000,
            swarm_nodes_update_interval: Duration::from_secs(1),
        }
    }
}

/// Transport handles created on the `Libp2pTransport`.
///
/// A transport can be used for multiple cells, so multiple handles for the same
/// layers, but on different cells may be created.
struct Handles {
    handles: HashMap<(CellId, TransportLayer), HandleChannels>,
}

impl Handles {
    fn all_peer_nodes(&self) -> HashMap<NodeId, Node> {
        let mut nodes = HashMap::new();
        for inner_layer in self.handles.values() {
            for cell_node in inner_layer.cell.nodes().iter().all_except_local() {
                let node = cell_node.node().clone();
                nodes.insert(node.id().clone(), node);
            }
        }
        nodes
    }

    fn remove_handle(&mut self, cell_id: &CellId, layer: TransportLayer) {
        self.handles.remove(&(cell_id.clone(), layer));
    }
}

struct HandleChannels {
    cell: Cell,
    in_sender: mpsc::Sender<InEvent>,
    out_receiver: Option<mpsc::Receiver<OutEvent>>,
}

/// Handle taken by a Cell layer to receive and send message for a given node &
/// cell.
pub struct Libp2pTransportHandle {
    cell_id: CellId,
    layer: TransportLayer,
    inner: Weak<RwLock<Handles>>,
    sink: Option<mpsc::Sender<OutEvent>>,
    stream: Option<mpsc::Receiver<InEvent>>,
    handle: Handle,
}

impl TransportHandle for Libp2pTransportHandle {
    type Sink = SinkMapErr<mpsc::Sender<OutEvent>, fn(SendError) -> Error>;
    type Stream = mpsc::Receiver<InEvent>;

    fn on_started(&self) -> TransportHandleOnStart {
        Box::new(self.handle.on_set_started())
    }

    fn get_sink(&mut self) -> Self::Sink {
        self.sink
            .take()
            .expect("Sink was already consumed")
            .sink_map_err(|err| Error::Other(format!("Sink error: {}", err)))
    }

    fn get_stream(&mut self) -> Self::Stream {
        self.stream.take().expect("Stream was already consumed")
    }
}

impl Future for Libp2pTransportHandle {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.handle.on_set_dropped().poll_unpin(cx)
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
    use std::sync::Mutex;

    use futures::{SinkExt, StreamExt};

    use exocore_core::cell::Node;
    use exocore_core::framing::CapnpFrameBuilder;
    use exocore_core::futures::Runtime;
    use exocore_core::protos::generated::data_chain_capnp::block_operation_header;
    use exocore_core::tests_utils::expect_eventually;
    use exocore_core::time::{ConsistentTimestamp, Instant};
    use exocore_core::{cell::FullCell, tests_utils::expect_result_eventually};

    use crate::OutMessage;

    use super::*;

    #[test]
    fn test_integration() -> anyhow::Result<()> {
        let mut rt = exocore_core::futures::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()?;

        let node1 = LocalNode::generate();
        node1.add_address("/ip4/127.0.0.1/tcp/3003".parse().unwrap());
        let node1_cell = FullCell::generate(node1.clone());

        let node2 = LocalNode::generate();
        node2.add_address("/ip4/127.0.0.1/tcp/3004".parse().unwrap());
        let node2_cell = node1_cell.clone().with_local_node(node2.clone());

        node1_cell.nodes_mut().add(node2.node().clone());
        node2_cell.nodes_mut().add(node1.node().clone());

        let mut transport1 = Libp2pTransport::new(node1.clone(), Libp2pTransportConfig::default());
        let handle1 = transport1.get_handle(node1_cell.cell().clone(), TransportLayer::Chain)?;
        let handle1_tester = TransportHandleTester::new(&mut rt, handle1, node1_cell.clone());
        rt.spawn(async {
            let res = transport1.run().await;
            info!("Transport done: {:?}", res);
        });
        handle1_tester.start_handle(&mut rt);

        let mut transport2 = Libp2pTransport::new(node2.clone(), Libp2pTransportConfig::default());
        let handle2 = transport2.get_handle(node2_cell.cell().clone(), TransportLayer::Chain)?;
        let handle2_tester = TransportHandleTester::new(&mut rt, handle2, node2_cell);
        rt.spawn(async {
            let res = transport2.run().await;
            info!("Transport done: {:?}", res);
        });
        handle2_tester.start_handle(&mut rt);

        // wait for nodes to be connected
        expect_eventually(|| {
            handle1_tester.node_status(node2.id()) == ConnectionStatus::Connected
                && handle2_tester.node_status(node1.id()) == ConnectionStatus::Connected
        });

        // send 1 to 2
        handle1_tester.send(vec![node2.node().clone()], 123);
        let msg = expect_result_eventually(|| {
            handle2_tester
                .receive_memo_message(123)
                .ok_or_else(|| anyhow::anyhow!("not received"))
        });

        // reply to message
        let msg_frame = TransportHandleTester::empty_message_frame();
        let reply_msg = msg.to_response_message(node1_cell.cell(), msg_frame)?;
        handle2_tester.send_message(reply_msg);
        expect_eventually(|| handle1_tester.check_received_memo_message(123));

        // send 2 to 1 by duplicating node, should expect receiving 2 new messages (so total 3 because of prev reply)
        handle2_tester.send(vec![node1.node().clone(), node1.node().clone()], 345);
        expect_eventually(|| handle1_tester.received_messages().len() == 3);

        Ok(())
    }

    #[test]
    fn handle_removal_and_transport_kill() -> anyhow::Result<()> {
        let mut rt = Runtime::new()?;

        let node1 = LocalNode::generate();
        node1.add_address("/ip4/127.0.0.1/tcp/0".parse()?);
        let node1_cell = FullCell::generate(node1.clone());

        let node2 = LocalNode::generate();
        node2.add_address("/ip4/127.0.0.1/tcp/0".parse()?);
        let node2_cell = FullCell::generate(node2);

        let mut transport = Libp2pTransport::new(node1, Libp2pTransportConfig::default());
        let inner_weak = Arc::downgrade(&transport.handles);

        // we create 2 handles
        let handle1 = transport.get_handle(node1_cell.cell().clone(), TransportLayer::Chain)?;
        let handle1_tester = TransportHandleTester::new(&mut rt, handle1, node1_cell);

        let handle2 = transport.get_handle(node2_cell.cell().clone(), TransportLayer::Chain)?;
        let handle2_tester = TransportHandleTester::new(&mut rt, handle2, node2_cell);

        rt.spawn(async {
            let res = transport.run().await;
            info!("Transport done: {:?}", res);
        });
        handle1_tester.start_handle(&mut rt);

        // we drop first handle, we expect inner to now contain its handle anymore
        {
            drop(handle1_tester);
            let inner = inner_weak.upgrade().unwrap();
            let inner = inner.read().unwrap();
            assert_eq!(1, inner.handles.len());
        }

        // we drop second handle, we expect inner to be dropped and therefor transport
        // killed
        drop(handle2_tester);
        expect_eventually(|| inner_weak.upgrade().is_none());

        Ok(())
    }

    #[test]
    fn should_queue_message_until_connected() -> anyhow::Result<()> {
        let mut rt = Runtime::new()?;

        let node1 = LocalNode::generate();
        node1.add_address("/ip4/127.0.0.1/tcp/3005".parse().unwrap());
        let node1_cell = FullCell::generate(node1.clone());

        let node2 = LocalNode::generate();
        node2.add_address("/ip4/127.0.0.1/tcp/3006".parse().unwrap());
        let node2_cell = node1_cell.clone().with_local_node(node2.clone());

        node1_cell.nodes_mut().add(node2.node().clone());
        node2_cell.nodes_mut().add(node1.node().clone());

        let mut transport1 = Libp2pTransport::new(node1, Libp2pTransportConfig::default());
        let handle1 = transport1.get_handle(node1_cell.cell().clone(), TransportLayer::Chain)?;
        let handle1_tester = TransportHandleTester::new(&mut rt, handle1, node1_cell.clone());
        rt.spawn(async {
            let res = transport1.run().await;
            info!("Transport done: {:?}", res);
        });
        handle1_tester.start_handle(&mut rt);

        // send 1 to 2, but 2 is not yet connected. It should queue
        handle1_tester.send(vec![node2.node().clone()], 1);

        // send 1 to 2, but with expired message, which shouldn't be delivered
        let msg_frame = TransportHandleTester::empty_message_frame();
        let msg = OutMessage::from_framed_message(&node1_cell, TransportLayer::Chain, msg_frame)?
            .with_expiration(Some(Instant::now() - Duration::from_secs(5)))
            .with_rendez_vous_id(ConsistentTimestamp(2))
            .with_to_nodes(vec![node2.node().clone()]);
        handle1_tester.send_message(msg);

        // leave some time for first messages to arrive
        std::thread::sleep(Duration::from_millis(100));

        // we create second node
        let mut transport2 = Libp2pTransport::new(node2.clone(), Libp2pTransportConfig::default());
        let handle2 = transport2.get_handle(node2_cell.cell().clone(), TransportLayer::Chain)?;
        let handle2_tester = TransportHandleTester::new(&mut rt, handle2, node2_cell);
        rt.spawn(async {
            let res = transport2.run().await;
            info!("Transport done: {:?}", res);
        });
        handle2_tester.start_handle(&mut rt);

        // leave some time to start listening and connect
        std::thread::sleep(Duration::from_millis(100));

        // send another message to force redial
        handle1_tester.send(vec![node2.node().clone()], 3);

        // should receive 1 & 3, but not 2 since it had expired
        expect_eventually(|| {
            handle2_tester.check_received_memo_message(1)
                && !handle2_tester.check_received_memo_message(2)
                && handle2_tester.check_received_memo_message(3)
        });

        Ok(())
    }

    struct TransportHandleTester {
        cell: FullCell,
        handle: Libp2pTransportHandle,
        sender: mpsc::UnboundedSender<OutEvent>,
        received: Arc<Mutex<Vec<InEvent>>>,
    }

    impl TransportHandleTester {
        fn new(
            rt: &mut Runtime,
            mut handle: Libp2pTransportHandle,
            cell: FullCell,
        ) -> TransportHandleTester {
            let (sender, receiver) = mpsc::unbounded();
            let mut sink = handle.get_sink();
            rt.spawn(async move {
                let mut receiver = receiver;
                while let Some(event) = receiver.next().await {
                    if let Err(err) = sink.send(event).await {
                        error!("Error sending to transport: {}", err);
                    }
                }
            });

            let received = Arc::new(Mutex::new(Vec::new()));
            let received_weak = Arc::downgrade(&received);
            let mut stream = handle.get_stream();
            rt.spawn(async move {
                while let Some(msg) = stream.next().await {
                    let received = received_weak.upgrade().unwrap();
                    let mut received = received.lock().unwrap();
                    received.push(msg);
                }
            });

            TransportHandleTester {
                cell,
                handle,
                sender,
                received,
            }
        }

        fn start_handle(&self, rt: &mut Runtime) {
            rt.block_on(self.handle.on_started());
        }

        fn empty_message_frame() -> CapnpFrameBuilder<block_operation_header::Owned> {
            let mut frame_builder = CapnpFrameBuilder::<block_operation_header::Owned>::new();
            let _ = frame_builder.get_builder();
            frame_builder
        }

        fn send(&self, to_nodes: Vec<Node>, memo: u64) {
            let frame_builder = Self::empty_message_frame();
            let msg =
                OutMessage::from_framed_message(&self.cell, TransportLayer::Chain, frame_builder)
                    .unwrap()
                    .with_rendez_vous_id(ConsistentTimestamp(memo))
                    .with_to_nodes(to_nodes);

            self.send_message(msg);
        }

        fn send_message(&self, message: OutMessage) {
            self.sender
                .unbounded_send(OutEvent::Message(message))
                .unwrap();
        }

        fn received_events(&self) -> Vec<InEvent> {
            let received = self.received.lock().unwrap();
            received.clone()
        }

        fn received_messages(&self) -> Vec<InEvent> {
            let received = self.received.lock().unwrap();
            received
                .iter()
                .filter(|event| match event {
                    InEvent::Message(_event) => true,
                    _ => false,
                })
                .cloned()
                .collect()
        }

        fn node_status(&self, node_id: &NodeId) -> ConnectionStatus {
            let received = self.received.lock().unwrap();
            let status = received
                .iter()
                .flat_map(|event| match event {
                    InEvent::NodeStatus(some_node_id, status) if some_node_id == node_id => {
                        Some(*status)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            status
                .last()
                .cloned()
                .unwrap_or_else(|| ConnectionStatus::Disconnected)
        }

        fn check_received_memo_message(&self, memo: u64) -> bool {
            self.receive_memo_message(memo).is_some()
        }

        fn receive_memo_message(&self, memo: u64) -> Option<Box<InMessage>> {
            let received = self.received_events();
            received
                .into_iter()
                .flat_map(|msg| match msg {
                    InEvent::Message(event) => {
                        if event.rendez_vous_id == Some(ConsistentTimestamp(memo)) {
                            Some(event)
                        } else {
                            None
                        }
                    }
                    InEvent::NodeStatus(_, _) => None,
                })
                .next()
        }
    }
}
