use std::collections::{HashMap, VecDeque};

use futures::task::{Context, Poll};
use libp2p::core::{ConnectedPoint, Multiaddr, PeerId};
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};

use exocore_core::time::Instant;

use crate::lp2p::protocol::{ExocoreProtoHandler, ExocoreProtoMessage};
use std::time::Duration;

const MAX_PEER_QUEUE: usize = 20;
const DEFAULT_DIALING_MESSAGE_TIMEOUT: Duration = Duration::from_secs(5);

/// Libp2p's behaviour for Exocore transport.
///
/// This manages:
///   * Peers that we want to be connected to.
///   * Incoming messages from the protocol handler, to be dispatched via Exocore's transport.
///   * Outgoing messages from Exocore's transport to be dispatched to the protocol handler.
pub struct ExocoreBehaviour {
    local_node: PeerId,
    actions: VecDeque<BehaviourAction>,
    peers: HashMap<PeerId, Peer>,
}

type BehaviourAction = NetworkBehaviourAction<ExocoreProtoMessage, ExocoreBehaviourEvent>;

impl ExocoreBehaviour {
    pub fn new() -> ExocoreBehaviour {
        ExocoreBehaviour {
            local_node: PeerId::random(),
            actions: VecDeque::new(),
            peers: HashMap::new(),
        }
    }

    pub fn send_message(&mut self, peer_id: PeerId, expiration: Option<Instant>, data: Vec<u8>) {
        let event = NetworkBehaviourAction::SendEvent {
            peer_id: peer_id.clone(),
            event: ExocoreProtoMessage { data },
        };

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if peer.status == PeerStatus::Connected {
                self.actions.push_back(event);
            } else {
                let expiration =
                    expiration.unwrap_or_else(|| Instant::now() + DEFAULT_DIALING_MESSAGE_TIMEOUT);

                debug!("Peer {} not connected. Queuing message.", peer_id);
                // Node is disconnected, push the event to a queue and try to connect
                peer.temp_queue.push_back(QueuedPeerEvent {
                    event,
                    expiration: Some(expiration),
                });

                // make sure queue doesn't go higher than limit
                while peer.temp_queue.len() > MAX_PEER_QUEUE {
                    peer.temp_queue.pop_front();
                }

                self.dial_peer(peer_id);
            }
        }
    }

    pub fn add_peer(&mut self, peer_id: PeerId, addresses: Vec<Multiaddr>) {
        if let Some(current_peer) = self.peers.get(&peer_id) {
            if current_peer.addresses == addresses {
                // no need to update, peer already exist with same addr
                return;
            }
        }

        self.peers.insert(
            peer_id.clone(),
            Peer {
                addresses,
                temp_queue: VecDeque::new(),
                status: PeerStatus::Disconnected,
            },
        );

        self.dial_peer(peer_id);
    }

    fn dial_peer(&mut self, peer_id: PeerId) {
        self.actions
            .push_back(NetworkBehaviourAction::DialPeer { peer_id });
    }
}

impl Default for ExocoreBehaviour {
    fn default() -> Self {
        ExocoreBehaviour::new()
    }
}

impl NetworkBehaviour for ExocoreBehaviour {
    type ProtocolsHandler = ExocoreProtoHandler;
    type OutEvent = ExocoreBehaviourEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        // We use OneShot protocol handler that opens a new stream for every message (stream, not connection)
        Default::default()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.peers
            .get(peer_id)
            .map(|p| p.addresses.clone())
            .unwrap_or_else(Vec::new)
    }

    fn inject_connected(&mut self, peer_id: PeerId, _endpoint: ConnectedPoint) {
        debug!("{}: Connected to {}", self.local_node, peer_id,);

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.status = PeerStatus::Connected;

            self.actions
                .push_back(NetworkBehaviourAction::GenerateEvent(
                    ExocoreBehaviourEvent::PeerStatus(peer_id.clone(), peer.status),
                ));

            // send any messages that were queued while node was disconnected, but that haven't expired
            while let Some(event) = peer.temp_queue.pop_front() {
                if !event.has_expired() {
                    self.actions.push_back(event.event);
                }
            }
        }
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _endpoint: ConnectedPoint) {
        debug!("{}: Disconnected from {}", self.local_node, peer_id,);

        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.status = PeerStatus::Disconnected;

            self.actions
                .push_back(NetworkBehaviourAction::GenerateEvent(
                    ExocoreBehaviourEvent::PeerStatus(peer_id.clone(), peer.status),
                ));

            // check if we need to reconnect
            peer.cleanup_expired();
            if !peer.temp_queue.is_empty() {
                self.dial_peer(peer_id.clone());
            }
        }
    }

    fn inject_node_event(&mut self, peer_id: PeerId, msg: ExocoreProtoMessage) {
        trace!("{}: Received message from {}", self.local_node, peer_id);

        self.actions
            .push_back(NetworkBehaviourAction::GenerateEvent(
                ExocoreBehaviourEvent::Message(ExocoreBehaviourMessage {
                    source: peer_id,
                    data: msg.data,
                }),
            ));
    }

    fn inject_dial_failure(&mut self, peer_id: &PeerId) {
        debug!("{}: Failed to connect to {}", self.local_node, peer_id);
    }

    fn poll(
        &mut self,
        _ctx: &mut Context,
        _params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<ExocoreProtoMessage, ExocoreBehaviourEvent>> {
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
    temp_queue: VecDeque<QueuedPeerEvent>,
    status: PeerStatus,
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
/// It may get discarded if it reaches expiration before the peer gets connected.
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

/// Event emitted by the ExocoreBehaviour (ex: incoming message), consumed by `Libp2pTransport`.
pub enum ExocoreBehaviourEvent {
    Message(ExocoreBehaviourMessage),
    PeerStatus(PeerId, PeerStatus),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExocoreBehaviourMessage {
    pub source: PeerId,
    pub data: Vec<u8>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PeerStatus {
    Connected,
    Disconnected,
}
