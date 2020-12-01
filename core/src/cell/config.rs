use super::Error;
use crate::{
    protos::{
        apps::manifest_schema,
        core::{cell_application_config, NodeConfig},
        generated::{
            exocore_apps::Manifest,
            exocore_core::{
                node_cell_config, CellConfig, CellNodeConfig, LocalNodeConfig, NodeCellConfig,
            },
        },
    },
    utils::path::{child_to_abs_path_string, child_to_relative_path_string},
};
use std::{
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

/// Extension for `LocalNodeConfig` proto.
pub trait LocalNodeConfigExt {
    fn config(&self) -> &LocalNodeConfig;

    fn from_yaml_reader<R: Read>(reader: R) -> Result<LocalNodeConfig, Error>;

    fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<LocalNodeConfig, Error>;

    fn from_json_reader<R: Read>(bytes: R) -> Result<LocalNodeConfig, Error>;

    fn to_yaml(&self) -> Result<String, Error>;

    fn to_yaml_writer<W: Write>(&self, write: W) -> Result<(), Error>;

    fn to_yaml_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error>;

    fn to_json(&self) -> Result<String, Error>;

    fn inlined(&self) -> Result<LocalNodeConfig, Error>;

    fn make_absolute_paths<P: AsRef<Path>>(&mut self, directory: P);

    fn make_relative_paths<P: AsRef<Path>>(&mut self, directory: P);

    fn add_cell(&mut self, cell: NodeCellConfig);
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

        let directory = path
            .as_ref()
            .parent()
            .expect("Couldn't get enclosing directory of yaml file");

        let mut config = Self::from_yaml_reader(file)?;
        config.make_absolute_paths(directory);

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

    fn to_yaml_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let file = File::create(path.as_ref()).map_err(|err| {
            Error::Config(format!(
                "Couldn't create YAML node file at path '{:?}': {}",
                path.as_ref(),
                err
            ))
        })?;

        let directory = path
            .as_ref()
            .parent()
            .expect("Couldn't get enclosing directory of yaml file");

        let mut config = self.clone();
        config.make_relative_paths(directory);
        config.to_yaml_writer(file)?;

        Ok(())
    }

    fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self.config())
            .map_err(|err| Error::Config(format!("Couldn't encode node config to JSON: {}", err)))
    }

    fn inlined(&self) -> Result<LocalNodeConfig, Error> {
        let mut config = self.config().clone();
        config.path = String::new();

        let mut cells = Vec::new();
        for node_cell_config in &config.cells {
            let cell_config = CellConfig::from_node_cell(node_cell_config)?;
            let cell_config = cell_config.inlined()?;

            let mut node_cell_config = node_cell_config.clone();
            node_cell_config.location = Some(node_cell_config::Location::Inline(cell_config));

            cells.push(node_cell_config);
        }

        config.cells = cells;

        Ok(config)
    }

    fn make_absolute_paths<P: AsRef<Path>>(&mut self, directory: P) {
        self.path = child_to_abs_path_string(&directory, &self.path);

        for cell in &mut self.cells {
            match cell.location.as_mut() {
                Some(node_cell_config::Location::Inline(cell)) => {
                    cell.make_absolute_paths(directory.as_ref());
                }
                Some(node_cell_config::Location::Path(path)) => {
                    *path = child_to_abs_path_string(&self.path, path.clone());
                }
                _ => {}
            }
        }
    }

    fn make_relative_paths<P: AsRef<Path>>(&mut self, directory: P) {
        self.path = child_to_relative_path_string(&directory, &self.path);

        for cell in &mut self.cells {
            match cell.location.as_mut() {
                Some(node_cell_config::Location::Inline(cell)) => {
                    cell.make_relative_paths(directory.as_ref());
                }
                Some(node_cell_config::Location::Path(path)) => {
                    *path = child_to_relative_path_string(directory.as_ref(), path.clone());
                }
                _ => {}
            }
        }
    }

    fn add_cell(&mut self, cell: NodeCellConfig) {
        let mut current_position: Option<usize> = None;

        use node_cell_config::Location;
        for (idx, other_cell) in self.cells.iter().enumerate() {
            match (other_cell.location.as_ref(), &cell.location.as_ref()) {
                (Some(Location::Path(other_path)), Some(Location::Path(path))) => {
                    let abs_path = child_to_abs_path_string(&self.path, path);
                    if other_path == path || other_path == &abs_path {
                        current_position = Some(idx);
                        break;
                    }
                }
                (Some(Location::Inline(other_cell)), Some(Location::Inline(cell))) => {
                    if other_cell.id == cell.id {
                        current_position = Some(idx);
                        break;
                    }
                }
                _ => {}
            }
        }

        if let Some(current_position) = current_position {
            self.cells.remove(current_position);
        }

        self.cells.push(cell);
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
        let config = serde_yaml::from_reader(bytes).map_err(|err| {
            Error::Config(format!("Couldn't decode YAML cell node config: {}", err))
        })?;

        Ok(config)
    }
}

pub trait CellConfigExt {
    fn config(&self) -> &CellConfig;

    fn from_yaml<R: Read>(bytes: R) -> Result<CellConfig, Error>;

    fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<CellConfig, Error>;

    fn inlined(&self) -> Result<CellConfig, Error>;

    fn to_yaml(&self) -> Result<String, Error>;

    fn to_yaml_writer<W: Write>(&self, write: W) -> Result<(), Error>;

    fn to_yaml_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error>;

    fn from_node_cell(config: &NodeCellConfig) -> Result<CellConfig, Error>;

    fn make_absolute_paths<P: AsRef<Path>>(&mut self, directory: P);

    fn make_relative_paths<P: AsRef<Path>>(&mut self, directory: P);

    fn add_node(&mut self, node: CellNodeConfig);
}

impl CellConfigExt for CellConfig {
    fn config(&self) -> &CellConfig {
        self
    }

    fn from_yaml<R: Read>(bytes: R) -> Result<CellConfig, Error> {
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

        let directory = path
            .as_ref()
            .parent()
            .expect("Couldn't get enclosing directory of yaml file");

        let mut cell_config = Self::from_yaml(file)?;
        cell_config.make_absolute_paths(directory);

        Ok(cell_config)
    }

    fn inlined(&self) -> Result<CellConfig, Error> {
        let mut config = self.config().clone();
        config.path = String::new();

        for app in config.apps.iter_mut() {
            let app_manifest = match app.location.take() {
                Some(crate::protos::core::cell_application_config::Location::Path(dir)) => {
                    let mut manifest_path = PathBuf::from(dir);
                    manifest_path.push("app.yaml");
                    let mut manifest = Manifest::read_yaml_file(manifest_path)?;
                    manifest.path = String::new();
                    manifest
                }
                Some(crate::protos::core::cell_application_config::Location::Inline(manifest)) => {
                    manifest
                }
                other => {
                    return Err(Error::Application(
                        "unnamed".to_string(),
                        format!("Unsupported application location: {:?}", other),
                    ));
                }
            };

            let app_manifest = app_manifest.inlined()?;

            app.location =
                Some(crate::protos::core::cell_application_config::Location::Inline(app_manifest));
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

    fn to_yaml_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let file = File::create(path.as_ref()).map_err(|err| {
            Error::Config(format!(
                "Couldn't open YAML node file at path '{:?}': {}",
                path.as_ref(),
                err
            ))
        })?;

        let directory = path
            .as_ref()
            .parent()
            .expect("Couldn't get enclosing directory of yaml file");

        let mut config = self.clone();
        config.make_relative_paths(directory);
        config.to_yaml_writer(file)?;

        Ok(())
    }

    fn from_node_cell(config: &NodeCellConfig) -> Result<CellConfig, Error> {
        match &config.location {
            Some(node_cell_config::Location::Inline(cell_config)) => Ok(cell_config.clone()),
            Some(node_cell_config::Location::Path(directory)) => {
                let mut config_path = PathBuf::from(directory);
                config_path.push("cell.yaml");

                Self::from_yaml_file(config_path)
            }
            other => Err(Error::Config(format!(
                "Invalid cell instance config: {:?}",
                other
            ))),
        }
    }

    fn make_absolute_paths<P: AsRef<Path>>(&mut self, directory: P) {
        self.path = child_to_abs_path_string(directory.as_ref(), &self.path);

        for app in &mut self.apps {
            match app.location.as_mut() {
                Some(cell_application_config::Location::Inline(app_manifest)) => {
                    app_manifest.make_absolute_paths(directory.as_ref());
                }
                Some(cell_application_config::Location::Path(path)) => {
                    *path = child_to_abs_path_string(&self.path, path.clone());
                }
                _ => {}
            }
        }
    }

    fn make_relative_paths<P: AsRef<Path>>(&mut self, directory: P) {
        self.path = child_to_relative_path_string(directory.as_ref(), &self.path);

        for app in &mut self.apps {
            match app.location.as_mut() {
                Some(cell_application_config::Location::Inline(app_manifest)) => {
                    app_manifest.make_relative_paths(directory.as_ref());
                }
                Some(cell_application_config::Location::Path(path)) => {
                    *path = child_to_relative_path_string(directory.as_ref(), path.clone());
                }
                _ => {}
            }
        }
    }

    fn add_node(&mut self, node: CellNodeConfig) {
        let new_node_id = node.node.as_ref().map(|n| n.id.as_str());

        // check if node exists first
        for cell_node in &mut self.nodes {
            let is_node = {
                let node_id = cell_node.node.as_ref().map(|n| n.id.as_str());
                new_node_id == node_id
            };

            if is_node {
                *cell_node = node;
                return;
            }
        }

        // otherwise it doesn't exist, we just add it
        self.nodes.push(node);
    }
}

/// Extension for `NodeConfig` proto.
pub trait NodeConfigExt {
    fn from_yaml<R: Read>(bytes: R) -> Result<NodeConfig, Error>;

    fn to_yaml(&self) -> Result<String, Error>;
}

impl NodeConfigExt for NodeConfig {
    fn from_yaml<R: Read>(bytes: R) -> Result<NodeConfig, Error> {
        let config: NodeConfig = serde_yaml::from_reader(bytes)
            .map_err(|err| Error::Config(format!("Couldn't decode YAML node config: {}", err)))?;

        Ok(config)
    }

    fn to_yaml(&self) -> Result<String, Error> {
        serde_yaml::to_string(self).map_err(|err| {
            Error::Config(format!("Couldn't encode cell node config to YAML: {}", err))
        })
    }
}

/// Extension for `Manifest` proto.
pub trait ManifestExt {
    fn manifest(&self) -> &Manifest;

    fn inlined(&self) -> Result<Manifest, Error>;

    fn read_yaml_file<P: AsRef<Path>>(path: P) -> Result<Manifest, Error>;

    fn make_absolute_paths<P: AsRef<Path>>(&mut self, directory: P);

    fn make_relative_paths<P: AsRef<Path>>(&mut self, directory: P);
}

impl ManifestExt for Manifest {
    fn manifest(&self) -> &Manifest {
        self
    }

    fn inlined(&self) -> Result<Manifest, Error> {
        let mut app_manifest = self.manifest().clone();
        app_manifest.path = String::new();

        let app_name = app_manifest.name.clone();
        for schema in app_manifest.schemas.iter_mut() {
            let final_source = match schema.source.take() {
                Some(crate::protos::apps::manifest_schema::Source::File(schema_path)) => {
                    let mut file = File::open(&schema_path).map_err(|err| {
                        Error::Application(
                            app_name.clone(),
                            format!(
                                "Couldn't open application schema at path '{:?}': {}",
                                schema_path, err
                            ),
                        )
                    })?;

                    let mut content = vec![];
                    file.read_to_end(&mut content).map_err(|err| {
                        Error::Application(
                            app_name.clone(),
                            format!(
                                "Couldn't read application schema at path '{:?}': {}",
                                schema_path, err
                            ),
                        )
                    })?;
                    crate::protos::apps::manifest_schema::Source::Bytes(content)
                }
                Some(src @ crate::protos::apps::manifest_schema::Source::Bytes(_)) => src,
                other => {
                    return Err(Error::Application(
                        app_name,
                        format!("Unsupported application schema type: {:?}", other),
                    ));
                }
            };

            schema.source = Some(final_source);
        }

        Ok(app_manifest)
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
        manifest.make_absolute_paths(file_directory);

        Ok(manifest)
    }

    fn make_absolute_paths<P: AsRef<Path>>(&mut self, directory: P) {
        self.path = child_to_abs_path_string(directory.as_ref(), &self.path);

        for schema in &mut self.schemas {
            if let Some(manifest_schema::Source::File(path)) = schema.source.as_mut() {
                *path = child_to_abs_path_string(&directory, path.clone());
            }
        }
    }

    fn make_relative_paths<P: AsRef<Path>>(&mut self, directory: P) {
        self.path = child_to_relative_path_string(directory.as_ref(), &self.path);

        for schema in &mut self.schemas {
            if let Some(manifest_schema::Source::File(path)) = schema.source.as_mut() {
                *path = child_to_relative_path_string(&directory, path.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{Cell, CellNodeRole, CellNodes},
        *,
    };
    use crate::{
        protos::{
            apps::manifest_schema,
            core::{cell_application_config, CellApplicationConfig, NodeAddresses},
            generated::exocore_core::{
                cell_node_config, node_cell_config, CellConfig, CellNodeConfig, LocalNodeConfig,
                NodeCellConfig, NodeConfig,
            },
        },
        tests_utils::root_test_fixtures_path,
    };

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
                    location: Some(node_cell_config::Location::Inline(CellConfig {
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
                                location: Some(cell_application_config::Location::Inline(
                                    Manifest {
                                        name: "name".to_string(),
                                        ..Default::default()
                                    },
                                )),
                            },
                            CellApplicationConfig {
                                location: Some(cell_application_config::Location::Path(
                                    "some_path".to_string(),
                                )),
                            },
                        ],
                    })),
                },
                NodeCellConfig {
                    location: Some(node_cell_config::Location::Path("some_path".to_string())),
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
    fn node_config_absolute_relative_paths() -> anyhow::Result<()> {
        let mut config = LocalNodeConfig {
            path: "path".to_string(),
            ..Default::default()
        };
        config.make_absolute_paths("parent");

        assert_eq!(PathBuf::from(&config.path), PathBuf::from("parent/path"));

        config.make_relative_paths("parent");

        assert_eq!(PathBuf::from(&config.path), PathBuf::from("path"));

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

        let dir = tempfile::tempdir()?;
        let file = dir.path().join("file");

        config_init.to_yaml_file(&file)?;

        let mut config_read = LocalNodeConfig::from_yaml_file(&file)?;
        config_read.path = "".to_string();

        assert_eq!(config_init, config_read);

        Ok(())
    }

    #[test]
    fn node_config_inlined() -> anyhow::Result<()> {
        let config_path = root_test_fixtures_path("examples/node.yaml");
        let config = LocalNodeConfig::from_yaml_file(config_path)?;

        let inlined_config = config.inlined().unwrap();

        fn validate_node_cell_config(node_cell_config: &NodeCellConfig) {
            match node_cell_config.location.as_ref() {
                Some(node_cell_config::Location::Inline(cell_config)) => {
                    validate_cell(cell_config);
                }
                other => panic!("Expected cell to be an instance location, got: {:?}", other),
            }
        }

        fn validate_cell(cell_config: &CellConfig) {
            for cell_app_config in &cell_config.apps {
                match cell_app_config.location.as_ref() {
                    Some(cell_application_config::Location::Inline(app_manifest)) => {
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

        for cell in &inlined_config.cells {
            validate_node_cell_config(cell);
        }

        // should be able to load cell inlined
        assert!(Cell::new_from_local_node_config(inlined_config).is_ok());

        Ok(())
    }

    #[test]
    fn node_config_add_cell() -> anyhow::Result<()> {
        {
            // cell by path
            let mut config = LocalNodeConfig::default();

            config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Path("some_path".to_string())),
            });
            assert_eq!(1, config.cells.len());

            config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Path("some_path".to_string())),
            });
            assert_eq!(1, config.cells.len());

            config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Path("other_path".to_string())),
            });
            assert_eq!(2, config.cells.len());
        }

        {
            // cell inlined
            let mut config = LocalNodeConfig::default();

            config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Inline(CellConfig {
                    id: "id1".to_string(),
                    ..Default::default()
                })),
            });
            assert_eq!(1, config.cells.len());

            config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Inline(CellConfig {
                    id: "id1".to_string(),
                    ..Default::default()
                })),
            });
            assert_eq!(1, config.cells.len());

            config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Inline(CellConfig {
                    id: "id2".to_string(),
                    ..Default::default()
                })),
            });
            assert_eq!(2, config.cells.len());
        }

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
  - inline:
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
        - inline:
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

    #[test]
    fn cell_config_absolute_relative_paths() -> anyhow::Result<()> {
        let mut config = CellConfig {
            path: "path".to_string(),
            ..Default::default()
        };
        config.make_absolute_paths("parent");

        assert_eq!(PathBuf::from(&config.path), PathBuf::from("parent/path"));

        config.make_relative_paths("parent");

        assert_eq!(PathBuf::from(&config.path), PathBuf::from("path"));

        Ok(())
    }

    #[test]
    fn cell_config_yaml_file() -> anyhow::Result<()> {
        let config_init = CellConfig {
            ..Default::default()
        };

        let dir = tempfile::tempdir()?;
        let file = dir.path().join("file");

        config_init.to_yaml_file(&file)?;

        let mut config_read = CellConfig::from_yaml_file(&file)?;
        config_read.path = "".to_string();

        assert_eq!(config_init, config_read);

        Ok(())
    }

    #[test]
    fn cell_config_add_node() -> anyhow::Result<()> {
        let mut config = CellConfig {
            ..Default::default()
        };

        let node1 = CellNodeConfig {
            node: Some(NodeConfig {
                id: "id1".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        config.add_node(node1);
        assert_eq!(config.nodes.len(), 1);

        let node1_changed = CellNodeConfig {
            node: Some(NodeConfig {
                id: "id1".to_string(),
                name: "new name".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        config.add_node(node1_changed);
        assert_eq!(config.nodes.len(), 1);
        assert_eq!("new name", config.nodes[0].node.as_ref().unwrap().name);

        let node2 = CellNodeConfig {
            node: Some(NodeConfig {
                id: "id2".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        config.add_node(node2);
        assert_eq!(config.nodes.len(), 2);

        Ok(())
    }
}
