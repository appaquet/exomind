use clap::Clap;
use exocore_core::{cell::LocalNodeConfigExt, protos::core::LocalNodeConfig};
use std::path::PathBuf;

use crate::utils::expand_tild;

#[derive(Clap)]
#[clap(name = "exocore-cli", about = "Exocore Command Line Interface")]
pub struct ExoOptions {
    /// Logging level (off, error, warn, info, debug, trace)
    #[clap(long, short, default_value = "info", env = "EXO_LOG")]
    pub logging_level: String,

    /// Home directory where config, cells and data will be stored. If none is specified, the parent directory
    /// of the configuration is used.
    #[clap(long, short = 'h', env = "EXO_HOME")]
    pub home: Option<PathBuf>,

    /// Configuration of the node to use. If a home directory is specified, the config is relative to it.
    #[clap(
        long,
        short = 'c',
        default_value = "~/.exocore/node.yaml",
        env = "EXO_CONFIG"
    )]
    pub config: PathBuf,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

impl ExoOptions {
    pub fn validate(&mut self) -> anyhow::Result<()> {
        if let Some(home) = &mut self.home {
            *home = expand_tild(&home)?;
        }

        self.config = expand_tild(&self.config)?;

        Ok(())
    }

    pub fn home_path(&self) -> anyhow::Result<PathBuf> {
        if let Some(home) = &self.home {
            Ok(home.clone())
        } else {
            match self.config.parent() {
                Some(parent) => Ok(parent.to_owned()),
                None => Err(anyhow!(
                    "Couldn't find home path since config file doesn't have parent: config={:?}",
                    self.config
                )),
            }
        }
    }

    pub fn config_path(&self) -> anyhow::Result<PathBuf> {
        if let Some(home) = &self.home {
            if !self.config.is_absolute() {
                Ok(home.join(&self.config))
            } else {
                Err(anyhow!(
                    "Expected config to be relative is a home path was specified, but got '{:?}'",
                    self.config
                ))
            }
        } else {
            Ok(self.config.clone())
        }
    }

    pub fn read_configuration(&self) -> anyhow::Result<LocalNodeConfig> {
        let config = LocalNodeConfig::from_yaml_file(self.config_path()?)
            .map_err(|err| anyhow!("Couldn't read configuration: {}", err))?;
        Ok(config)
    }
}

#[derive(Clap)]
pub enum SubCommand {
    Init(NodeInitOptions),
    Daemon,
    Keys(KeysOptions),
    Cell(CellOptions),
    Config(ConfigOptions),
}

/// Node intialization related options
#[derive(Clap)]
pub struct NodeInitOptions {
    #[clap(long)]
    /// Name of the node
    pub node_name: Option<String>,
}

/// Keys related options
#[derive(Clap)]
pub struct KeysOptions {
    #[clap(subcommand)]
    pub command: KeysCommand,
}

#[derive(Clap)]
pub enum KeysCommand {
    Generate,
}

/// Cell related options
#[derive(Clap)]
pub struct CellOptions {
    /// Public key of the cell we want to make an action on. If not specified
    /// and the node config only contains 1 cell, this cell will be taken.
    #[clap(long, short)]
    pub public_key: Option<String>,

    /// Name of the cell we want to make an action on. If not specified
    /// and the node config only contains 1 cell, this cell will be taken.
    #[clap(long, short)]
    pub name: Option<String>,

    #[clap(subcommand)]
    pub command: CellCommand,
}

#[derive(Clap)]
pub enum CellCommand {
    Init(CellInitOptions),
    CreateGenesisBlock,
    CheckChain,
    Exportchain(ChainExportOptions),
    ImportChain(ChainImportOptions),
    GenerateAuthToken(GenerateAuthTokenOptions),
}

/// Cell intialization related options
#[derive(Clap)]
pub struct CellInitOptions {
    /// Name of the cell
    pub cell_name: Option<String>,

    /// The node will not host the chain locally. The chain will need to be
    /// initialized on another node manually using "create_genesis_block".
    #[clap(long)]
    pub no_chain: bool,

    /// The node will not expose an entity store server.
    #[clap(long)]
    pub no_store: bool,
}

#[derive(Clap)]
pub struct ChainExportOptions {
    // File in which chain will be exported
    pub file: PathBuf,
}

#[derive(Clap)]
pub struct ChainImportOptions {
    // Number of operations per blocks
    #[clap(long, default_value = "30")]
    pub operations_per_block: usize,

    // Files from which chain will be imported
    pub files: Vec<PathBuf>,
}

#[derive(Clap)]
pub struct GenerateAuthTokenOptions {
    // Token expiration duration in days
    #[clap(long, default_value = "30")]
    pub expiration_days: u16,
}

/// Configs related options
#[derive(Clap)]
pub struct ConfigOptions {
    #[clap(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Clap)]
pub enum ConfigCommand {
    /// Validate the node's configuration
    Validate,

    /// Convert the node's configuration to a standalone configuration
    Standalone(StandaloneOpts),
}

#[derive(Clap)]
pub struct StandaloneOpts {
    #[clap(default_value = "json")]
    pub format: String,

    #[clap(long)]
    pub exclude_app_schemas: bool,
}
