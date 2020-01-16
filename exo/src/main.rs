#![deny(bare_trait_objects)]

mod cell;
mod config;
mod logging;
mod options;
mod server;

#[macro_use]
extern crate log;

use exocore_common::crypto::keys::Keypair;
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
            CellCommand::create_genesis_block => cell::create_genesis_block(&opt, cell_opts),
            CellCommand::check_chain => cell::check_chain(&opt, cell_opts),
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
