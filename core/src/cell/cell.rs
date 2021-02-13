use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use libp2p::core::PeerId;

use super::{
    config::CellConfigExt, CellApplications, CellNode, CellNodeRole, CellNodes, CellNodesRead,
    CellNodesWrite, Error, LocalNode, Node, NodeId,
};
use crate::sec::keys::{Keypair, PublicKey};
use exocore_protos::{
    generated::exocore_core::{CellConfig, LocalNodeConfig},
    registry::Registry,
};

/// A Cell represents a private enclosure in which the data and applications of
/// a user are hosted. A Cell resides on multiple nodes.
#[derive(Clone)]
pub struct Cell {
    identity: Arc<Identity>,
    nodes: Arc<RwLock<HashMap<NodeId, CellNode>>>,
    apps: CellApplications,
    schemas: Arc<Registry>,
}

struct Identity {
    public_key: PublicKey,
    cell_id: CellId,
    local_node: LocalNode,
    name: String,
    path: Option<PathBuf>,
}

impl Cell {
    pub fn new(public_key: PublicKey, local_node: LocalNode) -> Cell {
        Self::build(public_key, local_node, None, None)
    }

    pub fn from_config(config: CellConfig, local_node: LocalNode) -> Result<EitherCell, Error> {
        let either_cell = if !config.keypair.is_empty() {
            let keypair = Keypair::decode_base58_string(&config.keypair)
                .map_err(|err| Error::Cell(format!("Couldn't parse cell keypair: {}", err)))?;

            let name = Some(config.name.clone()).filter(String::is_empty);

            let path = if !config.path.is_empty() {
                Some(PathBuf::from(config.path.clone()))
            } else {
                None
            };

            let full_cell = FullCell::build(keypair, local_node, name, path);
            EitherCell::Full(Box::new(full_cell))
        } else {
            let public_key = PublicKey::decode_base58_string(&config.public_key)
                .map_err(|err| Error::Cell(format!("Couldn't parse cell public key: {}", err)))?;

            let name = Some(config.name.clone()).filter(String::is_empty);

            let path = if !config.path.is_empty() {
                Some(PathBuf::from(config.path.clone()))
            } else {
                None
            };

            let cell = Cell::build(public_key, local_node, name, path);
            EitherCell::Cell(Box::new(cell))
        };

        {
            // load nodes from config
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

        {
            // load apps from config
            let cell = either_cell.cell();
            cell.apps.load_from_cell_apps_conf(config.apps.iter())?;
        }

        Ok(either_cell)
    }

    pub fn from_directory<P: AsRef<Path>>(
        directory: P,
        local_node: LocalNode,
    ) -> Result<EitherCell, Error> {
        let mut config_path = directory.as_ref().to_path_buf();
        config_path.push("cell.yaml");

        let cell_config = CellConfig::from_yaml_file(config_path)?;

        Self::from_config(cell_config, local_node)
    }

    pub fn from_local_node(local_node: LocalNode) -> Result<(Vec<EitherCell>, LocalNode), Error> {
        let config = local_node.config();

        let mut either_cells = Vec::new();
        for node_cell_config in &config.cells {
            let cell_config = CellConfig::from_node_cell(node_cell_config)?;
            let either_cell = Self::from_config(cell_config, local_node.clone())?;
            either_cells.push(either_cell);
        }

        Ok((either_cells, local_node))
    }

    pub fn from_local_node_config(
        config: LocalNodeConfig,
    ) -> Result<(Vec<EitherCell>, LocalNode), Error> {
        let local_node = LocalNode::new_from_config(config)?;
        Self::from_local_node(local_node)
    }

    fn build(
        public_key: PublicKey,
        local_node: LocalNode,
        name: Option<String>,
        path: Option<PathBuf>,
    ) -> Cell {
        let cell_id = CellId::from_public_key(&public_key);

        let mut nodes_map = HashMap::new();
        let local_cell_node = CellNode::new(local_node.node().clone());
        nodes_map.insert(local_node.id().clone(), local_cell_node);

        let name = name.unwrap_or_else(|| public_key.generate_name());

        let schemas = Arc::new(Registry::new_with_exocore_types());

        Cell {
            identity: Arc::new(Identity {
                public_key,
                cell_id,
                local_node,
                name,
                path,
            }),
            apps: CellApplications::new(schemas.clone()),
            nodes: Arc::new(RwLock::new(nodes_map)),
            schemas,
        }
    }

    pub fn id(&self) -> &CellId {
        &self.identity.cell_id
    }

    pub fn name(&self) -> &str {
        &self.identity.name
    }

    pub fn local_node(&self) -> &LocalNode {
        &self.identity.local_node
    }

    pub fn local_node_has_role(&self, role: CellNodeRole) -> bool {
        let nodes = self.nodes();
        if let Some(cn) = nodes.get(self.identity.local_node.id()) {
            cn.has_role(role)
        } else {
            false
        }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.identity.public_key
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

    pub fn schemas(&self) -> &Arc<Registry> {
        &self.schemas
    }

    pub fn applications(&self) -> &CellApplications {
        &self.apps
    }

    pub fn cell_directory(&self) -> Option<&Path> {
        self.identity.path.as_deref()
    }

    pub fn chain_directory(&self) -> Option<PathBuf> {
        if let Some(path) = &self.identity.path {
            let mut chain_dir = PathBuf::from(path);
            chain_dir.push("chain");
            Some(chain_dir)
        } else {
            None
        }
    }

    pub fn store_directory(&self) -> Option<PathBuf> {
        if let Some(path) = &self.identity.path {
            let mut store_dir = PathBuf::from(path);
            store_dir.push("store");
            Some(store_dir)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Cell{")?;
        f.write_str(&self.identity.name)?;
        f.write_str("}")
    }
}

/// Unique identifier of a cell, which is built by hashing the public key
///
/// For now, this ID is generated the same way as node IDs.
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
        Self::build(keypair, local_node, None, None)
    }

    pub fn generate(local_node: LocalNode) -> FullCell {
        let cell_keypair = Keypair::generate_ed25519();
        Self::build(cell_keypair, local_node, None, None)
    }

    fn build(
        keypair: Keypair,
        local_node: LocalNode,
        name: Option<String>,
        path: Option<PathBuf>,
    ) -> FullCell {
        FullCell {
            cell: Cell::build(keypair.public(), local_node, name, path),
            keypair,
        }
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn cell(&self) -> &Cell {
        &self.cell
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn with_local_node(self, local_node: LocalNode) -> FullCell {
        FullCell::from_keypair(self.keypair, local_node)
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn with_path(self, path: PathBuf) -> FullCell {
        Self::build(
            self.keypair,
            self.cell.local_node().clone(),
            None,
            Some(path),
        )
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
            EitherCell::Full(full_cell) => full_cell.cell().nodes(),
            EitherCell::Cell(cell) => cell.nodes(),
        }
    }

    pub fn nodes_mut(&self) -> CellNodesWrite {
        match self {
            EitherCell::Full(full_cell) => full_cell.cell().nodes_mut(),
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
