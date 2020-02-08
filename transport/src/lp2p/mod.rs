pub mod behaviour;
pub mod protocol;

use crate::messages::InMessage;
use crate::transport::{InEvent, OutEvent, TransportHandleOnStart};
use crate::Error;
use crate::{TransportHandle, TransportLayer};
use behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage};
use exocore_common::cell::{Cell, CellId, CellNodes};
use exocore_common::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_common::node::{LocalNode, NodeId};
use exocore_common::protos::generated::common_capnp::envelope;
use exocore_common::utils::handle_set::{Handle, HandleSet};
use futures::channel::mpsc;
use futures::channel::mpsc::SendError;
use futures::compat::Future01CompatExt;
use futures::future::Future as Future03;
use futures::sink::SinkMapErr;
use futures::{FutureExt, SinkExt, StreamExt, TryStreamExt};
use futures01::stream::Stream as Stream01;
use futures01::Async as Async01;
use libp2p::core::{Multiaddr, PeerId};
use libp2p::swarm::Swarm;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll};
use std::time::Duration;

/// libp2p transport configuration
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
    handles: Arc<RwLock<Handles>>,
    handle_set: HandleSet,
}

struct Handles {
    handles: HashMap<(CellId, TransportLayer), HandleChannels>,
}

impl Handles {
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
    }
}

struct HandleChannels {
    cell: Cell,
    in_sender: mpsc::Sender<InEvent>,
    out_receiver: Option<mpsc::Receiver<OutEvent>>,
}

impl Libp2pTransport {
    /// Creates a new transport for given node and config. The node is important here
    /// since all messages are authenticated using the node's private key thanks to secio
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
            cell.id(),
            layer
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
        let local_keypair = self.local_node.keypair().clone();
        let transport = libp2p::build_tcp_ws_secio_mplex_yamux(local_keypair.to_libp2p().clone());

        let behaviour = ExocoreBehaviour::new();
        let mut swarm = Swarm::new(transport, behaviour, self.local_node.peer_id().clone());

        let listen_address = self.config.listen_address(&self.local_node)?;
        Swarm::listen_on(&mut swarm, listen_address)?;

        // Spawn the swarm & receive message from a channel through which outgoing messages will go
        let (out_sender, out_receiver) =
            mpsc::channel::<OutEvent>(self.config.handles_to_behaviour_channel_size);

        // Add initial nodes to swarm
        {
            let inner = self.handles.read()?;
            for (peer_id, addresses) in inner.all_peers() {
                swarm.add_peer(peer_id, addresses);
            }
        }

        // Spawn the main Future which will take care of the swarm
        let inner = Arc::clone(&self.handles);
        let mut nodes_update_interval =
            exocore_common::futures::interval(self.config.swarm_nodes_update_interval)
                .map(Ok::<_, ()>)
                .compat();
        let mut out_receiver = out_receiver.map(Ok::<_, ()>).compat();
        let swarm_task = futures01::future::poll_fn(move || -> Result<Async01<()>, ()> {
            // at interval, we update peers that we should be connected to
            if let Async01::Ready(_) = nodes_update_interval
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
            while let Async01::Ready(Some(event)) = out_receiver
                .poll()
                .expect("Couldn't poll behaviour channel")
            {
                match event {
                    OutEvent::Message(msg) => {
                        let frame_data = msg.envelope_builder.as_bytes();

                        // prevent cloning frame if we only send to 1 node
                        if msg.to.len() == 1 {
                            let to_node = msg.to.first().unwrap();
                            swarm.send_message(
                                to_node.peer_id().clone(),
                                msg.expiration,
                                frame_data,
                            );
                        } else {
                            for to_node in msg.to {
                                swarm.send_message(
                                    to_node.peer_id().clone(),
                                    msg.expiration,
                                    frame_data.clone(),
                                );
                            }
                        }
                    }
                }
            }

            // we poll the behaviour for incoming messages to be dispatched to handles
            while let Async01::Ready(Some(data)) = swarm.poll().expect("Couldn't poll swarm") {
                match data {
                    ExocoreBehaviourEvent::Message(msg) => {
                        trace!("Got message from {}", msg.source);

                        if let Err(err) = Self::dispatch_message(&inner, msg) {
                            warn!("Couldn't dispatch message: {}", err);
                        }
                    }
                }
            }

            Ok(Async01::NotReady)
        })
        .compat();

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
            .try_send(InEvent::Message(msg))
            .map_err(|err| Error::Other(format!("Couldn't send message to cell layer: {}", err)))
    }
}

/// Handle taken by a Cell layer to receive and send message for a given node & cell.
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

impl Future03 for Libp2pTransportHandle {
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
    use super::*;
    use crate::OutMessage;
    use exocore_common::cell::FullCell;
    use exocore_common::framing::CapnpFrameBuilder;
    use exocore_common::futures::Runtime;
    use exocore_common::node::Node;
    use exocore_common::protos::generated::data_chain_capnp::block_operation_header;
    use exocore_common::tests_utils::expect_eventually;
    use exocore_common::time::{ConsistentTimestamp, Instant};
    use futures::{SinkExt, StreamExt};
    use std::sync::Mutex;

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
        let handle1_tester = TransportHandleTester::new(&mut rt, handle1, node1_cell);
        rt.spawn_std(async {
            let res = transport1.run().await;
            info!("Transport done: {:?}", res);
        });
        handle1_tester.start_handle(&mut rt);

        let mut transport2 = Libp2pTransport::new(node2.clone(), Libp2pTransportConfig::default());
        let handle2 = transport2.get_handle(node2_cell.cell().clone(), TransportLayer::Data)?;
        let handle2_tester = TransportHandleTester::new(&mut rt, handle2, node2_cell);
        rt.spawn_std(async {
            let res = transport2.run().await;
            info!("Transport done: {:?}", res);
        });
        handle2_tester.start_handle(&mut rt);

        // give time for nodes to connect to each others
        std::thread::sleep(Duration::from_millis(100));

        // send 1 to 2
        handle1_tester.send(vec![node2.node().clone()], 123);
        expect_eventually(|| handle2_tester.check_received(123));

        // send 2 to 1 by duplicating node, should expect receiving 2 messages
        handle2_tester.send(vec![node1.node().clone(), node1.node().clone()], 234);
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
        assert_eq!(addr2, config.listen_address(&node1)?);

        // fallback to node if not specified in config
        let node1 = LocalNode::generate();
        node1.add_address(addr1.clone());
        let config = Libp2pTransportConfig::default();
        assert_eq!(addr1, config.listen_address(&node1)?);

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
        let node2_cell = FullCell::generate(node2);

        let mut transport = Libp2pTransport::new(node1, Libp2pTransportConfig::default());
        let inner_weak = Arc::downgrade(&transport.handles);

        // we create 2 handles
        let handle1 = transport.get_handle(node1_cell.cell().clone(), TransportLayer::Data)?;
        let handle1_tester = TransportHandleTester::new(&mut rt, handle1, node1_cell);

        let handle2 = transport.get_handle(node2_cell.cell().clone(), TransportLayer::Data)?;
        let handle2_tester = TransportHandleTester::new(&mut rt, handle2, node2_cell);

        rt.spawn_std(async {
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

        // we drop second handle, we expect inner to be dropped and therefor transport killed
        drop(handle2_tester);
        expect_eventually(|| inner_weak.upgrade().is_none());

        Ok(())
    }

    #[test]
    fn should_queue_message_until_connected() -> Result<(), failure::Error> {
        let mut rt = Runtime::new()?;

        let node1 = LocalNode::generate();
        node1.add_address("/ip4/127.0.0.1/tcp/3005".parse().unwrap());
        let node1_cell = FullCell::generate(node1.clone());

        let node2 = LocalNode::generate();
        node2.add_address("/ip4/127.0.0.1/tcp/3006".parse().unwrap());
        let node2_cell = node1_cell.clone_for_local_node(node2.clone());

        node1_cell.nodes_mut().add(node2.node().clone());
        node2_cell.nodes_mut().add(node1.node().clone());

        let mut transport1 = Libp2pTransport::new(node1, Libp2pTransportConfig::default());
        let handle1 = transport1.get_handle(node1_cell.cell().clone(), TransportLayer::Data)?;
        let handle1_tester = TransportHandleTester::new(&mut rt, handle1, node1_cell.clone());
        rt.spawn_std(async {
            let res = transport1.run().await;
            info!("Transport done: {:?}", res);
        });
        handle1_tester.start_handle(&mut rt);

        // send 1 to 2, but 2 is not yet connected. It should queue
        handle1_tester.send(vec![node2.node().clone()], 1);

        // send 1 to 2, but with expired message, which shouldn't be delivered
        let mut frame_builder = CapnpFrameBuilder::<block_operation_header::Owned>::new();
        let _builder = frame_builder.get_builder();
        let msg = OutMessage::from_framed_message(&node1_cell, TransportLayer::Data, frame_builder)
            .unwrap()
            .with_expiration(Some(Instant::now() - Duration::from_secs(5)))
            .with_rendez_vous_id(ConsistentTimestamp(2))
            .with_to_nodes(vec![node2.node().clone()]);
        handle1_tester.send_message(msg);

        // leave some time for first messages to arrive
        std::thread::sleep(Duration::from_millis(100));

        // we create second node
        let mut transport2 = Libp2pTransport::new(node2.clone(), Libp2pTransportConfig::default());
        let handle2 = transport2.get_handle(node2_cell.cell().clone(), TransportLayer::Data)?;
        let handle2_tester = TransportHandleTester::new(&mut rt, handle2, node2_cell);
        rt.spawn_std(async {
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
            handle2_tester.check_received(1)
                && !handle2_tester.check_received(2)
                && handle2_tester.check_received(3)
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
            rt.spawn_std(async move {
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
            rt.spawn_std(async move {
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
            rt.block_on_std(self.handle.on_started());
        }

        fn send(&self, to_nodes: Vec<Node>, memo: u64) {
            let mut frame_builder = CapnpFrameBuilder::<block_operation_header::Owned>::new();
            let _builder = frame_builder.get_builder();
            let msg =
                OutMessage::from_framed_message(&self.cell, TransportLayer::Data, frame_builder)
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

        fn received(&self) -> Vec<InEvent> {
            let received = self.received.lock().unwrap();
            received.clone()
        }

        fn check_received(&self, memo: u64) -> bool {
            let received = self.received();
            received.iter().any(|msg| match msg {
                InEvent::Message(event) => event.rendez_vous_id == Some(ConsistentTimestamp(memo)),
                InEvent::NodeStatus(_, _) => false,
            })
        }
    }
}
