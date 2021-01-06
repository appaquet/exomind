use clap::Clap;
use exocore_core::cell::{Node, NodeId};
use exocore_core::sec::keys::Keypair;

use crate::Context;

#[derive(Clap)]
pub struct KeysOptions {
    #[clap(subcommand)]
    pub command: KeysCommand,
}

#[derive(Clap)]
pub enum KeysCommand {
    /// Generate a keypair.
    Generate,
}

pub fn handle_cmd(ctx: &Context, keys_opts: &KeysOptions) -> anyhow::Result<()> {
    match keys_opts.command {
        KeysCommand::Generate => cmd_generate(ctx, keys_opts),
    }
}

fn cmd_generate(_ctx: &Context, _keys_opts: &KeysOptions) -> anyhow::Result<()> {
    let keypair = Keypair::generate_ed25519();
    println!("keypair: {}", keypair.encode_base58_string());
    println!("public_key: {}", keypair.public().encode_base58_string());
    println!(
        "name: {}",
        Node::new_from_public_key(keypair.public()).name()
    );

    let node_id = NodeId::from_public_key(&keypair.public());
    println!("id: {}", node_id);

    Ok(())
}
