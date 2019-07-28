use futures::prelude::*;
use libp2p_core::{ConnectedPoint, Multiaddr, PeerId};
use libp2p_swarm::{
    NetworkBehaviour, NetworkBehaviourAction, OneShotHandler, PollParameters, ProtocolsHandler,
};
use std::collections::{HashMap, VecDeque};
use tokio::io::{AsyncRead, AsyncWrite};

use super::protocol::{ExocoreProtoConfig, WireMessage};

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
        // TODO: If node is not online, we should queue https://github.com/appaquet/exocore/issues/60
        self.events.push_back(NetworkBehaviourAction::SendEvent {
            peer_id: peer_id.clone(),
            event: WireMessage { data },
        });
    }

    pub fn add_peer(&mut self, peer_id: PeerId, addresses: Vec<Multiaddr>) {
        let current_addresses = self.peers.get(&peer_id);
        if current_addresses.is_none() || current_addresses != Some(&addresses) {
            self.peers.insert(peer_id.clone(), addresses);
            self.events.push_back(NetworkBehaviourAction::DialPeer {
                peer_id: peer_id.clone(),
            });
        }
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
        self.peers.get(peer_id).cloned().unwrap_or_else(Vec::new)
    }

    fn inject_connected(&mut self, peer_id: PeerId, _endpoint: ConnectedPoint) {
        debug!("{}: Connected to {}", self.local_node, peer_id,);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _endpoint: ConnectedPoint) {
        debug!("{}: Disconnected from {}", self.local_node, peer_id,);
        self.events.push_back(NetworkBehaviourAction::DialPeer {
            peer_id: peer_id.clone(),
        });
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
