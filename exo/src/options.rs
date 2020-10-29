use clap::Clap;
use exocore_core::protos::core::LocalNodeConfig;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clap)]
#[clap(name = "exocore-cli", about = "Exocore Command Line Interface")]
pub struct ExoOptions {
    /// Logging level (off, error, warn, info, debug, trace)
    #[clap(long, short, default_value = "info", env = "EXO_INFO")]
    pub logging_level: String,

    /// Home directory where config, cells and data will be store.
    #[clap(long, short = 'h', default_value = "~/.exocore", env = "EXO_HOME")]
    pub home: PathBuf,

    /// Configuration of the node to use, relative to the home directory.
    #[clap(long, short = 'c', default_value = "node.yaml", env = "EXO_CONFIG")]
    pub config: PathBuf,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

impl ExoOptions {
    pub fn read_configuration(&self) -> anyhow::Result<LocalNodeConfig> {
        let config_path = if self.config.is_absolute() {
            self.config.clone()
        } else {
            self.home.join(&self.config)
        };

        let config = exocore_core::cell::node_config_from_yaml_file(config_path)?;

        Ok(config)
    }
}

#[derive(Clap)]
pub enum SubCommand {
    Daemon,
    Keys(KeysOptions),
    Cell(CellOptions),
    Config(ConfigOptions),
}

/// Keys related options
#[derive(Clap)]
pub struct KeysOptions {
    #[clap(long, default_value = "ed25519")]
    /// Algorithm of the keypair to generate (ed25519, rsa)
    pub algorithm: KeyAlgorithm,
    #[clap(subcommand)]
    pub command: KeysCommand,
}

#[derive(Clap)]
pub enum KeysCommand {
    Generate,
}

#[derive(Clap)]
pub enum KeyAlgorithm {
    ED25519,
    RSA,
}

impl FromStr for KeyAlgorithm {
    type Err = anyhow::Error;

    fn from_str(k: &str) -> Result<Self, Self::Err> {
        match k {
            "ed25519" => Ok(KeyAlgorithm::ED25519),
            _ => Err(anyhow!("Unsupported key type")),
        }
    }
}

/// Cell related options
#[derive(Clap)]
pub struct CellOptions {
    #[clap(long, short)]
    /// Public key of the cell we want to make an action on. If not specified
    /// and the node config only contains 1 cell, this cell will be taken.
    pub public_key: Option<String>,

    #[clap(subcommand)]
    pub command: CellCommand,
}

#[derive(Clap)]
pub enum CellCommand {
    CreateGenesisBlock,
    CheckChain,
    Exportchain(ChainExportOptions),
    ImportChain(ChainImportOptions),
    GenerateAuthToken(GenerateAuthTokenOptions),
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
pub struct ValidateOpts {
    /// Path to configuration
    pub config: PathBuf,
}

#[derive(Clap)]
pub struct StandaloneOpts {
    #[clap(default_value = "json")]
    pub format: String,

    #[clap(long)]
    pub exclude_app_schemas: bool,
}
