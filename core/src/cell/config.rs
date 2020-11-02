use super::Error;
use crate::{
    protos::generated::exocore_core::{
        node_cell_config, CellConfig, CellNodeConfig, LocalNodeConfig, NodeCellConfig,
    },
    protos::{core::cell_application_config, generated::exocore_apps::Manifest},
    utils::path::{child_to_abs_path, child_to_abs_path_string},
};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

/// Extension for `LocalNodeConfig` proto.
pub trait LocalNodeConfigExt {
    fn config(&self) -> &LocalNodeConfig;
    fn from_yaml_reader<R: Read>(reader: R) -> Result<LocalNodeConfig, Error>;
    fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<LocalNodeConfig, Error>;
    fn from_json_reader<R: Read>(bytes: R) -> Result<LocalNodeConfig, Error>;
    fn to_yaml(&self) -> Result<String, Error>;
    fn to_yaml_writer<W: Write>(&self, write: W) -> Result<(), Error>;
    fn to_json(&self) -> Result<String, Error>;
    fn to_standalone(&self) -> Result<LocalNodeConfig, Error>;
}

impl LocalNodeConfigExt for LocalNodeConfig {
    fn config(&self) -> &LocalNodeConfig {
        self
    }

    fn from_yaml_reader<R: Read>(reader: R) -> Result<LocalNodeConfig, Error> {
        let config = serde_yaml::from_reader(reader)
            .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

        Ok(config)
    }

    fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<LocalNodeConfig, Error> {
        let file = File::open(path.as_ref()).map_err(|err| {
            Error::Config(format!(
                "Couldn't open YAML node file at path '{:?}': {}",
                path.as_ref(),
                err
            ))
        })?;

        let mut config = Self::from_yaml_reader(file)?;
        resolve_local_node_config_paths(&mut config, path.as_ref());

        Ok(config)
    }

    fn from_json_reader<R: Read>(bytes: R) -> Result<LocalNodeConfig, Error> {
        let config = serde_json::from_reader(bytes)
            .map_err(|err| Error::Config(format!("Couldn't decode JSON node config: {}", err)))?;

        Ok(config)
    }

    fn to_yaml(&self) -> Result<String, Error> {
        serde_yaml::to_string(self.config())
            .map_err(|err| Error::Config(format!("Couldn't encode node config to YAML: {}", err)))
    }

    fn to_yaml_writer<W: Write>(&self, write: W) -> Result<(), Error> {
        serde_yaml::to_writer(write, self.config())
            .map_err(|err| Error::Config(format!("Couldn't encode node config to YAML: {}", err)))
    }

    fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self.config())
            .map_err(|err| Error::Config(format!("Couldn't encode node config to JSON: {}", err)))
    }

    fn to_standalone(&self) -> Result<LocalNodeConfig, Error> {
        let mut config = self.config().clone();

        let mut cells = Vec::new();
        for node_cell_config in &config.cells {
            let cell_config = CellConfig::from_node_cell(node_cell_config, &config)?;
            let cell_config = cell_config.to_standalone()?;

            let mut node_cell_config = node_cell_config.clone();
            node_cell_config.location = Some(node_cell_config::Location::Instance(cell_config));

            cells.push(node_cell_config);
        }

        config.cells = cells;

        Ok(config)
    }
}

/// Extension for `CellNodeConfig` proto.
pub trait CellNodeConfigExt {
    fn config(&self) -> &CellNodeConfig;

    fn from_yaml<R: Read>(bytes: R) -> Result<CellNodeConfig, Error>;
}

impl CellNodeConfigExt for CellNodeConfig {
    fn config(&self) -> &CellNodeConfig {
        self
    }

    fn from_yaml<R: Read>(bytes: R) -> Result<CellNodeConfig, Error> {
        let config = serde_yaml::from_reader(bytes)
            .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

        Ok(config)
    }
}

pub trait CellConfigExt {
    fn config(&self) -> &CellConfig;

    fn from_yaml_reader<R: Read>(bytes: R) -> Result<CellConfig, Error>;

    fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<CellConfig, Error>;

    fn to_standalone(&self) -> Result<CellConfig, Error>;

    fn to_yaml(&self) -> Result<String, Error>;

    fn to_yaml_writer<W: Write>(&self, write: W) -> Result<(), Error>;

    fn from_node_cell(
        config: &NodeCellConfig,
        node_config: &LocalNodeConfig,
    ) -> Result<CellConfig, Error>;
}

impl CellConfigExt for CellConfig {
    fn config(&self) -> &CellConfig {
        self
    }

    fn from_yaml_reader<R: Read>(bytes: R) -> Result<CellConfig, Error> {
        let config: CellConfig = serde_yaml::from_reader(bytes)
            .map_err(|err| Error::Config(format!("Couldn't decode YAML cell config: {}", err)))?;

        Ok(config)
    }

    fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<CellConfig, Error> {
        let file = File::open(path.as_ref()).map_err(|err| {
            Error::Config(format!(
                "Couldn't open YAML cell config at path '{:?}': {}",
                path.as_ref(),
                err
            ))
        })?;

        let mut cell_config = Self::from_yaml_reader(file)?;
        resolve_cell_config_paths(&mut cell_config, path.as_ref());

        Ok(cell_config)
    }

    fn to_standalone(&self) -> Result<CellConfig, Error> {
        let mut config = self.config().clone();

        for app in config.apps.iter_mut() {
            let mut final_manifest = match app.location.take() {
                Some(crate::protos::core::cell_application_config::Location::Directory(dir)) => {
                    let absolute_path = child_to_abs_path_string(&config.path, &dir);

                    let mut manifest_path = PathBuf::from(&absolute_path);
                    manifest_path.push("app.yaml");
                    let mut manifest = Manifest::read_yaml_file(manifest_path)?;
                    manifest.path = absolute_path;
                    manifest
                }
                Some(crate::protos::core::cell_application_config::Location::Instance(
                    manifest,
                )) => manifest,
                other => {
                    return Err(Error::Application(
                        "unnamed".to_string(),
                        format!("Unsupported application location: {:?}", other),
                    ));
                }
            };

            let app_name = final_manifest.name.clone();

            for schema in final_manifest.schemas.iter_mut() {
                let final_source = match schema.source.take() {
                    Some(crate::protos::apps::manifest_schema::Source::File(schema_path)) => {
                        let abs_schema_path =
                            child_to_abs_path_string(&final_manifest.path, &schema_path);
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

            app.location = Some(
                crate::protos::core::cell_application_config::Location::Instance(final_manifest),
            );
        }

        Ok(config)
    }

    fn to_yaml(&self) -> Result<String, Error> {
        serde_yaml::to_string(self.config())
            .map_err(|err| Error::Config(format!("Couldn't encode cell config to YAML: {}", err)))
    }

    fn to_yaml_writer<W: Write>(&self, write: W) -> Result<(), Error> {
        serde_yaml::to_writer(write, self.config())
            .map_err(|err| Error::Config(format!("Couldn't encode cell config to YAML: {}", err)))
    }

    fn from_node_cell(
        config: &NodeCellConfig,
        node_config: &LocalNodeConfig,
    ) -> Result<CellConfig, Error> {
        match &config.location {
            Some(node_cell_config::Location::Instance(cell_config)) => {
                let mut cell_config = cell_config.clone();
                cell_config.path = child_to_abs_path_string(&node_config.path, cell_config.path);
                Ok(cell_config)
            }
            Some(node_cell_config::Location::Directory(directory)) => {
                let mut config_path = child_to_abs_path(&node_config.path, directory);
                config_path.push("cell.yaml");

                Self::from_yaml_file(config_path)
            }
            other => Err(Error::Config(format!(
                "Invalid cell instance config: {:?}",
                other
            ))),
        }
    }
}

/// Extension for `Manifest` proto.
pub trait ManifestExt {
    fn manifest(&self) -> &Manifest;

    fn read_yaml_file<P: AsRef<Path>>(path: P) -> Result<Manifest, Error>;
}

impl ManifestExt for Manifest {
    fn manifest(&self) -> &Manifest {
        self
    }

    fn read_yaml_file<P: AsRef<Path>>(path: P) -> Result<Manifest, Error> {
        let path = path.as_ref();

        let file = File::open(path).map_err(|err| {
            Error::Application(
                "unnamed".to_string(),
                format!(
                    "Couldn't open application manifest at path '{:?}': {}",
                    path, err
                ),
            )
        })?;

        let mut manifest = serde_yaml::from_reader::<_, Manifest>(file).map_err(|err| {
            Error::Application(
                "unnamed".to_string(),
                format!(
                    "Couldn't decode YAML manifest at path '{:?}': {}",
                    path, err
                ),
            )
        })?;

        let file_directory = path.parent().unwrap_or(path);
        manifest.path = child_to_abs_path_string(file_directory, &manifest.path);

        Ok(manifest)
    }
}

/// Resolves all paths in a `LocalNodeConfig` to absolute paths using the file path where it was read from.
fn resolve_local_node_config_paths(config: &mut LocalNodeConfig, config_file_path: &Path) {
    let config_directory = config_file_path.parent().unwrap_or(config_file_path);

    config.path = child_to_abs_path_string(&config_directory, &config.path);

    for cell in &mut config.cells {
        match cell.location.as_mut() {
            Some(node_cell_config::Location::Instance(cell)) => {
                resolve_cell_config_paths(cell, config_file_path);
            }
            Some(node_cell_config::Location::Directory(path)) => {
                *path = child_to_abs_path_string(&config.path, path.clone());
            }
            _ => {}
        }
    }
}

/// Resolves all paths in a `CellConfig` to absolute paths using the file path where it was read from.
fn resolve_cell_config_paths(cell_config: &mut CellConfig, config_file_path: &Path) {
    let config_directory = config_file_path.parent().unwrap_or(config_file_path);

    cell_config.path = child_to_abs_path_string(&config_directory, &cell_config.path);

    for app in &mut cell_config.apps {
        match app.location.as_mut() {
            Some(cell_application_config::Location::Instance(app_manifest)) => {
                app_manifest.path = child_to_abs_path_string(&cell_config.path, &app_manifest.path);
            }
            Some(cell_application_config::Location::Directory(path)) => {
                *path = child_to_abs_path_string(&cell_config.path, path.clone());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::SeekFrom;

    use super::super::{Cell, CellNodeRole, CellNodes};
    use super::*;
    use crate::protos::{
        apps::manifest_schema,
        core::NodeAddresses,
        core::{cell_application_config, CellApplicationConfig},
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
            cells: vec![
                NodeCellConfig {
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
                            roles: vec![cell_node_config::Role::ChainRole.into()],
                        }],
                        apps: vec![
                            CellApplicationConfig {
                                location: Some(cell_application_config::Location::Instance(
                                    Manifest {
                                        name: "name".to_string(),
                                        ..Default::default()
                                    },
                                )),
                            },
                            CellApplicationConfig {
                                location: Some(cell_application_config::Location::Directory(
                                    "some_path".to_string(),
                                )),
                            },
                        ],
                    })),
                },
                NodeCellConfig {
                    location: Some(node_cell_config::Location::Directory(
                        "some_path".to_string(),
                    )),
                },
            ],
            addresses: Some(NodeAddresses {
                p2p: vec!["maddr".to_string()],
                http: vec!["httpaddr".to_string()],
            }),
        };

        let conf_yaml = conf_ser.to_yaml()?;
        let conf_deser = LocalNodeConfig::from_yaml_reader(conf_yaml.as_bytes())?;
        assert_eq!(conf_ser, conf_deser);

        Ok(())
    }

    #[test]
    fn parse_node_config_example_yaml_file() -> anyhow::Result<()> {
        let config_path = root_test_fixtures_path("examples/node.yaml");
        let config = LocalNodeConfig::from_yaml_file(config_path)?;

        let (cells, node) = Cell::new_from_local_node_config(config)?;
        assert_eq!(2, cells.len());
        assert_eq!(2, node.p2p_addresses().len());

        {
            // inlined cell
            let cell = cells[1].clone().unwrap_full();

            {
                let nodes = cell.nodes();
                assert_eq!(2, nodes.count());

                let nodes_iter = nodes.iter();
                let node = nodes_iter.with_role(CellNodeRole::Store).next().unwrap();
                assert_eq!(2, node.roles().len());
            }

            {
                let schemas = cell
                    .schemas()
                    .get_message_descriptor("exocore.example_app.Task");
                assert!(schemas.is_ok());
            }
        }

        {
            // cell from directory
            let cell = cells[1].clone().unwrap_full();

            {
                let nodes = cell.nodes();
                assert_eq!(2, nodes.count());
            }

            {
                let schemas = cell
                    .schemas()
                    .get_message_descriptor("exocore.example_app.Task");
                assert!(schemas.is_ok());
            }
        }

        Ok(())
    }

    #[test]
    fn write_node_config_yaml_file() -> anyhow::Result<()> {
        let config_init = LocalNodeConfig {
            name: "node_name".to_string(),
            ..Default::default()
        };

        let mut file = tempfile::tempfile()?;
        config_init.to_yaml_writer(&mut file)?;

        file.seek(SeekFrom::Start(0))?;

        let config_read = LocalNodeConfig::from_yaml_reader(file)?;

        assert_eq!(config_init, config_read);

        Ok(())
    }

    #[test]
    fn node_config_to_standalone() -> anyhow::Result<()> {
        let config_path = root_test_fixtures_path("examples/node.yaml");
        let config = LocalNodeConfig::from_yaml_file(config_path)?;

        let standalone_config = config.to_standalone().unwrap();

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
  - instance:
      public_key: pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ
      keypair: ""
      name: ""
      path: target/data/cell1
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
        - instance:
             name: some application
             public_key: peHZC1CM51uAugeMNxbXkVukFzCwMJY52m1xDCfLmm1pc1
"#;

        let config = LocalNodeConfig::from_yaml_reader(yaml.as_bytes())?;

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
}
