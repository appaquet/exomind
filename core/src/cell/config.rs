use super::Error;
use super::{Cell, FullCell, LocalNode, Node};
use crate::crypto::keys::{Keypair, PublicKey};
use serde_derive::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

/// Root configuration of a Node running servers and Cells
#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_keypair: String,
    pub cells: Vec<CellConfig>,
    pub listen_addresses: Vec<String>,
}

impl NodeConfig {
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<NodeConfig, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(path.as_ref())
            .map_err(Error::ConfigIO)
            .map_err(|err| Error::ConfigOther(format!("Couldn't open YAML node file: {}", err)))?;

        Self::from_yaml_reader(file)
    }

    pub fn from_yaml_reader<R: std::io::Read>(bytes: R) -> Result<NodeConfig, Error> {
        let config = serde_yaml::from_reader(bytes).map_err(|err| {
            Error::ConfigOther(format!("Couldn't decode YAML node config: {}", err))
        })?;

        Ok(config)
    }

    pub fn create_local_node(&self) -> Result<LocalNode, Error> {
        let local_node_keypair =
            Keypair::decode_base58_string(&self.node_keypair).map_err(|err| {
                Error::ConfigOther(format!("Couldn't decode local node key pair: {}", err))
            })?;
        let local_node = LocalNode::new_from_keypair(local_node_keypair);
        for listen_address in &self.listen_addresses {
            local_node.add_address(listen_address.parse().unwrap());
        }

        Ok(local_node)
    }
}

/// Configuration of a Cell running in the node
#[derive(Serialize, Deserialize)]
pub struct CellConfig {
    pub public_key: String,
    pub keypair: Option<String>,
    pub data_directory: PathBuf,
    pub nodes: Vec<CellConfigNode>,
}

impl CellConfig {
    pub fn create_cell(&self, local_node: &LocalNode) -> Result<(Option<FullCell>, Cell), Error> {
        let (full_cell, cell) = if let Some(cell_keypair) = &self.keypair {
            let keypair = Keypair::decode_base58_string(cell_keypair).map_err(|err| {
                Error::ConfigOther(format!("Couldn't decode cell key pair: {}", err))
            })?;
            let full_cell = FullCell::from_keypair(keypair, local_node.clone());
            (Some(full_cell.clone()), full_cell.cell().clone())
        } else {
            let public_key = PublicKey::decode_base58_string(&self.public_key).map_err(|err| {
                Error::ConfigOther(format!("Couldn't decode cell public key: {}", err))
            })?;
            let cell = Cell::new(public_key, local_node.clone());
            (None, cell)
        };

        {
            let mut cell_nodes = cell.nodes_mut();
            for nodes_config in &self.nodes {
                let public_key = PublicKey::decode_base58_string(&nodes_config.public_key)
                    .map_err(|err| {
                        Error::ConfigOther(format!("Couldn't decode node public key: {}", err))
                    })?;
                let node = Node::new_from_public_key(public_key);

                for node_address in &nodes_config.addresses {
                    node.add_address(node_address.parse().map_err(|err| {
                        Error::ConfigOther(format!("Couldn't parse node config: {}", err))
                    })?);
                }

                cell_nodes.add(node);
            }
        }

        Ok((full_cell, cell))
    }
}

#[derive(Serialize, Deserialize)]
pub struct CellConfigNode {
    pub public_key: String,
    pub addresses: Vec<String>,
}
