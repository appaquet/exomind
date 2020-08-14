#![deny(bare_trait_objects)]

mod cell;
mod config;
mod keys;
mod options;
mod server;

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use log::LevelFilter;
use options::{CellCommand, ConfigCommand, KeysCommand, ServerCommand, SubCommand};
use std::str::FromStr;
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    let opt: options::ExoOptions = options::ExoOptions::from_args();
    exocore_core::logging::setup(Some(LevelFilter::from_str(&opt.logging_level)?));

    let result = match &opt.subcommand {
        SubCommand::server(server_opts) => match server_opts.command {
            ServerCommand::start => server::start(&opt, server_opts),
        },
        SubCommand::keys(keys_opts) => match keys_opts.command {
            KeysCommand::generate => keys::generate(&opt, keys_opts),
        },
        SubCommand::cell(cell_opts) => match &cell_opts.command {
            CellCommand::create_genesis_block => cell::create_genesis_block(&opt, cell_opts),
            CellCommand::check_chain => cell::check_chain(&opt, cell_opts),
            CellCommand::export_chain(export_opts) => {
                cell::export_chain(&opt, cell_opts, export_opts)
            }
            CellCommand::import_chain(import_opts) => {
                cell::import_chain(&opt, cell_opts, import_opts)
            }
        },
        SubCommand::config(config_opts) => match &config_opts.command {
            ConfigCommand::validate(validate_opts) => {
                config::validate(&opt, config_opts, validate_opts)
            }
            ConfigCommand::standalone(standalone_opts) => {
                config::standalone(&opt, config_opts, standalone_opts)
            }
        },
    };

    if let Err(err) = result {
        println!("Error: {}", err);
    }

    Ok(())
}
