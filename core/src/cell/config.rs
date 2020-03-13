use super::Error;
use super::{Cell, FullCell, LocalNode, Node};
use crate::cell::CellNode;
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

                let mut cell_node = CellNode::new(node);
                for node_role in &nodes_config.roles {
                    cell_node.add_role(node_role.parse().map_err(|err| {
                        Error::ConfigOther(format!("Couldn't parse node role: {}", err))
                    })?);
                }

                cell_nodes.add_cell_node(cell_node);
            }
        }

        Ok((full_cell, cell))
    }
}

#[derive(Serialize, Deserialize)]
pub struct CellConfigNode {
    pub public_key: String,
    pub addresses: Vec<String>,
    pub roles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::super::CellNodes;
    use super::*;
    use crate::cell::nodes::CellNodeRole;

    #[test]
    pub fn parse_node_config_from_file_full_cell() -> Result<(), failure::Error> {
        // hacky way of getting the files as the test execution may not be done in the same dir
        let config = NodeConfig::from_yaml_file("./examples/config.yaml")
            .or_else(|_| NodeConfig::from_yaml_file("../examples/config.yaml"))
            .or_else(|_| NodeConfig::from_yaml_file("../../examples/config.yaml"))?;

        let node = config.create_local_node()?;
        assert_eq!(2, node.addresses().len());

        let (full_cell, _cell) = config.cells.first().unwrap().create_cell(&node)?;
        let full_cell = full_cell.expect("Expected the cell to be a full cell");

        {
            let nodes = full_cell.nodes();
            assert_eq!(1, nodes.len());

            let nodes_iter = nodes.iter();
            let node = nodes_iter.all().next().unwrap();
            assert_eq!(2, node.roles().len());
        }

        Ok(())
    }

    #[test]
    pub fn parse_node_config_from_str_read_cell() -> Result<(), failure::Error> {
        let yaml = r#"
node_keypair: ae2oiM2PYznyfqEMPraKbpAuA8LWVhPUiUTgdwjvnwbDjnz9W9FAiE9431NtVjfBaX44nPPoNR8Mv6iYcJdqSfp8eZ
node_public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP

listen_addresses:
  - /ip4/0.0.0.0/tcp/3330
  - /ip4/0.0.0.0/tcp/3341/ws

cells:
   - public_key: pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ
     data_directory: target/data/cell1
     nodes:
       - public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP
         addresses:
           - /ip4/192.168.2.67/tcp/3330
         roles:
           - data
"#;

        let config = NodeConfig::from_yaml_reader(yaml.as_bytes())?;
        let node = config.create_local_node()?;

        let (full_cell, cell) = config.cells[0].create_cell(&node)?;
        assert!(full_cell.is_none());

        {
            let nodes = cell.nodes();
            assert_eq!(1, nodes.len());

            let nodes_iter = nodes.iter();
            let node = nodes_iter.all().next().unwrap();

            assert_eq!(
                "peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP",
                node.node().public_key().encode_base58_string()
            );

            // libp2p's PeerId
            assert_eq!(
                "QmQCewLJsDyEyubzHF67LsFFtChBdRdumeQyPwMhDVqLzk",
                node.node().id().to_string()
            );
        }

        {
            assert!(cell.local_node_has_role(CellNodeRole::Data));
            assert!(!cell.local_node_has_role(CellNodeRole::IndexStore));
        }

        Ok(())
    }
}
