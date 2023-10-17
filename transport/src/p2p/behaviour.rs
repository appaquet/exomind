use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use exocore_core::{cell::Node, time::Instant};
use futures::task::{Context, Poll};
use libp2p::{
    core::Multiaddr,
    swarm::{
        dial_opts::{DialOpts, PeerCondition},
        CloseConnection, ConnectionId, FromSwarm, NetworkBehaviour, NotifyHandler, PollParameters,
        THandler, THandlerInEvent, ToSwarm,
    },
    PeerId,
};

use super::protocol::{ExocoreProtoHandler, MessageData};

const MAX_PEER_QUEUE: usize = 20;
const DEFAULT_DIALING_MESSAGE_TIMEOUT: Duration = Duration::from_secs(5);

/// Libp2p's behaviour for Exocore transport.
///
/// This manages:
///   * Peers that we want to be connected to.
///   * Incoming messages from the protocol handler, to be dispatched via
///     Exocore's transport.
///   * Outgoing messages from Exocore's transport to be dispatched to the
///     protocol handler.
#[derive(Default)]
pub struct ExocoreBehaviour {
    actions: VecDeque<BehaviourAction>,
    peers: HashMap<PeerId, Peer>,
    last_redial_check: Option<Instant>,
}

type BehaviourAction = ToSwarm<ExocoreBehaviourEvent, THandlerInEvent<ExocoreBehaviour>>;

impl ExocoreBehaviour {
    pub fn send_message(
        &mut self,
        peer_id: PeerId,
        expiration: Option<Instant>,
        connection: Option<ConnectionId>,
        msg: MessageData,
    ) {
        let handler = if let Some(connection_id) = connection {
            NotifyHandler::One(connection_id)
        } else {
            NotifyHandler::Any
        };

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if peer.status == PeerStatus::Connected {
                let event = ToSwarm::NotifyHandler {
                    peer_id,
                    handler,
                    event: msg,
                };

                self.actions.push_back(event);
            } else {
                let expiration =
                    expiration.unwrap_or_else(|| Instant::now() + DEFAULT_DIALING_MESSAGE_TIMEOUT);

                debug!("Got new message for peer {}, but not connected. Queuing message while dialing.", peer.node);

                // Node is disconnected, push the event to a queue and try to connect
                peer.temp_queue.push_back(QueuedPeerEvent {
                    event: ToSwarm::NotifyHandler {
                        peer_id,
                        handler,
                        event: msg,
                    },
                    expiration: Some(expiration),
                });

                // make sure queue doesn't go higher than limit
                while peer.temp_queue.len() > MAX_PEER_QUEUE {
                    peer.temp_queue.pop_front();
                }

                self.dial_peer(peer_id, false);
            }
        }
    }

    pub fn add_node(&mut self, node: &Node) {
        let peer_id = *node.peer_id();
        let addresses = node.p2p_addresses();

        if let Some(current_peer) = self.peers.get_mut(&peer_id) {
            if current_peer.addresses == addresses {
                // we stop here if addresses match to prevent re-dialing again
                return;
            }

            // update peer addresses
            current_peer.addresses = addresses;
        } else {
            self.peers.insert(
                peer_id,
                Peer {
                    addresses,
                    node: node.clone(),
                    temp_queue: VecDeque::new(),
                    status: PeerStatus::Disconnected,
                    last_dial: None,
                },
            );
        }

        self.dial_peer(peer_id, true);
    }

    pub fn report_ping_success(&mut self, peer_id: &PeerId, rtt: Duration) {
        if let Some(peer) = self.peers.get(peer_id) {
            debug!("Successfully ping peer {}: {:?}", peer.node, rtt);
            self.mark_peer_connected(peer_id);
        }
    }

    pub fn reset_peers(&mut self) {
        for (peer_id, peer) in &mut self.peers {
            peer.last_dial = None;
            self.actions.push_back(ToSwarm::CloseConnection {
                peer_id: *peer_id,
                connection: CloseConnection::All,
            });
        }
    }

    fn dial_peer(&mut self, peer_id: PeerId, force_expire: bool) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if peer.addresses.is_empty() {
                return;
            }

            if peer.status != PeerStatus::Disconnected {
                return;
            }

            let dial_expired = peer
                .last_dial
                .map_or(true, |i| i.elapsed() > Duration::from_secs(30));
            if !dial_expired && !force_expire {
                return;
            }

            debug!(
                "Triggering dial of peer {} on addresses {:?}",
                peer.node, peer.addresses
            );
            peer.last_dial = Some(Instant::now());
            self.actions.push_back(ToSwarm::Dial {
                opts: DialOpts::peer_id(peer_id)
                    .condition(PeerCondition::NotDialing)
                    .build(),
            });
        }
    }

    fn mark_peer_connected(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            if peer.status == PeerStatus::Connected {
                return;
            }

            info!("Connected to peer {}", peer.node);
            peer.status = PeerStatus::Connected;
            self.actions
                .push_back(ToSwarm::GenerateEvent(ExocoreBehaviourEvent::PeerStatus(
                    *peer_id,
                    peer.status,
                )));

            // send any messages that were queued while node was disconnected, but that
            // haven't expired
            while let Some(event) = peer.temp_queue.pop_front() {
                if !event.has_expired() {
                    self.actions.push_back(event.event);
                }
            }
        } else {
            warn!("Got connection from unknown peer {}", peer_id);
        }
    }

    fn handle_connection_closed(
        &mut self,
        event: libp2p::swarm::ConnectionClosed<ExocoreProtoHandler>,
    ) {
        let peer_id = event.peer_id;
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            info!("Disconnected from peer {}", peer.node);

            peer.status = PeerStatus::Disconnected;
            self.actions
                .push_back(ToSwarm::GenerateEvent(ExocoreBehaviourEvent::PeerStatus(
                    peer_id,
                    peer.status,
                )));

            // cleanup old messages
            peer.cleanup_expired();

            // trigger reconnection
            self.dial_peer(peer_id, true);
        }
    }

    fn handle_dial_failure(&mut self, event: libp2p::swarm::DialFailure) {
        if let Some(peer) = event
            .peer_id
            .and_then(|peer_id| self.peers.get_mut(&peer_id))
        {
            info!(
                "Failed to connect to peer {}: {:?}. {} messages in queue for node.",
                peer.node,
                event.error,
                peer.temp_queue.len()
            );
        }
    }
}

impl NetworkBehaviour for ExocoreBehaviour {
    type ConnectionHandler = ExocoreProtoHandler;
    type ToSwarm = ExocoreBehaviourEvent;

    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        peer: PeerId,
        _local_addr: &Multiaddr,
        _remote_addr: &Multiaddr,
    ) -> Result<THandler<Self>, libp2p::swarm::ConnectionDenied> {
        self.mark_peer_connected(&peer);
        Ok(ExocoreProtoHandler::default())
    }

    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        peer: PeerId,
        _addr: &Multiaddr,
        _role_override: libp2p::core::Endpoint,
    ) -> Result<THandler<Self>, libp2p::swarm::ConnectionDenied> {
        self.mark_peer_connected(&peer);
        Ok(ExocoreProtoHandler::default())
    }

    fn handle_pending_inbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        _local_addr: &Multiaddr,
        _remote_addr: &Multiaddr,
    ) -> Result<(), libp2p::swarm::ConnectionDenied> {
        Ok(())
    }

    fn handle_pending_outbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        peer_id: Option<PeerId>,
        _addresses: &[Multiaddr],
        _effective_role: libp2p::core::Endpoint,
    ) -> Result<Vec<Multiaddr>, libp2p::swarm::ConnectionDenied> {
        let peer_id = peer_id.as_ref().ok_or_else(|| {
            libp2p::swarm::ConnectionDenied::new(crate::Error::Other(
                "cannot open outbound connection without peer id".to_string(),
            ))
        })?;

        let addrs = self
            .peers
            .get(peer_id)
            .map(|p| p.addresses.clone())
            .unwrap_or_default();

        Ok(addrs)
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        match event {
            FromSwarm::ConnectionClosed(event) => {
                self.handle_connection_closed(event);
            }
            FromSwarm::DialFailure(event) => {
                self.handle_dial_failure(event);
            }
            FromSwarm::ListenFailure(event) => {
                error!("Listen failure: {err}", err = event.error);
            }
            FromSwarm::ListenerError(event) => {
                error!("Listener error: {err}", err = event.err);
            }
            _ => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection: ConnectionId,
        event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            trace!("Received message from {}", peer.node);

            self.actions
                .push_back(ToSwarm::GenerateEvent(ExocoreBehaviourEvent::Message(
                    ExocoreBehaviourMessage {
                        source: peer_id,
                        connection,
                        message: event,
                    },
                )));
        }
    }

    fn poll(
        &mut self,
        _cx: &mut Context<'_>,
        _params: &mut impl PollParameters,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        // check if we could try to dial to disconnected nodes
        let redial_check = self
            .last_redial_check
            .map_or(true, |i| i.elapsed() > Duration::from_secs(5));
        if redial_check {
            self.last_redial_check = Some(Instant::now());
            let peer_ids: Vec<PeerId> = self.peers.keys().cloned().collect();
            for peer_id in peer_ids {
                self.dial_peer(peer_id, false);
            }
        }

        if let Some(event) = self.actions.pop_front() {
            return Poll::Ready(event);
        }

        Poll::Pending
    }
}

/// Peer that the behaviour connects with, or may be connecting too.
/// The behaviour manages messages sent and received to these peers.
struct Peer {
    addresses: Vec<Multiaddr>,
    node: Node,
    temp_queue: VecDeque<QueuedPeerEvent>,
    status: PeerStatus,
    last_dial: Option<Instant>,
}

impl Peer {
    fn cleanup_expired(&mut self) {
        if !self.temp_queue.is_empty() {
            let mut old_queue = VecDeque::new();
            std::mem::swap(&mut self.temp_queue, &mut old_queue);

            for event in old_queue {
                if !event.has_expired() {
                    self.temp_queue.push_back(event)
                }
            }
        }
    }
}

/// Queued events to be sent to a peer that may not be connected yet.
/// It may get discarded if it reaches expiration before the peer gets
/// connected.
struct QueuedPeerEvent {
    event: BehaviourAction,
    expiration: Option<Instant>,
}

impl QueuedPeerEvent {
    fn has_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            expiration <= Instant::now()
        } else {
            false
        }
    }
}

/// Event emitted by the ExocoreBehaviour (ex: incoming message), consumed by
/// `Libp2pTransport`.
#[derive(Debug)]
pub enum ExocoreBehaviourEvent {
    Message(ExocoreBehaviourMessage),
    PeerStatus(PeerId, PeerStatus),
}

#[derive(Debug)]
pub struct ExocoreBehaviourMessage {
    pub source: PeerId,
    pub connection: ConnectionId,
    pub message: MessageData,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PeerStatus {
    Connected,
    Disconnected,
}
