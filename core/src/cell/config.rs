use super::Error;
use crate::protos::generated::exocore_core::{
    node_cell_config, CellConfig, CellNodeConfig, LocalNodeConfig, NodeCellConfig,
};
use crate::{protos::generated::exocore_apps::Manifest, utils::path::child_to_abs_path};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

pub fn node_config_from_yaml_file<P: AsRef<Path>>(path: P) -> Result<LocalNodeConfig, Error> {
    let file = File::open(path.as_ref()).map_err(|err| {
        Error::Config(format!(
            "Couldn't open YAML node file at path '{:?}': {}",
            path.as_ref(),
            err
        ))
    })?;

    let mut config = node_config_from_yaml(file)?;

    if config.path.is_empty() {
        let mut node_path = path.as_ref().to_path_buf();
        node_path.pop();
        config.path = node_path.to_string_lossy().to_string();
    }

    Ok(config)
}

pub fn node_config_to_standalone(mut config: LocalNodeConfig) -> Result<LocalNodeConfig, Error> {
    let mut cells = Vec::new();
    for node_cell_config in &config.cells {
        let cell_config = cell_config_from_node_cell(node_cell_config, &config)?;
        let cell_config = cell_config_to_standalone(cell_config)?;

        let mut node_cell_config = node_cell_config.clone();
        node_cell_config.location = Some(node_cell_config::Location::Instance(cell_config));

        cells.push(node_cell_config);
    }

    config.cells = cells;

    Ok(config)
}

pub fn node_config_from_yaml<R: std::io::Read>(bytes: R) -> Result<LocalNodeConfig, Error> {
    let config = serde_yaml::from_reader(bytes)
        .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

    Ok(config)
}

pub fn node_config_to_yaml(config: &LocalNodeConfig) -> Result<String, Error> {
    serde_yaml::to_string(config)
        .map_err(|err| Error::Config(format!("Couldn't encode node config to YAML: {}", err)))
}

pub fn node_config_from_json<R: std::io::Read>(bytes: R) -> Result<LocalNodeConfig, Error> {
    let config = serde_json::from_reader(bytes)
        .map_err(|err| Error::Config(format!("Couldn't decode JSON node config: {}", err)))?;

    Ok(config)
}

pub fn node_config_to_json(config: &LocalNodeConfig) -> Result<String, Error> {
    serde_json::to_string_pretty(config)
        .map_err(|err| Error::Config(format!("Couldn't encode node config to JSON: {}", err)))
}

pub fn cell_config_from_yaml_file<P: AsRef<Path>>(path: P) -> Result<CellConfig, Error> {
    let file = File::open(path.as_ref()).map_err(|err| {
        Error::Config(format!(
            "Couldn't open YAML cell config at path '{:?}': {}",
            path.as_ref(),
            err
        ))
    })?;

    let mut config: CellConfig = serde_yaml::from_reader(file)
        .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

    if config.path.is_empty() {
        let mut node_path = path.as_ref().to_path_buf();
        node_path.pop();
        config.path = node_path.to_string_lossy().to_string();
    }

    Ok(config)
}

pub fn cell_config_from_node_cell(
    config: &NodeCellConfig,
    node_config: &LocalNodeConfig,
) -> Result<CellConfig, Error> {
    match &config.location {
        Some(node_cell_config::Location::Instance(cell_config)) => {
            let mut cell_config = cell_config.clone();
            cell_config.path = node_config.path.clone();
            Ok(cell_config)
        }
        Some(node_cell_config::Location::Directory(directory)) => {
            let mut config_path = child_to_abs_path(&node_config.path, directory);
            config_path.push("cell.yaml");

            cell_config_from_yaml_file(config_path)
        }
        other => Err(Error::Config(format!(
            "Invalid cell instance config: {:?}",
            other
        ))),
    }
}

pub fn cell_config_to_standalone(mut config: CellConfig) -> Result<CellConfig, Error> {
    for app in config.apps.iter_mut() {
        let mut final_manifest = match app.location.take() {
            Some(crate::protos::core::cell_application_config::Location::Directory(dir)) => {
                let absolute_path = child_to_abs_path(&config.path, &dir)
                    .to_string_lossy()
                    .to_string();

                let mut manifest_path = PathBuf::from(&absolute_path);
                manifest_path.push("app.yaml");
                let mut manifest = app_manifest_from_yaml_file(manifest_path)?;
                manifest.path = absolute_path;
                manifest
            }
            Some(crate::protos::core::cell_application_config::Location::Instance(manifest)) => {
                manifest
            }
            other => {
                return Err(Error::Application(
                    String::new(),
                    format!("Unsupported application location: {:?}", other),
                ));
            }
        };

        let app_name = final_manifest.name.clone();

        for schema in final_manifest.schemas.iter_mut() {
            let final_source = match schema.source.take() {
                Some(crate::protos::apps::manifest_schema::Source::File(schema_path)) => {
                    let abs_schema_path = child_to_abs_path(&final_manifest.path, &schema_path)
                        .to_string_lossy()
                        .to_string();
                    let mut file = File::open(&abs_schema_path).map_err(|err| {
                        Error::Application(
                            app_name.clone(),
                            format!(
                                "Couldn't open application schema at path '{:?}': {}",
                                abs_schema_path, err
                            ),
                        )
                    })?;

                    let mut content = vec![];
                    file.read_to_end(&mut content).map_err(|err| {
                        Error::Application(
                            app_name.clone(),
                            format!(
                                "Couldn't read application schema at path '{:?}': {}",
                                abs_schema_path, err
                            ),
                        )
                    })?;
                    crate::protos::apps::manifest_schema::Source::Bytes(content)
                }
                Some(src @ crate::protos::apps::manifest_schema::Source::Bytes(_)) => src,
                other => {
                    return Err(Error::Application(
                        config.name.clone(),
                        format!("Unsupported application schema type: {:?}", other),
                    ));
                }
            };

            schema.source = Some(final_source);
        }

        app.location =
            Some(crate::protos::core::cell_application_config::Location::Instance(final_manifest));
    }

    Ok(config)
}

pub fn cell_config_from_yaml<R: std::io::Read>(bytes: R) -> Result<CellNodeConfig, Error> {
    let config = serde_yaml::from_reader(bytes)
        .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

    Ok(config)
}

pub fn app_manifest_from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Manifest, Error> {
    let path = path.as_ref();

    let file = File::open(path).map_err(|err| {
        Error::Application(
            String::new(),
            format!(
                "Couldn't open application manifest at path '{:?}': {}",
                path, err
            ),
        )
    })?;

    let manifest = serde_yaml::from_reader(file).map_err(|err| {
        Error::Application(
            String::new(),
            format!("Couldn't decode YAML manifest at path {:?}: {}", path, err),
        )
    })?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::super::{Cell, CellNodeRole, CellNodes};
    use super::*;
    use crate::protos::{
        apps::manifest_schema,
        core::cell_application_config,
        core::NodeAddresses,
        generated::exocore_core::{
            cell_node_config, node_cell_config, CellConfig, CellNodeConfig, LocalNodeConfig,
            NodeCellConfig, NodeConfig,
        },
    };
    use crate::tests_utils::root_test_fixtures_path;

    #[test]
    fn parse_node_config_yaml_ser_deser() -> anyhow::Result<()> {
        let conf_ser = LocalNodeConfig {
            keypair: "keypair".to_string(),
            public_key: "pk".to_string(),
            name: "node_name".to_string(),
            id: String::new(),
            path: "path".to_string(),
            cells: vec![NodeCellConfig {
                location: Some(node_cell_config::Location::Instance(CellConfig {
                    public_key: "pk".to_string(),
                    keypair: "kp".to_string(),
                    name: "cell_name".to_string(),
                    id: String::new(),
                    path: "path".to_string(),
                    nodes: vec![CellNodeConfig {
                        node: Some(NodeConfig {
                            public_key: "pk".to_string(),
                            name: "node_name".to_string(),
                            id: String::new(),
                            addresses: Some(NodeAddresses {
                                p2p: vec!["maddr".to_string()],
                                http: vec!["httpaddr".to_string()],
                            }),
                        }),
                        roles: vec![cell_node_config::Role::InvalidRole.into()],
                    }],
                    apps: vec![],
                })),
            }],
            addresses: Some(NodeAddresses {
                p2p: vec!["maddr".to_string()],
                http: vec!["httpaddr".to_string()],
            }),
        };

        let yaml = serde_yaml::to_string(&conf_ser)?;
        // println!("{}", yaml);

        let conf_deser = node_config_from_yaml(yaml.as_bytes())?;

        assert_eq!(conf_ser, conf_deser);

        Ok(())
    }

    #[test]
    fn parse_node_config_example_yaml_file() -> anyhow::Result<()> {
        let config_path = root_test_fixtures_path("examples/config.yaml");
        let config = node_config_from_yaml_file(config_path)?;

        let (cells, node) = Cell::new_from_local_node_config(config)?;
        assert_eq!(2, cells.len());
        assert_eq!(2, node.p2p_addresses().len());

        let full_cell = cells.first().cloned().unwrap().unwrap_full();

        {
            let nodes = full_cell.nodes();
            assert_eq!(2, nodes.count());

            let nodes_iter = nodes.iter();
            let node = nodes_iter.with_role(CellNodeRole::Store).next().unwrap();
            assert_eq!(2, node.roles().len());
        }

        {
            let schemas = full_cell
                .schemas()
                .get_message_descriptor("exocore.example_app.Task");
            assert!(schemas.is_ok());
        }

        Ok(())
    }

    #[test]
    fn test_node_config_to_standalone() -> anyhow::Result<()> {
        let config_path = root_test_fixtures_path("examples/config.yaml");
        let config = node_config_from_yaml_file(config_path)?;

        let standalone_config = node_config_to_standalone(config).unwrap();

        fn validate_node_cell_config(node_cell_config: &NodeCellConfig) {
            match node_cell_config.location.as_ref() {
                Some(node_cell_config::Location::Instance(cell_config)) => {
                    validate_cell(cell_config);
                }
                other => panic!("Expected cell to be an instance location, got: {:?}", other),
            }
        }

        fn validate_cell(cell_config: &CellConfig) {
            for cell_app_config in &cell_config.apps {
                match cell_app_config.location.as_ref() {
                    Some(cell_application_config::Location::Instance(app_manifest)) => {
                        validate_app(app_manifest);
                    }
                    other => panic!("Expected app to be an instance location, got: {:?}", other),
                }
            }
        }

        fn validate_app(app_manifest: &Manifest) {
            for schema in &app_manifest.schemas {
                match schema.source.as_ref() {
                    Some(manifest_schema::Source::Bytes(_)) => {}
                    other => panic!(
                        "Expected app schema to be in bytes format, got: {:?}",
                        other
                    ),
                }
            }
        }

        for cell in &standalone_config.cells {
            validate_node_cell_config(cell);
        }

        // should be able to load cell standalone
        assert!(Cell::new_from_local_node_config(standalone_config).is_ok());

        Ok(())
    }

    #[test]
    pub fn parse_node_config_from_yaml() -> anyhow::Result<()> {
        let yaml = r#"
name: node name
keypair: ae2oiM2PYznyfqEMPraKbpAuA8LWVhPUiUTgdwjvnwbDjnz9W9FAiE9431NtVjfBaX44nPPoNR8Mv6iYcJdqSfp8eZ
public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP

addresses:
  p2p:
    - /ip4/0.0.0.0/tcp/3330
    - /ip4/0.0.0.0/tcp/3341/ws
  http:
    - http://0.0.0.0:8080

cells:
   - location:
        Instance:
             public_key: pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ
             keypair: ""
             name: ""
             data_directory: target/data/cell1
             nodes:
               - node:
                   name: node name
                   public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP
                   addresses:
                     p2p:
                       - /ip4/192.168.2.67/tcp/3330
                     http:
                       - http://192.168.2.67:8080
                 roles:
                   - 1
             apps:
               - location:
                   Instance:
                     name: some application
                     public_key: peHZC1CM51uAugeMNxbXkVukFzCwMJY52m1xDCfLmm1pc1
"#;

        let config = node_config_from_yaml(yaml.as_bytes())?;

        let (cells, node) = Cell::new_from_local_node_config(config)?;
        assert_eq!(1, cells.len());
        assert_eq!(2, node.p2p_addresses().len());
        assert_eq!(1, node.http_addresses().len());

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
                "12D3KooWQSm3A1DZBMHmWVu3g7NonTMTenGQccmY9bUWtVjWTQ5K",
                node.node().id().to_string()
            );

            assert_eq!(1, node.node().p2p_addresses().len());
            assert_eq!(1, node.node().http_addresses().len());
        }

        {
            assert!(cell.local_node_has_role(CellNodeRole::Chain));
            assert!(!cell.local_node_has_role(CellNodeRole::Store));
        }

        Ok(())
    }

    #[test]
    pub fn parse_node_optional_fields_yaml() -> anyhow::Result<()> {
        let yaml = r#"
keypair: ae2oiM2PYznyfqEMPraKbpAuA8LWVhPUiUTgdwjvnwbDjnz9W9FAiE9431NtVjfBaX44nPPoNR8Mv6iYcJdqSfp8eZ
public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP

addresses:
  p2p:
    - /ip4/0.0.0.0/tcp/3330
    - /ip4/0.0.0.0/tcp/3341/ws

cells:
   - location:
       Instance:
           public_key: pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ
           nodes:
             - node:
                   public_key: peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP
                   addresses:
                      p2p:
                         - /ip4/192.168.2.67/tcp/3330
"#;

        node_config_from_yaml(yaml.as_bytes())?;

        Ok(())
    }
}
