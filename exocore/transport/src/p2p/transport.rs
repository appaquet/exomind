use std::{
    num::NonZeroU8,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use exocore_core::{
    cell::{Cell, CellId, CellNodes, LocalNode, Node, NodeId},
    framing::{FrameBuilder, TypedCapnpFrame},
    utils::handle_set::HandleSet,
};
use exocore_protos::generated::common_capnp::envelope;
use futures::{channel::mpsc, prelude::*, FutureExt, SinkExt, StreamExt};
use libp2p::{
    ping,
    swarm::{NetworkBehaviour, Swarm},
    PeerId, Transport,
};

use super::{
    behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage, PeerStatus},
    handles::ServiceHandles,
    Libp2pTransportConfig,
};
use crate::{
    messages::InMessage,
    p2p::protocol::MessageData,
    transport::{ConnectionId, ConnectionStatus, InEvent, OutEvent},
    Error, Libp2pTransportServiceHandle, ServiceType,
};

/// Libp2p transport used by all services of Exocore through handles. There is
/// one handle per cell per service.
///
/// The transport itself is scheduled on an Executor, and its future will
/// complete as soon it's ready. Once all handles are dropped, all its scheduled
/// tasks will be stopped too.
pub struct Libp2pTransport {
    local_node: LocalNode,
    config: Libp2pTransportConfig,
    service_handles: Arc<RwLock<ServiceHandles>>,
    handle_set: HandleSet,
}

impl Libp2pTransport {
    /// Creates a new transport for given node and config. The node is important
    /// here since all messages are authenticated using the node's private
    /// key thanks to secio.
    pub fn new(local_node: LocalNode, config: Libp2pTransportConfig) -> Libp2pTransport {
        Libp2pTransport {
            local_node,
            config,
            service_handles: Default::default(),
            handle_set: Default::default(),
        }
    }

    /// Creates sink and streams that can be used for a given service of a cell.
    pub fn get_handle(
        &mut self,
        cell: Cell,
        service_type: ServiceType,
    ) -> Result<Libp2pTransportServiceHandle, Error> {
        let (in_sender, in_receiver) = mpsc::channel(self.config.handle_in_channel_size);
        let (out_sender, out_receiver) = mpsc::channel(self.config.handle_out_channel_size);

        // Register new handle and its streams
        let mut handles = self.service_handles.write()?;
        handles.push_handle(cell.clone(), service_type, in_sender, out_receiver);

        info!(
            "Registering transport for cell {} and service type {:?}",
            cell, service_type
        );

        Ok(Libp2pTransportServiceHandle {
            cell_id: cell.id().clone(),
            service_type,
            inner: Arc::downgrade(&self.service_handles),
            sink: Some(out_sender),
            stream: Some(in_receiver),
            handle: self.handle_set.get_handle(),
        })
    }

    #[cfg(test)]
    pub(super) fn get_service_handles(&self) -> Arc<RwLock<ServiceHandles>> {
        self.service_handles.clone()
    }

    /// Runs the transport to completion.
    pub async fn run(self) -> Result<(), Error> {
        let behaviour = CombinedBehaviour {
            // service_handles: Arc::clone(&self.service_handles),
            exocore: ExocoreBehaviour::default(),
            ping: ping::Behaviour::default(),
        };

        const DIAL_CONCURRENCY_FACTOR: u8 = 5;

        #[cfg(all(feature = "p2p-web", target_arch = "wasm32"))]
        let mut swarm = {
            use libp2p::websocket_websys::Transport;

            let keypair = self.local_node.keypair().to_libp2p();

            let transport = Transport::default()
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(
                    libp2p::noise::Config::new(keypair)
                        .expect("Couldn't build noise authentication"),
                )
                .multiplex(libp2p::core::upgrade::SelectUpgrade::new(
                    libp2p::yamux::Config::default(),
                    libp2p_mplex::MplexConfig::new(),
                ))
                .map(|(peer, muxer), _| (peer, libp2p::core::muxing::StreamMuxerBox::new(muxer)))
                .boxed();

            let config = libp2p::swarm::Config::with_wasm_executor()
                .with_dial_concurrency_factor(NonZeroU8::new(DIAL_CONCURRENCY_FACTOR).unwrap());

            libp2p::swarm::Swarm::new(transport, behaviour, *self.local_node.peer_id(), config)
        };

        #[cfg(feature = "p2p-full")]
        let mut swarm = {
            let transport = build_transport(self.local_node.keypair().to_libp2p().clone())?;

            // Create our own libp2p executor since by default it spawns its own thread pool
            // to spawn tcp related futures, but Tokio requires to be spawn from
            // within its runtime.
            struct CoreExecutor;
            impl libp2p::swarm::Executor for CoreExecutor {
                fn exec(&self, f: std::pin::Pin<Box<dyn Future<Output = ()> + Send>>) {
                    exocore_core::futures::spawn_future(f)
                }
            }

            let config = libp2p::swarm::Config::with_executor(CoreExecutor)
                .with_dial_concurrency_factor(NonZeroU8::new(DIAL_CONCURRENCY_FACTOR).unwrap());

            libp2p::swarm::Swarm::new(transport, behaviour, *self.local_node.peer_id(), config)
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
            let inner = self.service_handles.read()?;
            for node in inner.all_peer_nodes().values() {
                swarm.behaviour_mut().exocore.add_node(node);
            }
        }

        let mut nodes_update_interval =
            exocore_core::futures::interval(self.config.swarm_nodes_update_interval);

        // Spawn the main Future which will take care of the swarm
        let service_handles = Arc::clone(&self.service_handles);
        let inner = service_handles.clone();
        let swarm_task = future::poll_fn(move |cx: &mut Context| -> Poll<()> {
            // At interval, re-add all nodes to make sure that their newer addresses are
            // added.
            if nodes_update_interval.poll_tick(cx).is_ready() {
                let inner = inner.read().expect("Couldn't get inner lock");
                for node in inner.all_peer_nodes().values() {
                    swarm.behaviour_mut().exocore.add_node(node);
                }
            }

            // Drain all messages coming from handles that need to be sent to other nodes
            while let Poll::Ready(Some(event)) = out_receiver.poll_next_unpin(cx) {
                match event {
                    OutEvent::Message(msg) => {
                        let frame_data = msg.envelope_builder.as_bytes();

                        let connection =
                            if let Some(ConnectionId::Libp2p(connection)) = msg.connection {
                                Some(connection)
                            } else {
                                None
                            };

                        if let Some(dest) = msg.destination {
                            let msg_data = MessageData {
                                message: frame_data,
                                stream: msg.stream,
                            };

                            swarm.behaviour_mut().exocore.send_message(
                                *dest.peer_id(),
                                msg.expiration,
                                connection,
                                msg_data,
                            );
                        } else {
                            error!("Got a message to send to behaviour without destination node");
                        }
                    }
                    OutEvent::Reset => {
                        info!("Resetting connections to peers...");
                        swarm.behaviour_mut().exocore.reset_peers();
                    }
                }
            }

            // Poll the swarm to complete its job
            while let Poll::Ready(event) = swarm.poll_next_unpin(cx) {
                let Some(event) = event else {
                    continue;
                };

                match event {
                    libp2p::swarm::SwarmEvent::Behaviour(CombinedEvent::Exocore(
                        ExocoreBehaviourEvent::Message(msg),
                    )) => {
                        trace!("Got message from {}", msg.source);

                        if let Err(err) = dispatch_message(&service_handles, msg) {
                            warn!("Couldn't dispatch message: {}", err);
                        }
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(CombinedEvent::Exocore(
                        ExocoreBehaviourEvent::PeerStatus(peer_id, status),
                    )) => {
                        if let Err(err) = dispatch_node_status(&service_handles, peer_id, status) {
                            warn!("Couldn't dispatch node status: {}", err);
                        }
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(CombinedEvent::Ping(event)) => {
                        match event.result {
                            Ok(rtt) => {
                                // TODO: We could save round-trip time to node. Could be use for
                                // node selection.
                                swarm
                                    .behaviour_mut()
                                    .exocore
                                    .report_ping_success(&event.peer, rtt)
                            }
                            Err(failure) => {
                                debug!("Failed to ping peer {}: {}", event.peer, failure);
                            }
                        }
                    }
                    _ => {}
                }
            }

            Poll::Pending
        });

        // Sends handles' outgoing messages to the behaviour's input channel
        let handles_dispatcher = {
            let mut inner = self.service_handles.write()?;
            let mut futures = Vec::new();
            for service_handle in inner.service_handles.values_mut() {
                let out_receiver = service_handle
                    .out_receiver
                    .take()
                    .expect("Out receiver of one service handle was already consumed");

                let mut out_sender = out_sender.clone();
                futures.push(async move {
                    let mut out_receiver = out_receiver;
                    while let Some(event) = out_receiver.next().await {
                        let _ = out_sender.send(event).await;
                    }
                    error!("Handle out receiver has returned none.");
                });
            }
            futures::future::join_all(futures)
        };

        info!("Libp2p transport now running");
        futures::select! {
            _ = swarm_task.fuse() => {},
            _ = handles_dispatcher.fuse() => {},
            _ = self.handle_set.on_handles_dropped().fuse() => {},
        };
        info!("Libp2p transport is done");

        Ok(())
    }
}

/// Behaviour that combines exocore and ping behaviours.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "CombinedEvent")]
struct CombinedBehaviour {
    exocore: ExocoreBehaviour,
    ping: ping::Behaviour,
}

enum CombinedEvent {
    Exocore(ExocoreBehaviourEvent),
    Ping(ping::Event),
}

impl From<ExocoreBehaviourEvent> for CombinedEvent {
    fn from(event: ExocoreBehaviourEvent) -> Self {
        CombinedEvent::Exocore(event)
    }
}

impl From<ping::Event> for CombinedEvent {
    fn from(event: ping::Event) -> Self {
        CombinedEvent::Ping(event)
    }
}

/// Dispatches a received message from libp2p to corresponding handle
fn dispatch_message(
    inner: &RwLock<ServiceHandles>,
    message: ExocoreBehaviourMessage,
) -> Result<(), Error> {
    let frame = TypedCapnpFrame::<_, envelope::Owned>::new(message.message.message)?;
    let frame_reader: envelope::Reader = frame.get_reader()?;
    let cell_id_bytes = frame_reader.get_cell_id()?;

    let mut inner = inner.write()?;

    let cell_id = CellId::from_bytes(cell_id_bytes);
    let service_type = ServiceType::from_code(frame_reader.get_service()).ok_or_else(|| {
        Error::Other(format!(
            "Message has invalid service_type {}",
            frame_reader.get_service()
        ))
    })?;

    let key = (cell_id, service_type);
    let Some(service_handle) = inner.service_handles.get_mut(&key) else {
        return Err(Error::Other(format!(
            "Couldn't find transport for service & cell {:?}",
            key
        )));
    };

    let source_node = get_node_by_peer(&service_handle.cell, message.source)?;
    let mut msg = InMessage::from_node_and_frame(source_node, frame.to_owned())?;
    msg.connection = Some(ConnectionId::Libp2p(message.connection));
    msg.stream = message.message.stream;

    service_handle
        .in_sender
        .try_send(InEvent::Message(msg))
        .map_err(|err| Error::Other(format!("Couldn't send message to cell service: {}", err)))
}

/// Dispatches a node status change.
fn dispatch_node_status(
    inner: &RwLock<ServiceHandles>,
    peer_id: PeerId,
    peer_status: PeerStatus,
) -> Result<(), Error> {
    let mut inner = inner.write()?;

    let status = match peer_status {
        PeerStatus::Connected => ConnectionStatus::Connected,
        PeerStatus::Disconnected => ConnectionStatus::Disconnected,
    };

    for handle in inner.service_handles.values_mut() {
        if let Ok(node) = get_node_by_peer(&handle.cell, peer_id) {
            handle
                .in_sender
                .try_send(InEvent::NodeStatus(node.id().clone(), status))
                .map_err(|err| {
                    Error::Other(format!("Couldn't send message to cell service: {}", err))
                })?;
        }
    }

    Ok(())
}

/// Returns the node of a cell that has the given libp2p peer id.
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

/// Create a transport to be used in non-wasm target.
/// Copied from libp2p to force using Google DNS for iOS support.
#[cfg(feature = "p2p-full")]
pub fn build_transport(
    keypair: libp2p::identity::Keypair,
) -> std::io::Result<libp2p::core::transport::Boxed<(PeerId, libp2p::core::muxing::StreamMuxerBox)>>
{
    let transport = {
        let dns_tcp = || {
            libp2p::dns::tokio::Transport::custom(
                libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::new().nodelay(true)),
                libp2p::dns::ResolverConfig::google(),
                Default::default(),
            )
        };
        let ws_dns_tcp = libp2p::websocket::WsConfig::new(dns_tcp());
        ws_dns_tcp.or_transport(dns_tcp())
    };

    Ok(transport
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(
            libp2p::noise::Config::new(&keypair).expect("Couldn't build noise authentication"),
        )
        .multiplex(libp2p::core::upgrade::SelectUpgrade::new(
            libp2p::yamux::Config::default(),
            libp2p_mplex::MplexConfig::default(),
        ))
        .timeout(std::time::Duration::from_secs(20))
        .boxed())
}
