use crate::security::signature::Signature;
use crate::serialization::framed::{FrameSigner, MultihashFrameSigner};
use libp2p_core::identity::{Keypair, PublicKey};
use libp2p_core::{Multiaddr, PeerId};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, RwLock};

//
// TODO: To be cleaned up in https://github.com/appaquet/exocore/issues/37
// TODO: Encryption/signature ticket: https://github.com/appaquet/exocore/issues/46
//

/// Unique identifier of a node, which is built by hashing the public key of the node.
/// It has a one to one correspondence with libp2p's PeerId
pub type NodeID = String;

#[deprecated]
pub fn node_id_from_peer_id(peer_id: &PeerId) -> NodeID {
    peer_id.to_string()
}

/// Represents a machine / process on which Exocore runs. A node can host multiple `Cell`.
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

    #[deprecated]
    pub fn new(node_id: String) -> Node {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(keypair.public());

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

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn addresses(&self) -> Vec<Multiaddr> {
        let inner = self.inner.read().expect("Couldn't get inner lock");
        inner.addresses.iter().cloned().collect()
    }

    pub fn add_address(&self, address: Multiaddr) {
        let mut inner = self.inner.write().expect("Couldn't get inner lock");
        inner.addresses.insert(address);
    }

    #[deprecated]
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

/// Represents the local `Node` being run in the current process. Contrarily to other nodes,
/// we have a full private+public keypair that we can sign messages with.
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
}

impl Deref for LocalNode {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

/// Collection of nodes of a `Cell`
#[derive(Clone)]
pub struct Nodes {
    local_node: LocalNode,
    nodes: HashMap<NodeID, Node>,
}

impl Nodes {
    #[deprecated]
    pub fn new() -> Nodes {
        let nodes = HashMap::new();
        let local_node = LocalNode::generate();
        Nodes { local_node, nodes }
    }

    pub fn new_with_local(local_node: LocalNode) -> Nodes {
        let mut nodes = HashMap::new();
        nodes.insert(local_node.node_id.clone(), local_node.node.clone());
        Nodes { local_node, nodes }
    }

    pub fn local_node(&self) -> &LocalNode {
        &self.local_node
    }

    pub fn add(&mut self, node: Node) {
        self.nodes.insert(node.id().clone(), node);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn nodes_except<'a>(&'a self, node_id: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.nodes.values().filter(move |n| n.id() != node_id)
    }

    #[inline]
    pub fn get(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn is_quorum(&self, count: usize) -> bool {
        if self.is_empty() {
            false
        } else if self.len() == 1 {
            count == 1
        } else {
            count > self.len() / 2
        }
    }
}

impl Default for Nodes {
    fn default() -> Self {
        Nodes::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nodes_add_get() {
        let mut nodes = Nodes::new();
        assert!(nodes.is_empty());

        nodes.add(Node::new("node1".to_string()));

        assert!(!nodes.is_empty());
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes.nodes.len(), 1);

        assert!(nodes.get("node1").is_some());
        assert!(nodes.get("blabla").is_none());
    }

    #[test]
    fn nodes_quorum() {
        let mut nodes = Nodes::new();
        assert!(!nodes.is_quorum(10));

        nodes.add(Node::new("node1".to_string()));
        assert!(!nodes.is_quorum(0));
        assert!(nodes.is_quorum(1));

        nodes.add(Node::new("node2".to_string()));
        assert!(!nodes.is_quorum(1));
        assert!(nodes.is_quorum(2));

        nodes.add(Node::new("node3".to_string()));
        assert!(!nodes.is_quorum(1));
        assert!(nodes.is_quorum(2));
    }

}
