use super::Error;
use crate::protos::generated::exocore_core::{CellNodeConfig, LocalNodeConfig};
use std::fs::OpenOptions;
use std::path::Path;

pub fn node_config_from_yaml_file<P: AsRef<Path>>(path: P) -> Result<LocalNodeConfig, Error> {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(path.as_ref())
        .map_err(Error::ConfigIO)
        .map_err(|err| Error::Config(format!("Couldn't open YAML node file: {}", err)))?;

    node_config_from_yaml(file)
}

pub fn node_config_from_yaml<R: std::io::Read>(bytes: R) -> Result<LocalNodeConfig, Error> {
    let config = serde_yaml::from_reader(bytes)
        .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

    Ok(config)
}

pub fn node_config_from_json<R: std::io::Read>(bytes: R) -> Result<LocalNodeConfig, Error> {
    let config = serde_json::from_reader(bytes)
        .map_err(|err| Error::Config(format!("Couldn't decode JSON node config: {}", err)))?;

    Ok(config)
}

pub fn cell_config_from_yaml<R: std::io::Read>(bytes: R) -> Result<CellNodeConfig, Error> {
    let config = serde_yaml::from_reader(bytes)
        .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::super::{Cell, CellNodeRole, CellNodes};
    use super::*;
    use crate::protos::generated::exocore_core::{
        cell_node_config, CellConfig, CellNodeConfig, LocalNodeConfig, NodeConfig,
    };
    use crate::tests_utils::root_test_fixtures_path;

    #[test]
    fn parse_node_config_yaml_ser_deser() -> Result<(), failure::Error> {
        let conf_ser = LocalNodeConfig {
            keypair: "keypair".to_string(),
            public_key: "pk".to_string(),
            name: "node_name".to_string(),
            cells: vec![CellConfig {
                public_key: "pk".to_string(),
                keypair: "kp".to_string(),
                name: "cell_name".to_string(),
                data_directory: "data".to_string(),
                nodes: vec![CellNodeConfig {
                    node: Some(NodeConfig {
                        public_key: "pk".to_string(),
                        name: "node_name".to_string(),
                        addresses: vec!["maddr".to_string()],
                    }),
                    roles: vec![cell_node_config::Role::InvalidRole.into()],
                }],
            }],
            listen_addresses: vec!["maddr".to_string()],
        };

        let yaml = serde_yaml::to_string(&conf_ser)?;
        // println!("{}", yaml);

        let conf_deser = node_config_from_yaml(yaml.as_bytes())?;

        assert_eq!(conf_ser, conf_deser);

        Ok(())
    }

    #[test]
    fn parse_node_config_example_yaml_file() -> Result<(), failure::Error> {
        let config_path = root_test_fixtures_path("examples/config.yaml");
        let config = node_config_from_yaml_file(config_path)?;

        let (cells, node) = Cell::new_from_local_node_config(config)?;
        assert_eq!(1, cells.len());
        assert_eq!(2, node.addresses().len());

        let full_cell = cells.first().cloned().unwrap().unwrap_full();

        {
            let nodes = full_cell.nodes();
            assert_eq!(1, nodes.count());

            let nodes_iter = nodes.iter();
            let node = nodes_iter.all().next().unwrap();
            assert_eq!(2, node.roles().len());
        }

        Ok(())
    }

    #[test]
    pub fn parse_node_config_from_yaml() -> Result<(), failure::Error> {
        let yaml = r#"
name: node name
keypair: ae2oiM2PYznyfqEMPraKbpAuA8LWVhPUiUTgdwjvnwbDjnz9W9FAiE9431NtVjfBaX44nPPoNR8Mv6iYcJdqSfp8eZ
public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP

listen_addresses:
  - /ip4/0.0.0.0/tcp/3330
  - /ip4/0.0.0.0/tcp/3341/ws

cells:
   - public_key: pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ
     keypair: ""
     name: ""
     data_directory: target/data/cell1
     nodes:
       - node:
             name: node name
             public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP
             addresses:
                - /ip4/192.168.2.67/tcp/3330
         roles:
           - 1
"#;

        let config = node_config_from_yaml(yaml.as_bytes())?;

        let (cells, node) = Cell::new_from_local_node_config(config)?;
        assert_eq!(1, cells.len());
        assert_eq!(2, node.addresses().len());

        let cell = cells.first().cloned().unwrap().unwrap_cell();

        {
            let nodes = cell.nodes();
            assert_eq!(1, nodes.count());

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

            assert_eq!(1, node.node().addresses().len());
        }

        {
            assert!(cell.local_node_has_role(CellNodeRole::Data));
            assert!(!cell.local_node_has_role(CellNodeRole::IndexStore));
        }

        Ok(())
    }
}
