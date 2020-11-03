#![deny(bare_trait_objects)]

mod cell;
mod daemon;
mod keys;
mod node;
mod options;
mod utils;

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use clap::Clap;
use log::LevelFilter;
use options::{CellCommand, ConfigCommand, KeysCommand, SubCommand};
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    let mut opts: options::ExoOptions = options::ExoOptions::parse();
    opts.validate()?;

    exocore_core::logging::setup(Some(LevelFilter::from_str(&opts.log)?));

    let result = match &opts.subcommand {
        SubCommand::Init(init_opts) => node::cmd_init(&opts, init_opts),
        SubCommand::Daemon => daemon::cmd_start(&opts),
        SubCommand::Keys(keys_opts) => match keys_opts.command {
            KeysCommand::Generate => keys::cmd_generate(&opts, keys_opts),
        },
        SubCommand::Cell(cell_opts) => match &cell_opts.command {
            CellCommand::Init(init_opts) => cell::cmd_init(&opts, cell_opts, init_opts),
            CellCommand::List => cell::cmd_list(&opts, cell_opts),
            CellCommand::CheckChain => cell::cmd_check_chain(&opts, cell_opts),
            CellCommand::Exportchain(export_opts) => {
                cell::cmd_export_chain(&opts, cell_opts, export_opts)
            }
            CellCommand::ImportChain(import_opts) => {
                cell::cmd_import_chain(&opts, cell_opts, import_opts)
            }
            CellCommand::GenerateAuthToken(gen_opts) => {
                cell::cmd_generate_auth_token(&opts, cell_opts, gen_opts)
            }
            CellCommand::CreateGenesisBlock => cell::cmd_create_genesis_block(&opts, cell_opts),
        },
        SubCommand::Config(config_opts) => match &config_opts.command {
            ConfigCommand::Validate => node::cmd_validate(&opts, config_opts),
            ConfigCommand::Standalone(standalone_opts) => {
                node::cmd_standalone(&opts, config_opts, standalone_opts)
            }
        },
    };

    if let Err(err) = result {
        println!("Error: {}", err);
    }

    Ok(())
}
