use futures::prelude::*;
use libp2p::core::nodes::raw_swarm::ConnectedPoint;
use libp2p::core::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};
use libp2p::{Multiaddr, PeerId};

use libp2p::core::protocols_handler::{OneShotHandler, ProtocolsHandler};
use std::collections::{HashMap, VecDeque};
use tokio::prelude::{AsyncRead, AsyncWrite};

use super::protocol::{ExocoreProtoConfig, WireMessage};

///
/// Libp2p's behaviour for Exocore. The behaviour defines a protocol that is exposed to
/// libp2p, peers that we want to talk to and acts as a stream / sink of messages exchanged
/// between nodes.
///
pub struct ExocoreBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    local_node: PeerId,
    events: VecDeque<NetworkBehaviourAction<WireMessage, ExocoreBehaviourEvent>>,
    peers: HashMap<PeerId, Vec<Multiaddr>>,
    phantom: std::marker::PhantomData<TSubstream>,
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

    pub fn send_message(&mut self, peer_id: PeerId, data: Vec<u8>) {
        // TODO: Check if node is connected. Otherwise, we may want to queue
        self.events.push_back(NetworkBehaviourAction::SendEvent {
            peer_id: peer_id.clone(),
            event: WireMessage { data },
        });
    }

    pub fn add_peer(&mut self, peer_id: PeerId, addresses: Vec<Multiaddr>) {
        self.peers.insert(peer_id.clone(), addresses);
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
        // TODO: ideally, addresses should be ordered so that best address is first
        self.peers.get(peer_id).cloned().unwrap_or_else(Vec::new)
    }

    fn inject_connected(&mut self, peer_id: PeerId, _endpoint: ConnectedPoint) {
        println!("{}: Connected to {}", self.local_node, peer_id,);
        // TODO: If any queued message for this node, add them to events
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _endpoint: ConnectedPoint) {
        println!("{}: Disconnected from {}", self.local_node, peer_id,);
        self.events.push_back(NetworkBehaviourAction::DialPeer {
            peer_id: peer_id.clone(),
        });
    }

    fn inject_node_event(&mut self, peer_id: PeerId, event: OneshotEvent) {
        if let OneshotEvent::Received(msg) = event {
            println!(
                "{}: Received message from {}: {}",
                self.local_node,
                peer_id,
                String::from_utf8_lossy(&msg.data)
            );

            self.events.push_back(NetworkBehaviourAction::GenerateEvent(
                ExocoreBehaviourEvent::Message(ExocoreBehaviourMessage {
                    source: peer_id,
                    data: msg.data,
                }),
            ));
        } else {
            println!("{}: Our message got sent", self.local_node,);
        }
    }

    fn poll(
        &mut self,
        _poll_params: &mut PollParameters<'_>,
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
    // TODO: Connected, Disconnected
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
