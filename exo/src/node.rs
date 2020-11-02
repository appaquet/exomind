use std::fs::File;

use crate::{options, utils::shell_prompt};
use exocore_core::{
    cell::LocalNodeConfigExt,
    cell::Node,
    protos::core::LocalNodeConfig,
    protos::core::NodeAddresses,
    protos::core::{cell_application_config, node_cell_config},
    sec::keys::Keypair,
};

pub fn init(
    exo_opts: &options::ExoOptions,
    init_opts: &options::NodeInitOptions,
) -> anyhow::Result<()> {
    let config_path = exo_opts.config_path()?;
    if config_path.exists() {
        return Err(anyhow!(
            "Cannot initialize node. A file already exists at '{:?}'",
            config_path
        ));
    }

    let home_path = exo_opts.home_path()?;
    if !home_path.exists() {
        std::fs::create_dir_all(home_path).expect("Couldn't create home directory");
    }

    let keypair = Keypair::generate_ed25519();
    let node = Node::new_from_public_key(keypair.public());

    let mut node_name = node.name().to_string();
    if init_opts.node_name.is_none() {
        let resp = shell_prompt("Node name", Some(&node_name))?;
        if let Some(resp) = resp {
            node_name = resp;
        }
    }

    let local_node_config = LocalNodeConfig {
        keypair: keypair.encode_base58_string(),
        public_key: keypair.public().encode_base58_string(),
        id: node.id().to_string(),
        name: node_name,

        addresses: Some(NodeAddresses {
            p2p: vec![
                "/ip4/0.0.0.0/tcp/3330".to_string(),
                "/ip4/0.0.0.0/tcp/3430/ws".to_string(),
            ],
            http: vec!["http://0.0.0.0:8030".to_string()],
        }),
        ..Default::default()
    };

    let config_file = File::create(&config_path)?;
    local_node_config.to_yaml_writer(config_file)?;

    println!("Node config written to {:?}", config_path);

    Ok(())
}

pub fn validate(
    exo_opts: &options::ExoOptions,
    _conf_opts: &options::ConfigOptions,
) -> anyhow::Result<()> {
    // parse config
    let config = exo_opts.read_configuration()?;

    // create instance to validate the config
    let (_cells, _node) = exocore_core::cell::Cell::new_from_local_node_config(config)?;

    Ok(())
}

pub fn standalone(
    exo_opts: &options::ExoOptions,
    _conf_opts: &options::ConfigOptions,
    convert_opts: &options::StandaloneOpts,
) -> anyhow::Result<()> {
    let config = exo_opts.read_configuration()?;
    let mut config = config.to_standalone()?;

    if convert_opts.exclude_app_schemas {
        for cell in &mut config.cells {
            if let Some(node_cell_config::Location::Instance(cell_config)) = &mut cell.location {
                for app in &mut cell_config.apps {
                    if let Some(cell_application_config::Location::Instance(app_manifest)) =
                        &mut app.location
                    {
                        app_manifest.schemas.clear();
                    }
                }
            }
        }
    }

    if convert_opts.format == "json" {
        println!("{}", config.to_json()?);
    } else {
        println!("{}", config.to_yaml()?);
    }

    Ok(())
}
