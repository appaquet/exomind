use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use futures::channel::mpsc;
use futures::prelude::*;
use futures::{FutureExt, SinkExt, StreamExt};
use libp2p::core::PeerId;
use libp2p::swarm::Swarm;

use super::{
    behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage, PeerStatus},
    handles::ServiceHandles,
    Libp2pTransportConfig,
};
use exocore_core::cell::{Cell, CellId, CellNodes};
use exocore_core::cell::{LocalNode, Node, NodeId};
use exocore_core::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_core::protos::generated::common_capnp::envelope;
use exocore_core::utils::handle_set::HandleSet;

use crate::transport::{ConnectionStatus, InEvent, OutEvent};
use crate::Error;
use crate::{messages::InMessage, Libp2pTransportServiceHandle};
use crate::{transport::ConnectionID, ServiceType};

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
            "Registering transport for cell {} and service_type {:?}",
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
        let behaviour = ExocoreBehaviour::new();

        #[cfg(all(feature = "p2p-web", target_arch = "wasm32"))]
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

        #[cfg(feature = "p2p-full")]
        let mut swarm = {
            let transport = libp2p::build_tcp_ws_noise_mplex_yamux(
                self.local_node.keypair().to_libp2p().clone(),
            )?;

            // Create our own libp2p executor since by default it spawns its own thread pool
            // to spawn tcp related futures, but Tokio requires to be spawn from
            // within its runtime.
            struct CoreExecutor;
            impl libp2p::core::Executor for CoreExecutor {
                fn exec(&self, f: std::pin::Pin<Box<dyn Future<Output = ()> + Send>>) {
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
            let inner = self.service_handles.read()?;
            for node in inner.all_peer_nodes().values() {
                swarm.add_node_peer(node);
            }
        }

        let mut nodes_update_interval =
            exocore_core::futures::interval(self.config.swarm_nodes_update_interval);

        // Spawn the main Future which will take care of the swarm
        let inner = Arc::clone(&self.service_handles);
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
        inner: &RwLock<ServiceHandles>,
        message: ExocoreBehaviourMessage,
    ) -> Result<(), Error> {
        let frame = TypedCapnpFrame::<_, envelope::Owned>::new(message.data)?;
        let frame_reader: envelope::Reader = frame.get_reader()?;
        let cell_id_bytes = frame_reader.get_cell_id()?;

        let mut inner = inner.write()?;

        let cell_id = CellId::from_bytes(&cell_id_bytes);
        let service_type = ServiceType::from_code(frame_reader.get_service()).ok_or_else(|| {
            Error::Other(format!(
                "Message has invalid service_type {}",
                frame_reader.get_service()
            ))
        })?;

        let key = (cell_id, service_type);
        let service_handle = if let Some(service_handle) = inner.service_handles.get_mut(&key) {
            service_handle
        } else {
            return Err(Error::Other(format!(
                "Couldn't find transport for service & cell {:?}",
                key
            )));
        };

        let source_node = Self::get_node_by_peer(&service_handle.cell, message.source)?;
        let mut msg = InMessage::from_node_and_frame(source_node, frame.to_owned())?;
        msg.connection = Some(ConnectionID::Libp2p(message.connection));

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
            if let Ok(node) = Self::get_node_by_peer(&handle.cell, peer_id.clone()) {
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
