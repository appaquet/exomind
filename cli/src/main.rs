#![deny(bare_trait_objects)]

mod config;
mod logging;
mod options;
mod server;

#[macro_use]
extern crate log;

use config::NodeConfig;
use exocore_common::crypto::keys::Keypair;
use exocore_data::chain::ChainStore;
use exocore_data::{DirectoryChainStore, DirectoryChainStoreConfig};
use log::LevelFilter;
use std::str::FromStr;
use structopt::StructOpt;

fn main() -> Result<(), failure::Error> {
    let opt: options::Options = options::Options::from_args();
    logging::setup(Some(LevelFilter::from_str(&opt.logging_level)?));

    use options::{CellCommand, KeysCommand, ServerCommand, SubCommand};
    let result = match &opt.subcommand {
        SubCommand::server(server_opts) => match server_opts.command {
            ServerCommand::start => server::start(&opt, server_opts),
        },
        SubCommand::keys(keys_opts) => match keys_opts.command {
            KeysCommand::generate => keys_generate(&opt, keys_opts),
        },
        SubCommand::cell(cell_opts) => match cell_opts.command {
            CellCommand::create_genesis_block => cell_genesis_block(&opt, cell_opts),
        },
    };

    if let Err(err) = result {
        println!("Error: {}", err);
        println!("Backtrace: {}", err.backtrace());
    }

    Ok(())
}

fn keys_generate(
    _opt: &options::Options,
    keys_opts: &options::KeysOptions,
) -> Result<(), failure::Error> {
    let keypair = match keys_opts.algorithm {
        options::KeyAlgorithm::Ed25519 => {
            println!("Type: ED25519");
            Keypair::generate_ed25519()
        }
        options::KeyAlgorithm::Rsa => unimplemented!(),
    };

    println!("Keypair: {}", keypair.encode_base58_string());
    println!("Public key: {}", keypair.public().encode_base58_string());

    Ok(())
}

fn cell_genesis_block(
    _opt: &options::Options,
    cell_opts: &options::CellOptions,
) -> Result<(), failure::Error> {
    let config = NodeConfig::from_file(&cell_opts.config)?;
    let local_node = config.create_local_node()?;

    let cell_config = config
        .cells
        .iter()
        .find(|config| config.public_key == cell_opts.public_key)
        .expect("Couldn't find cell with given public key");

    let (full_cell, _cell) = cell_config.create_cell(&local_node)?;
    let full_cell = full_cell.expect("Cannot create genesis block on a non-full cell");

    let mut chain_dir = cell_config.data_directory.clone();
    chain_dir.push("chain");
    std::fs::create_dir_all(&chain_dir)?;

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;
    if chain_store.get_last_block()?.is_some() {
        panic!("Chain is already initialized");
    }

    let genesis_block = exocore_data::block::BlockOwned::new_genesis(&full_cell)?;
    chain_store.write_block(&genesis_block)?;

    Ok(())
}
