use crate::security::signature::Signature;
use crate::serialization::framed::{FrameSigner, MultihashFrameSigner};
use libp2p_core::identity::{Keypair, PublicKey};
use libp2p_core::nodes::Peer;
use libp2p_core::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::ops::Deref;

// TODO: Replace by struct
pub type NodeID = String;

// TODO: To be put back in cell when we'll implement it here: https://github.com/appaquet/exocore/issues/37
// TODO: ACLs

///
///
///
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    node_id: NodeID,
    peer_id: PeerId,
    public_key: PublicKey,
    addresses: Vec<Multiaddr>,
    //    address: String,
    //    is_me: bool,
}

impl Node {
    pub fn new(node_id: String) -> Node {
        // TODO: Fixme
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(keypair.public());

        Node {
            node_id,
            peer_id,
            public_key: keypair.public(),
            addresses: vec![],
        }
    }

    #[inline]
    pub fn id(&self) -> &NodeID {
        &self.node_id
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
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

///
///
///
#[derive(Clone)]
pub struct LocalNode {
    node: Node,
    keypair: Keypair,
}

impl LocalNode {
    pub fn generate() -> LocalNode {
        // TODO: Fixme
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(keypair.public());
        let node_id = peer_id.to_string();

        LocalNode {
            node: Node {
                node_id,
                peer_id,
                public_key: keypair.public(),
                addresses: vec![],
            },
            keypair,
        }
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

///
///
///
#[derive(Clone)]
pub struct Nodes {
    local_node: LocalNode,
    nodes: HashMap<NodeID, Node>,
}

impl Nodes {
    #[deprecated]
    pub fn new() -> Nodes {
        // TODO: Fix me
        let mut nodes = HashMap::new();
        let local_node = LocalNode::generate();
        Nodes {
            local_node,
            nodes,
        }
    }

    fn new_with_local(local_node: LocalNode) -> Nodes {
        let mut nodes = HashMap::new();
        nodes.insert(local_node.node_id.clone(), local_node.node.clone());
        Nodes {
            local_node,
            nodes,
        }
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
