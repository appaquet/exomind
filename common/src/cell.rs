//
// TODO: Encryption/signature ticket: https://github.com/appaquet/exocore/issues/46
//

use crate::node::{LocalNode, Node, NodeID};
use libp2p_core::identity::{Keypair, PublicKey};
use libp2p_core::PeerId;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

///
/// A Cell for which we have full access since we have the private key.
///
#[derive(Clone)]
pub struct FullCell {
    cell: Cell,
    keypair: Keypair,
}

impl FullCell {
    pub fn from_keypair(keypair: Keypair, local_node: LocalNode) -> FullCell {
        FullCell {
            cell: Cell::new(keypair.public(), local_node),
            keypair,
        }
    }

    pub fn generate(local_node: LocalNode) -> FullCell {
        let cell_keypair = Keypair::generate_ed25519();
        Self::from_keypair(cell_keypair, local_node)
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn cell(&self) -> &Cell {
        &self.cell
    }

    #[cfg(any(test, feature = "tests_utils"))]
    pub fn clone_for_local_node(&self, local_node: LocalNode) -> FullCell {
        FullCell::from_keypair(self.keypair.clone(), local_node)
    }
}

impl Deref for FullCell {
    type Target = Cell;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

///
/// A Cell represents a private enclosure in which the data and applications of a user
/// are hosted. A Cell resides on multiple nodes.
///
#[derive(Clone)]
pub struct Cell {
    public_key: PublicKey,
    cell_id: CellID,
    local_node: LocalNode,
    nodes: Arc<RwLock<HashMap<NodeID, Node>>>,
}

impl Cell {
    pub fn new(public_key: PublicKey, local_node: LocalNode) -> Cell {
        let dummy_peer_id = PeerId::from_public_key(public_key.clone());
        let cell_id = CellID::from_string(&dummy_peer_id.to_string());

        let mut nodes_map: HashMap<NodeID, Node> = HashMap::new();
        nodes_map.insert(local_node.id().clone(), local_node.node().clone());

        Cell {
            public_key,
            cell_id,
            local_node: local_node.clone(),
            nodes: Arc::new(RwLock::new(nodes_map)),
        }
    }

    #[inline]
    pub fn id(&self) -> &CellID {
        &self.cell_id
    }

    #[inline]
    pub fn local_node(&self) -> &LocalNode {
        &self.local_node
    }

    #[inline]
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn nodes(&self) -> CellNodesRead {
        let nodes = self
            .nodes
            .read()
            .expect("Couldn't acquire read lock on nodes");
        CellNodesRead { cell: self, nodes }
    }

    pub fn nodes_mut(&self) -> CellNodesWrite {
        let nodes = self
            .nodes
            .write()
            .expect("Couldn't acquire read lock on nodes");
        CellNodesWrite { cell: self, nodes }
    }
}

///
/// Unique identifier of a cell, which is built by hashing the public key
///
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct CellID {
    id: String,
}

impl CellID {
    pub fn from_public_key(public_key: &PublicKey) -> CellID {
        // TODO: Use our own method?
        let dummy_peer_id = PeerId::from_public_key(public_key.clone());
        CellID {
            id: dummy_peer_id.to_string(),
        }
    }

    pub fn from_string(id: &str) -> CellID {
        CellID { id: id.to_string() }
    }

    pub fn from_bytes(id: &[u8]) -> CellID {
        // TODO: fix
        CellID {
            id: String::from_utf8_lossy(id).to_string(),
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.id.as_bytes()
    }
}

///
/// Common methods collection of nodes of a `Cell`
///
pub trait CellNodes {
    fn cell(&self) -> &Cell;
    fn nodes_map(&self) -> &HashMap<NodeID, Node>;

    fn local_node(&self) -> &LocalNode {
        &self.cell().local_node
    }

    #[inline]
    fn len(&self) -> usize {
        self.nodes_map().len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn get(&self, node_id: &str) -> Option<&Node> {
        self.nodes_map().get(node_id)
    }

    fn is_quorum(&self, count: usize) -> bool {
        if self.is_empty() {
            false
        } else if self.len() == 1 {
            count == 1
        } else {
            count > self.len() / 2
        }
    }

    fn to_owned(&self) -> CellNodesOwned {
        CellNodesOwned {
            cell: self.cell().clone(),
            nodes: self.nodes_map().clone(),
        }
    }
}

///
/// Wraps a `CellNodes` to expose iterator methods. This is needed because of the complexity
/// of return types of iterators which require `impl` to be used, but cannot be used in traits.
///
pub struct CellNodesIter<'cn, N: CellNodes> {
    nodes: &'cn N,
}

impl<'cn, N: CellNodes> CellNodesIter<'cn, N> {
    pub fn all(&self) -> impl Iterator<Item = &Node> {
        self.nodes.nodes_map().values()
    }

    pub fn all_except<'a>(&'a self, node_id: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.nodes
            .nodes_map()
            .values()
            .filter(move |n| n.id() != node_id)
    }

    pub fn all_except_local<'a>(&'a self) -> impl Iterator<Item = &'a Node> + 'a {
        let local_node = self.nodes.cell().local_node();
        self.all_except(local_node.id())
    }
}

///
/// Read reference to nodes of a `Cell`
///
pub struct CellNodesRead<'cell> {
    cell: &'cell Cell,
    nodes: RwLockReadGuard<'cell, HashMap<NodeID, Node>>,
}

impl<'cell> CellNodesRead<'cell> {
    pub fn iter(&self) -> CellNodesIter<CellNodesRead> {
        CellNodesIter { nodes: self }
    }
}

impl<'cell> CellNodes for CellNodesRead<'cell> {
    fn cell(&self) -> &Cell {
        &self.cell
    }

    fn nodes_map(&self) -> &HashMap<String, Node> {
        &self.nodes
    }
}

///
/// Write reference to nodes of a `Cell`
///
pub struct CellNodesWrite<'cell> {
    cell: &'cell Cell,
    nodes: RwLockWriteGuard<'cell, HashMap<NodeID, Node>>,
}

impl<'cell> CellNodesWrite<'cell> {
    pub fn iter(&self) -> CellNodesIter<CellNodesWrite> {
        CellNodesIter { nodes: self }
    }

    pub fn add(&mut self, node: Node) {
        self.nodes.insert(node.id().clone(), node);
    }
}

impl<'cell> CellNodes for CellNodesWrite<'cell> {
    fn cell(&self) -> &Cell {
        &self.cell
    }

    fn nodes_map(&self) -> &HashMap<String, Node> {
        &self.nodes
    }
}

///
/// Owned copy of nodes of a `Cell`
///
pub struct CellNodesOwned {
    cell: Cell,
    nodes: HashMap<NodeID, Node>,
}

impl CellNodesOwned {
    pub fn iter(&self) -> CellNodesIter<CellNodesOwned> {
        CellNodesIter { nodes: self }
    }
}

impl CellNodes for CellNodesOwned {
    fn cell(&self) -> &Cell {
        &self.cell
    }

    fn nodes_map(&self) -> &HashMap<String, Node> {
        &self.nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nodes_add_get() {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node.clone());

        {
            let nodes = cell.nodes();
            assert!(!nodes.is_empty());
            assert_eq!(nodes.len(), 1); // self
        }

        {
            let mut nodes = cell.nodes_mut();
            nodes.add(Node::generate_for_tests());
        }

        {
            let nodes = cell.nodes();
            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes.iter().all().count(), 2);
            assert_eq!(nodes.iter().all_except(local_node.id()).count(), 1);
            assert_ne!(
                nodes.iter().all_except_local().next().unwrap().id(),
                local_node.id()
            );

            assert!(nodes.get(local_node.id()).is_some());
            assert!(nodes.get("blabla").is_none());
        }
    }

    #[test]
    fn nodes_quorum() {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);

        {
            // only have 1 node (local_node)
            let nodes = cell.nodes();
            assert!(!nodes.is_quorum(0));
            assert!(nodes.is_quorum(1));
        }

        {
            let mut nodes = cell.nodes_mut();
            nodes.add(Node::generate_for_tests());
            assert!(!nodes.is_quorum(1));
            assert!(nodes.is_quorum(2));
        }

        {
            let mut nodes = cell.nodes_mut();
            nodes.add(Node::generate_for_tests());
            assert!(!nodes.is_quorum(1));
            assert!(nodes.is_quorum(2));
        }
    }

}
