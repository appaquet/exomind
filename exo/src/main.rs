#![deny(bare_trait_objects)]

mod cell;
mod config;
mod daemon;
mod disco;
mod keys;
mod node;
mod utils;

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use clap::Clap;
use exocore_core::{cell::LocalNodeConfigExt, protos::core::LocalNodeConfig};
use log::LevelFilter;
use std::{path::PathBuf, str::FromStr};
use utils::expand_tild;

#[derive(Clap)]
#[clap(name = "exocore-cli", about = "Exocore Command Line Interface")]
pub struct Options {
    /// Logging level (off, error, warn, info, debug, trace)
    #[clap(long, short, default_value = "info", env = "EXO_LOG")]
    pub log: String,

    /// Directory where config, cells and data will be stored.
    #[clap(long, short = 'd', default_value = "~/.exocore", env = "EXO_DIR")]
    pub dir: PathBuf,

    /// Configuration of the node to use, relative to the directory.
    #[clap(long, short = 'c', default_value = "node.yaml", env = "EXO_CONF")]
    pub conf: PathBuf,

    #[clap(subcommand)]
    subcommand: Commands,
}

impl Options {
    pub fn validate(&mut self) -> anyhow::Result<()> {
        self.dir = expand_tild(&self.dir)?;
        self.conf = expand_tild(&self.conf)?;

        Ok(())
    }

    pub fn dir_path(&self) -> PathBuf {
        self.dir.clone()
    }

    pub fn conf_path(&self) -> PathBuf {
        self.dir.join(&self.conf)
    }

    pub fn read_configuration(&self) -> LocalNodeConfig {
        let config_path = self.conf_path();
        LocalNodeConfig::from_yaml_file(&config_path).expect("Couldn't read node config")
    }
}

#[derive(Clap)]
pub enum Commands {
    /// Nodes releated commands.
    Node(node::NodeOptions),

    /// Cells related commands.
    Cell(cell::CellOptions),

    /// Keys releated commands.
    Keys(keys::KeysOptions),

    /// Node configuration related commands.
    Config(config::ConfigOptions),

    /// Start the node daemon, with all its cells and roles.
    Daemon,

    /// Discovery service related commands.
    Discovery(disco::DiscoveryCommand),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut opts: Options = Options::parse();
    opts.validate()?;

    exocore_core::logging::setup(Some(LevelFilter::from_str(&opts.log)?));

    let result = match &opts.subcommand {
        Commands::Node(node_opts) => node::handle_cmd(&opts, node_opts),
        Commands::Daemon => daemon::cmd_daemon(&opts).await,
        Commands::Keys(keys_opts) => keys::handle_cmd(&opts, keys_opts),
        Commands::Cell(cell_opts) => cell::handle_cmd(&opts, cell_opts),
        Commands::Config(config_opts) => config::handle_cmd(&opts, config_opts),
        Commands::Discovery(disco_opts) => disco::cmd_daemon(&opts, disco_opts).await,
    };

    if let Err(err) = result {
        println!("Error: {}", err);
    }

    Ok(())
}
