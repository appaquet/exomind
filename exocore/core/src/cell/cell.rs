use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, RwLock},
};

use exocore_protos::{apps::Manifest, generated::exocore_core::CellConfig, registry::Registry};
use libp2p::PeerId;

use super::{
    cell_apps::cell_app_directory, config::CellConfigExt, ApplicationId, CellApplications,
    CellNode, CellNodeRole, CellNodes, CellNodesRead, CellNodesWrite, Error, LocalNode, Node,
    NodeId,
};
use crate::{
    dir::DynDirectory,
    sec::keys::{Keypair, PublicKey},
};

const CELL_CONFIG_FILE: &str = "cell.yaml";

/// A Cell represents a private enclosure in which the data and applications of
/// a user are hosted. A Cell resides on multiple nodes.
#[derive(Clone)]
pub struct Cell {
    identity: Arc<Identity>,
    nodes: Arc<RwLock<HashMap<NodeId, CellNode>>>,
    apps: CellApplications,
    schemas: Arc<Registry>,
    dir: DynDirectory,
}

struct Identity {
    config: CellConfig,
    public_key: PublicKey,
    cell_id: CellId,
    local_node: LocalNode,
    name: String,
}

impl Cell {
    pub fn from_config(config: CellConfig, local_node: LocalNode) -> Result<EitherCell, Error> {
        let cell = Cell::build(config.clone(), local_node)?;

        let either_cell = if !config.keypair.is_empty() {
            let keypair = Keypair::decode_base58_string(&config.keypair)
                .map_err(|err| Error::Cell(anyhow!("Couldn't parse cell keypair: {}", err)))?;
            let full_cell = FullCell::build(keypair, cell);
            EitherCell::Full(Box::new(full_cell))
        } else {
            EitherCell::Cell(Box::new(cell))
        };

        Ok(either_cell)
    }

    pub fn from_directory(
        dir: impl Into<DynDirectory>,
        local_node: LocalNode,
    ) -> Result<EitherCell, Error> {
        let dir = dir.into();

        let cell_config = {
            let config_file = dir.open_read(Path::new(CELL_CONFIG_FILE))?;
            CellConfig::read_yaml(config_file)?
        };

        Self::from_config(cell_config, local_node)
    }

    pub fn from_local_node(local_node: LocalNode) -> Result<Vec<EitherCell>, Error> {
        let config = local_node.config();

        let mut either_cells = Vec::new();
        for node_cell_config in &config.cells {
            let either_cell = if node_cell_config.location.is_none() {
                let cell_id = CellId::from_str(&node_cell_config.id).map_err(|_err| {
                    Error::Cell(anyhow!("couldn't parse cell id '{}'", node_cell_config.id))
                })?;
                let cell_dir = local_node.cell_directory(&cell_id);
                Cell::from_directory(cell_dir, local_node.clone()).map_err(|err| {
                    Error::Cell(anyhow!("Failed to load cell id '{}': {}", cell_id, err))
                })?
            } else {
                warn!("Loading from inlined cell config...");
                let cell_config = CellConfig::from_node_cell(node_cell_config)?;
                Self::from_config(cell_config, local_node.clone())?
            };

            either_cells.push(either_cell);
        }

        Ok(either_cells)
    }

    pub fn from_local_node_directory(
        dir: impl Into<DynDirectory>,
    ) -> Result<(Vec<EitherCell>, LocalNode), Error> {
        let local_node = LocalNode::from_directory(dir.into())?;
        let cells = Self::from_local_node(local_node.clone())?;
        Ok((cells, local_node))
    }

    fn build(config: CellConfig, local_node: LocalNode) -> Result<Cell, Error> {
        let public_key = PublicKey::decode_base58_string(&config.public_key)
            .map_err(|err| Error::Cell(anyhow!("Couldn't parse cell public key: {}", err)))?;
        let cell_id = CellId::from_public_key(&public_key);

        let mut nodes_map = HashMap::new();
        let local_cell_node = CellNode::new(local_node.node().clone());
        nodes_map.insert(local_node.id().clone(), local_cell_node);

        let name = Some(config.name.clone())
            .filter(|n| !String::is_empty(n))
            .unwrap_or_else(|| public_key.generate_name());

        let schemas = Arc::new(Registry::new_with_exocore_types());

        let dir = local_node.cell_directory(&cell_id);

        let cell = Cell {
            identity: Arc::new(Identity {
                config: config.clone(),
                public_key,
                cell_id,
                local_node,
                name,
            }),
            apps: CellApplications::new(schemas.clone()),
            nodes: Arc::new(RwLock::new(nodes_map)),
            schemas,
            dir,
        };

        {
            // load nodes from config
            let mut nodes = cell.nodes_mut();
            for node_config in &config.nodes {
                let node = Node::from_config(node_config.node.clone().ok_or_else(|| {
                    Error::Config(anyhow!("Cell node config node is not defined"))
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
            let apps_dir = &cell.apps_directory();
            cell.apps
                .load_from_configurations(cell.id(), apps_dir, config.apps.iter())?;
        }

        Ok(cell)
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

    pub fn config(&self) -> &CellConfig {
        &self.identity.config
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

    pub fn directory(&self) -> &DynDirectory {
        &self.dir
    }

    pub fn chain_directory(&self) -> DynDirectory {
        self.directory().scope(PathBuf::from("chain"))
    }

    pub fn store_directory(&self) -> DynDirectory {
        self.directory().scope(PathBuf::from("store"))
    }

    pub fn apps_directory(&self) -> DynDirectory {
        self.directory().scope(PathBuf::from("apps"))
    }

    pub fn app_directory(&self, app_manifest: &Manifest) -> Result<DynDirectory, Error> {
        let app_id = ApplicationId::from_base58_public_key(&app_manifest.public_key)?;
        let apps_dir = self.apps_directory();
        Ok(cell_app_directory(
            &apps_dir,
            &app_id,
            &app_manifest.version,
        ))
    }

    pub fn temp_directory(&self) -> DynDirectory {
        self.directory().scope(PathBuf::from("tmp"))
    }

    pub fn save_config(&self, config: &CellConfig) -> Result<(), Error> {
        Self::write_cell_config(self.directory(), config)
    }

    pub fn write_cell_config(dir: &DynDirectory, config: &CellConfig) -> Result<(), Error> {
        let file = dir.open_create(Path::new(CELL_CONFIG_FILE))?;
        config.write_yaml(file)?;
        Ok(())
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
        let peer_id = PeerId::from_public_key(public_key.to_libp2p());
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
    pub fn generate(local_node: LocalNode) -> Result<FullCell, Error> {
        let keypair = Keypair::generate_ed25519();
        let config = CellConfig {
            keypair: keypair.encode_base58_string(),
            public_key: keypair.public().encode_base58_string(),
            ..Default::default()
        };

        let cell = Cell::build(config.clone(), local_node)?;
        let full_cell = Self::build(keypair, cell);
        full_cell.cell().save_config(&config)?;

        Ok(full_cell)
    }

    fn build(keypair: Keypair, cell: Cell) -> FullCell {
        FullCell { cell, keypair }
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn cell(&self) -> &Cell {
        &self.cell
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn with_local_node(self, local_node: LocalNode) -> FullCell {
        let cell = Cell::from_config(self.cell.config().clone(), local_node)
            .expect("Couldn't rebuild cell from current cell");
        cell.unwrap_full()
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

#[cfg(test)]
mod tests {
    use exocore_protos::core::CellApplicationConfig;

    use super::*;
    use crate::{
        cell::{Application, CellApplicationConfigExt},
        dir::{ram::RamDirectory, Directory},
    };

    #[test]
    fn test_save_load_directory() {
        let dir = RamDirectory::new();
        let node = LocalNode::generate_in_directory(dir.clone()).unwrap();

        let cell1 = FullCell::generate(node.clone()).unwrap();
        let cell_dir = cell1.cell().directory().clone();

        let cell2 = Cell::from_directory(cell_dir, node).unwrap();
        assert_eq!(cell1.cell().id(), cell2.cell().id());
    }

    #[test]
    fn test_load_inlined_cell_apps() {
        let dir = RamDirectory::new();
        let node = LocalNode::generate_in_directory(dir.clone()).unwrap();

        let full_cell = FullCell::generate(node.clone()).unwrap();
        let cell = full_cell.cell();

        // Crate an application in memory
        let mem_dir = RamDirectory::default();
        let (_kp, app) = Application::generate(mem_dir.clone(), "some app".to_string()).unwrap();
        app.save_manifest(app.manifest()).unwrap();

        // Copy application to cell app directory
        let app_dir = cell.app_directory(app.manifest()).unwrap();
        mem_dir.copy_to(app_dir).unwrap();

        // Add it to the cell
        let mut cell_config = cell.config().clone();
        cell_config.add_application(CellApplicationConfig::from_manifest(app.manifest().clone()));
        cell.save_config(&cell_config).unwrap();

        // Reload cell
        let full_cell = Cell::from_directory(cell.directory().clone(), node.clone()).unwrap();
        let cell = full_cell.cell();
        let apps = cell.applications().get();
        assert_eq!(apps.len(), 1);
        assert!(apps[0].is_loaded());

        // Load cell from config directly. Should still have the app, but unloaded.
        let dir = RamDirectory::new();
        let node_config = node.config().clone();
        let node_prime = LocalNode::from_config(dir, node_config).unwrap();
        let full_cell_prime = Cell::from_config(cell_config, node_prime)
            .unwrap()
            .unwrap_full();
        let apps = full_cell_prime.cell().applications().get();
        assert_eq!(apps.len(), 1);
        assert!(!apps[0].is_loaded());
    }
}
