use std::path::PathBuf;

use clap::Clap;
use exocore_core::{
    cell::{Node, NodeId},
    sec::{
        hash::{multihash_sha3_256_file, MultihashExt},
        keys::Keypair,
    },
};

use crate::Context;

#[derive(Clap)]
pub struct SecOptions {
    #[clap(subcommand)]
    pub command: SecCommand,
}

#[derive(Clap)]
pub enum SecCommand {
    /// Generate a keypair.
    GenerateKey,

    /// Multihash a file.
    MultihashFile(MultihashFileOpt),
}

#[derive(Clap)]
pub struct MultihashFileOpt {
    // File to multihash
    file: PathBuf,
}

pub fn handle_cmd(ctx: &Context, keys_opts: &SecOptions) {
    match &keys_opts.command {
        SecCommand::GenerateKey => cmd_generate(ctx, keys_opts),
        SecCommand::MultihashFile(opt) => cmd_multihash_file(ctx, keys_opts, opt),
    }
}

fn cmd_generate(_ctx: &Context, _keys_opts: &SecOptions) {
    let keypair = Keypair::generate_ed25519();
    println!("keypair: {}", keypair.encode_base58_string());
    println!("public_key: {}", keypair.public().encode_base58_string());
    println!(
        "name: {}",
        Node::new_from_public_key(keypair.public()).name()
    );

    let node_id = NodeId::from_public_key(&keypair.public());
    println!("id: {}", node_id);
}

fn cmd_multihash_file(_ctx: &Context, _keys_opts: &SecOptions, opt: &MultihashFileOpt) {
    let bs58_mh = multihash_sha3_256_file(&opt.file).expect("Couldn't multihash file");
    println!("Multihash: {}", bs58_mh.encode_bs58());
}
