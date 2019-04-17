use crate::security::signature::Signature;
use crate::serialization::framed::{FrameSigner, MultihashFrameSigner};
use std::collections::HashMap;

pub type NodeID = String;

// TODO: To be put back in cell when we'll implement it here: https://github.com/appaquet/exocore/issues/37
// TODO: NodeID = hash(publickey)
// TODO: ACLs

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    id: NodeID,
    //    address: String,
    //    is_me: bool,
}

impl Node {
    pub fn new(id: String) -> Node {
        Node { id }
    }

    #[inline]
    pub fn id(&self) -> &NodeID {
        &self.id
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

pub struct Nodes {
    nodes: HashMap<NodeID, Node>,
}

impl Nodes {
    pub fn new() -> Nodes {
        Nodes {
            nodes: HashMap::new(),
        }
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
        count >= (self.len() / 2).max(1)
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

}
