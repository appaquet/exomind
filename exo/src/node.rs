use crate::{utils::shell_prompt, Options};
use clap::Clap;
use exocore_core::{
    cell::LocalNodeConfigExt, cell::Node, protos::core::LocalNodeConfig,
    protos::core::NodeAddresses, sec::keys::Keypair,
};
use std::fs::File;

#[derive(Clap)]
pub struct NodeOptions {
    #[clap(subcommand)]
    pub command: NodeCommand,
}

#[derive(Clap)]
pub enum NodeCommand {
    /// Initialize the node.
    Init(InitOptions),
}

#[derive(Clap)]
pub struct InitOptions {
    /// Name of the node.
    #[clap(long)]
    pub name: Option<String>,
}

pub fn handle_cmd(exo_opts: &Options, node_opts: &NodeOptions) -> anyhow::Result<()> {
    match &node_opts.command {
        NodeCommand::Init(init_opts) => cmd_init(exo_opts, init_opts),
    }
}

fn cmd_init(exo_opts: &Options, init_opts: &InitOptions) -> anyhow::Result<()> {
    let config_path = exo_opts.conf_path();
    if config_path.exists() {
        panic!(
            "Cannot initialize node. A file already exists at '{:?}'",
            config_path
        );
    }

    let home_path = exo_opts.dir_path();
    if !home_path.exists() {
        std::fs::create_dir_all(home_path).expect("Couldn't create home directory");
    }

    let keypair = Keypair::generate_ed25519();
    let node = Node::new_from_public_key(keypair.public());

    let mut node_name = node.name().to_string();
    if init_opts.name.is_none() {
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
