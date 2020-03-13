use super::{CellNodesRead, CellNodesWrite};
use crate::cell::nodes::{CellNode, CellNodeRole};
use crate::cell::{CellNodes, LocalNode, NodeId};
use crate::crypto::keys::{Keypair, PublicKey};
use libp2p_core::PeerId;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

/// A Cell represents a private enclosure in which the data and applications of
/// a user are hosted. A Cell resides on multiple nodes.
#[derive(Clone)]
pub struct Cell {
    public_key: Arc<PublicKey>,
    cell_id: CellId,
    local_node: LocalNode,
    nodes: Arc<RwLock<HashMap<NodeId, CellNode>>>,
}

impl Cell {
    pub fn new(public_key: PublicKey, local_node: LocalNode) -> Cell {
        let cell_id = CellId::from_public_key(&public_key);

        let mut nodes_map = HashMap::new();
        let local_cell_node = CellNode::new(local_node.node().clone());
        nodes_map.insert(local_node.id().clone(), local_cell_node);

        Cell {
            public_key: Arc::new(public_key),
            cell_id,
            local_node,
            nodes: Arc::new(RwLock::new(nodes_map)),
        }
    }

    #[inline]
    pub fn id(&self) -> &CellId {
        &self.cell_id
    }

    #[inline]
    pub fn local_node(&self) -> &LocalNode {
        &self.local_node
    }

    #[inline]
    pub fn local_node_has_role(&self, role: CellNodeRole) -> bool {
        let nodes = self.nodes();
        if let Some(cn) = nodes.get(self.local_node.id()) {
            cn.has_role(role)
        } else {
            false
        }
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
            .expect("Couldn't acquire write lock on nodes");
        CellNodesWrite { cell: self, nodes }
    }
}

/// Unique identifier of a cell, which is built by hashing the public key
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct CellId(String);

impl CellId {
    /// Create a Cell ID from a public key by using libp2p method to be
    /// compatible with it
    pub fn from_public_key(public_key: &PublicKey) -> CellId {
        let peer_id = PeerId::from_public_key(public_key.to_libp2p().clone());
        CellId(peer_id.to_string())
    }

    pub fn from_string(id: String) -> CellId {
        CellId(id)
    }

    pub fn from_bytes(id: &[u8]) -> CellId {
        CellId(String::from_utf8_lossy(id).to_string())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl std::fmt::Display for CellId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::str::FromStr for CellId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CellId(s.to_string()))
    }
}

/// A Cell for which we have full access since we have the private key.
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
