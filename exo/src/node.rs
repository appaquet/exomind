use std::fs::File;

use clap::Clap;
use exocore_core::cell::LocalNode;
use exocore_core::cell::LocalNodeConfigExt;
use exocore_protos::core::{LocalNodeConfig, NodeAddresses};

use crate::{term::*, Context};

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

pub fn handle_cmd(ctx: &Context, node_opts: &NodeOptions) -> anyhow::Result<()> {
    match &node_opts.command {
        NodeCommand::Init(init_opts) => cmd_init(ctx, init_opts),
    }
}

fn cmd_init(ctx: &Context, init_opts: &InitOptions) -> anyhow::Result<()> {
    let config_path = ctx.options.conf_path();
    if config_path.exists() {
        panic!(
            "Cannot initialize node. A file already exists at '{:?}'",
            config_path
        );
    }

    print_step("Initializing node directory");
    let home_path = ctx.options.dir_path();
    if !home_path.exists() {
        print_action(format!("Creating directory {}", style_value(&home_path)));
        std::fs::create_dir_all(home_path).expect("Couldn't create home directory");
    }

    let local_node = LocalNode::generate();
    let node = local_node.node();

    let mut node_name = node.name().to_string();
    if init_opts.name.is_none() {
        print_spacer();
        node_name = dialoguer::Input::with_theme(ctx.dialog_theme.as_ref())
            .with_prompt("Enter the name of the node")
            .default(node.name().to_string())
            .interact_text()?;
    }

    let local_node_config = LocalNodeConfig {
        name: node_name,
        addresses: Some(NodeAddresses {
            p2p: vec![
                "/ip4/0.0.0.0/tcp/3330".to_string(),
                "/ip4/0.0.0.0/tcp/3430/ws".to_string(),
            ],
            http: vec!["http://0.0.0.0:8030".to_string()],
        }),

        ..local_node.config().clone()
    };

    print_action(format!(
        "Writing configuration to {}",
        style_value(&config_path)
    ));
    let config_file = File::create(&config_path)?;
    local_node_config.to_yaml_writer(config_file)?;

    print_success(format!(
        "Node {} with public key {} created",
        style_value(&local_node_config.name),
        style_value(&local_node_config.public_key),
    ));
    Ok(())
}
