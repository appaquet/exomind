use futures::prelude::*;
use libp2p_core::{ConnectedPoint, Multiaddr, PeerId};
use libp2p_swarm::{
    NetworkBehaviour, NetworkBehaviourAction, OneShotHandler, PollParameters, ProtocolsHandler,
};
use std::collections::{HashMap, VecDeque};
use tokio::io::{AsyncRead, AsyncWrite};

use super::protocol::{ExocoreProtoConfig, WireMessage};
use exocore_common::time::Instant;

const MAX_PEER_QUEUE: usize = 20;

///
/// Libp2p's behaviour for Exocore. The behaviour defines a protocol that is exposed to
/// lp2p, peers that we want to talk to and acts as a stream / sink of messages exchanged
/// between nodes.
///
pub struct ExocoreBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    local_node: PeerId,
    events: VecDeque<BehaviourEvent>,
    peers: HashMap<PeerId, Peer>,
    phantom: std::marker::PhantomData<TSubstream>,
}

type BehaviourEvent = NetworkBehaviourAction<WireMessage, ExocoreBehaviourEvent>;

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

#[derive(Copy, Clone, PartialEq)]
enum PeerStatus {
    Connected,
    Disconnected,
}

struct QueuedPeerEvent {
    event: BehaviourEvent,
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

impl<TSubstream> ExocoreBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    pub fn new() -> ExocoreBehaviour<TSubstream> {
        ExocoreBehaviour {
            local_node: PeerId::random(),
            events: VecDeque::new(),
            peers: HashMap::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn send_message(&mut self, peer_id: PeerId, expiration: Option<Instant>, data: Vec<u8>) {
        let event = NetworkBehaviourAction::SendEvent {
            peer_id: peer_id.clone(),
            event: WireMessage { data },
        };

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if peer.status == PeerStatus::Connected {
                self.events.push_back(event);
            } else {
                debug!("Peer {} not connected. Queuing message.", peer_id);
                // Node is disconnected, push the event to a queue and try to connect
                peer.temp_queue
                    .push_back(QueuedPeerEvent { event, expiration });

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

        self.dial_peer(peer_id.clone());
    }

    fn dial_peer(&mut self, peer_id: PeerId) {
        self.events.push_back(NetworkBehaviourAction::DialPeer {
            peer_id: peer_id.clone(),
        });
    }
}

impl<TSubstream> Default for ExocoreBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    fn default() -> Self {
        ExocoreBehaviour::new()
    }
}

impl<TSubstream> NetworkBehaviour for ExocoreBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    type ProtocolsHandler =
        OneShotHandler<TSubstream, ExocoreProtoConfig, WireMessage, OneshotEvent>;
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

            // send any messages that were queued while node was disconnected, but that haven't expired
            while let Some(event) = peer.temp_queue.pop_front() {
                if !event.has_expired() {
                    self.events.push_back(event.event);
                }
            }
        }
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _endpoint: ConnectedPoint) {
        debug!("{}: Disconnected from {}", self.local_node, peer_id,);

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.status = PeerStatus::Disconnected;

            // check if we need to reconnect
            peer.cleanup_expired();
            if !peer.temp_queue.is_empty() {
                self.dial_peer(peer_id.clone());
            }
        }
    }

    fn inject_node_event(&mut self, peer_id: PeerId, event: OneshotEvent) {
        if let OneshotEvent::Received(msg) = event {
            trace!("{}: Received message from {}", self.local_node, peer_id);

            self.events.push_back(NetworkBehaviourAction::GenerateEvent(
                ExocoreBehaviourEvent::Message(ExocoreBehaviourMessage {
                    source: peer_id,
                    data: msg.data,
                }),
            ));
        } else {
            trace!("{}: Our message got sent", self.local_node);
        }
    }

    fn inject_dial_failure(&mut self, peer_id: &PeerId) {
        debug!("{}: Failed to connect to {}", self.local_node, peer_id);
    }

    fn poll(
        &mut self,
        _poll_params: &mut impl PollParameters,
    ) -> Async<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        if let Some(event) = self.events.pop_front() {
            return Async::Ready(event);
        }

        Async::NotReady
    }
}

///
/// Event emitted by the ExocoreBehaviour
///
#[derive(Debug)]
pub enum ExocoreBehaviourEvent {
    Message(ExocoreBehaviourMessage),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExocoreBehaviourMessage {
    pub source: PeerId,
    pub data: Vec<u8>,
}

///
/// Event emitted by OneShotHandler (protocol handler) when a message has been received
/// or sent.
///
pub enum OneshotEvent {
    Received(WireMessage),
    Sent,
}

impl From<WireMessage> for OneshotEvent {
    #[inline]
    fn from(rpc: WireMessage) -> OneshotEvent {
        OneshotEvent::Received(rpc)
    }
}

impl From<()> for OneshotEvent {
    #[inline]
    fn from(_: ()) -> OneshotEvent {
        OneshotEvent::Sent
    }
}
