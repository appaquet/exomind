use crate::security::signature::Signature;
use crate::serialization::framed::{FrameSigner, MultihashFrameSigner};
use libp2p_core::identity::{Keypair, PublicKey};
use libp2p_core::{Multiaddr, PeerId};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

//
// TODO: Encryption/signature ticket: https://github.com/appaquet/exocore/issues/46
//

///
/// Represents a machine / process on which Exocore runs. A node can host multiple `Cell`.
///
#[derive(Clone)]
pub struct Node {
    node_id: NodeID,
    peer_id: PeerId,
    public_key: PublicKey,
    inner: Arc<RwLock<SharedInner>>,
}

struct SharedInner {
    addresses: HashSet<Multiaddr>,
}

impl Node {
    pub fn new_from_public_key(public_key: PublicKey) -> Node {
        let peer_id = PeerId::from_public_key(public_key.clone());
        let node_id = peer_id.to_string();

        Node {
            node_id,
            peer_id,
            public_key,
            inner: Arc::new(RwLock::new(SharedInner {
                addresses: HashSet::new(),
            })),
        }
    }

    #[cfg(any(test, feature = "tests_utils"))]
    pub fn generate_for_tests() -> Node {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(keypair.public());
        let node_id = peer_id.to_string();

        Node {
            node_id,
            peer_id,
            public_key: keypair.public(),
            inner: Arc::new(RwLock::new(SharedInner {
                addresses: HashSet::new(),
            })),
        }
    }

    #[inline]
    pub fn id(&self) -> &NodeID {
        &self.node_id
    }

    #[inline]
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn has_full_access(&self) -> bool {
        // TODO: This should return if the node has access to the cell's private key
        //        Probably in https://github.com/appaquet/exocore/issues/46
        true
    }

    pub fn addresses(&self) -> Vec<Multiaddr> {
        let inner = self.inner.read().expect("Couldn't get inner lock");
        inner.addresses.iter().cloned().collect()
    }

    pub fn add_address(&self, address: Multiaddr) {
        let mut inner = self.inner.write().expect("Couldn't get inner lock");
        inner.addresses.insert(address);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.node_id.eq(&other.node_id)
    }
}

impl Eq for Node {}

///
/// Represents the local `Node` being run in the current process. Contrarily to other nodes,
/// we have a full private+public keypair that we can sign messages with.
///
#[derive(Clone)]
pub struct LocalNode {
    node: Node,
    keypair: Keypair,
}

impl LocalNode {
    pub fn new_from_keypair(keypair: Keypair) -> LocalNode {
        LocalNode {
            node: Node::new_from_public_key(keypair.public()),
            keypair,
        }
    }

    pub fn generate() -> LocalNode {
        LocalNode::new_from_keypair(Keypair::generate_ed25519())
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn frame_signer(&self) -> impl FrameSigner {
        // TODO: Signature ticket: https://github.com/appaquet/exocore/issues/46
        //       Include signature, not just hash.
        MultihashFrameSigner::new_sha3256()
    }

    pub fn sign_message(&self, _message: &[u8]) -> Signature {
        // TODO: Signature ticket: https://github.com/appaquet/exocore/issues/46
        //       Make sure we're local and we have access to private key
        Signature::empty()
    }
}

impl Deref for LocalNode {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl PartialEq for LocalNode {
    fn eq(&self, other: &Self) -> bool {
        self.node_id.eq(&other.node_id)
    }
}

impl Eq for LocalNode {}

///
/// Unique identifier of a node, which is built by hashing the public key of the node.
/// It has a one to one correspondence with libp2p's PeerId
///
pub type NodeID = String;

pub fn node_id_from_peer_id(peer_id: &PeerId) -> NodeID {
    peer_id.to_string()
}
