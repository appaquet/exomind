use super::{
    CellNode, CellNodeRole, CellNodes, CellNodesRead, CellNodesWrite, Error, LocalNode, Node,
    NodeId,
};
use crate::crypto::keys::{Keypair, PublicKey};
use crate::protos::generated::exocore_core::{CellConfig, LocalNodeConfig};
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
    name: String,
    nodes: Arc<RwLock<HashMap<NodeId, CellNode>>>,
}

impl Cell {
    pub fn new(public_key: PublicKey, local_node: LocalNode) -> Cell {
        let cell_id = CellId::from_public_key(&public_key);

        let mut nodes_map = HashMap::new();
        let local_cell_node = CellNode::new(local_node.node().clone());
        nodes_map.insert(local_node.id().clone(), local_cell_node);

        // generate a deterministic random name for the cell
        let name = public_key.generate_name();

        Cell {
            public_key: Arc::new(public_key),
            cell_id,
            local_node,
            name,
            nodes: Arc::new(RwLock::new(nodes_map)),
        }
    }

    pub fn new_from_config(config: CellConfig, local_node: LocalNode) -> Result<EitherCell, Error> {
        let either_cell = if !config.keypair.is_empty() {
            let keypair = Keypair::decode_base58_string(&config.keypair)
                .map_err(|err| Error::Config(format!("Couldn't parse cell keypair: {}", err)))?;

            let mut full_cell = FullCell::from_keypair(keypair, local_node);

            if config.name != "" {
                full_cell.cell.name = config.name;
            }

            EitherCell::Full(Box::new(full_cell))
        } else {
            let public_key = PublicKey::decode_base58_string(&config.public_key)
                .map_err(|err| Error::Config(format!("Couldn't parse cell public key: {}", err)))?;

            let mut cell = Cell::new(public_key, local_node);

            if config.name != "" {
                cell.name = config.name;
            }

            EitherCell::Cell(Box::new(cell))
        };

        {
            let mut nodes = either_cell.nodes_mut();
            for node_config in &config.nodes {
                let node = Node::new_from_config(node_config.node.clone().ok_or_else(|| {
                    Error::Config("Cell node config node is not defined".to_string())
                })?)?;

                let mut cell_node = CellNode::new(node);

                for role in node_config.roles() {
                    cell_node.add_role(CellNodeRole::from_config(role)?);
                }

                nodes.add_cell_node(cell_node);
            }
        }

        Ok(either_cell)
    }

    pub fn new_from_local_node_config(
        config: LocalNodeConfig,
    ) -> Result<(Vec<EitherCell>, LocalNode), Error> {
        let local_node = LocalNode::new_from_config(config.clone())?;

        let mut either_cells = Vec::new();
        for cell_config in config.cells {
            let either_cell = Cell::new_from_config(cell_config.clone(), local_node.clone())?;
            either_cells.push(either_cell);
        }

        Ok((either_cells, local_node))
    }

    #[inline]
    pub fn id(&self) -> &CellId {
        &self.cell_id
    }

    pub fn name(&self) -> &str {
        &self.name
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

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Cell{")?;
        f.write_str(&self.name)?;
        f.write_str("}")
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

/// Enum wrapping a full or non-full cell
#[derive(Clone)]
pub enum EitherCell {
    Full(Box<FullCell>),
    Cell(Box<Cell>),
}

impl EitherCell {
    pub fn nodes(&self) -> CellNodesRead {
        match self {
            EitherCell::Full(cell) => cell.nodes(),
            EitherCell::Cell(cell) => cell.nodes(),
        }
    }

    pub fn nodes_mut(&self) -> CellNodesWrite {
        match self {
            EitherCell::Full(cell) => cell.nodes_mut(),
            EitherCell::Cell(cell) => cell.nodes_mut(),
        }
    }

    pub fn cell(&self) -> &Cell {
        match self {
            EitherCell::Full(cell) => cell.cell(),
            EitherCell::Cell(cell) => cell,
        }
    }

    pub fn unwrap_full(self) -> FullCell {
        match self {
            EitherCell::Full(cell) => cell.as_ref().clone(),
            _ => panic!("Tried to unwrap EitherCell into Full, but wasn't"),
        }
    }

    pub fn unwrap_cell(self) -> Cell {
        match self {
            EitherCell::Cell(cell) => cell.as_ref().clone(),
            _ => panic!("Tried to unwrap EitherCell into Cell, but wasn't"),
        }
    }
}
