use std::path::PathBuf;

use exocore_core::{
    cell::{Node, NodeId},
    sec::{
        hash::{multihash_sha3_256_file, MultihashExt},
        keys::{Keypair, PublicKey},
    },
};

use crate::Context;

#[derive(clap::Parser)]
pub struct SecOptions {
    #[clap(subcommand)]
    pub command: SecCommand,
}

#[derive(clap::Parser)]
pub enum SecCommand {
    /// Generates a keypair.
    GenerateKey,

    /// Parse a public key and print its ID.
    ParsePublicKey(ParsePublicKeyOpt),

    /// Multihash a file.
    MultihashFile(MultihashFileOpt),
}

#[derive(clap::Parser)]
pub struct MultihashFileOpt {
    // File to multihash
    file: PathBuf,
}

#[derive(clap::Parser)]
pub struct ParsePublicKeyOpt {
    public_key: String,
}

pub fn handle_cmd(ctx: &Context, keys_opts: &SecOptions) {
    match &keys_opts.command {
        SecCommand::GenerateKey => cmd_generate(ctx, keys_opts),
        SecCommand::MultihashFile(opt) => cmd_multihash_file(ctx, keys_opts, opt),
        SecCommand::ParsePublicKey(opt) => cmd_parse_public_key(ctx, keys_opts, opt),
    }
}

fn cmd_generate(_ctx: &Context, _keys_opts: &SecOptions) {
    let keypair = Keypair::generate_ed25519();
    println!("keypair: {}", keypair.encode_base58_string());
    println!("public_key: {}", keypair.public().encode_base58_string());
    println!("name: {}", Node::from_public_key(keypair.public()).name());

    let id = NodeId::from_public_key(&keypair.public());
    println!("id: {}", id);
}

fn cmd_multihash_file(_ctx: &Context, _keys_opts: &SecOptions, opt: &MultihashFileOpt) {
    let bs58_mh = multihash_sha3_256_file(&opt.file).expect("Couldn't multihash file");
    println!("Multihash: {}", bs58_mh.encode_bs58());
}

fn cmd_parse_public_key(_ctx: &Context, _keys_opts: &SecOptions, opt: &ParsePublicKeyOpt) {
    let pk = PublicKey::decode_base58_string(&opt.public_key).expect("Couldn't parse public key");
    let id = NodeId::from_public_key(&pk);
    println!("id: {}", id);
}
