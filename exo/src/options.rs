#![deny(bare_trait_objects)]
#![allow(non_camel_case_types)]

use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

/// CLI options
#[derive(StructOpt)]
#[structopt(name = "exocore-cli", about = "Exocore Command Line Interface")]
pub struct Options {
    #[structopt(long, short, default_value = "info")]
    /// Logging level (off, error, warn, info, debug, trace)
    pub logging_level: String,
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    server(ServerOptions),
    keys(KeysOptions),
    cell(CellOptions),
    config(ConfigOptions),
}

#[derive(StructOpt)]
pub struct ServerOptions {
    #[structopt(long, short = "c", default_value = "config.yaml")]
    pub config: PathBuf,
    #[structopt(subcommand)]
    pub command: ServerCommand,
}

#[derive(StructOpt)]
pub enum ServerCommand {
    start,
}

/// Keys related options
#[derive(StructOpt)]
pub struct KeysOptions {
    #[structopt(long, default_value = "ed25519")]
    /// Algorithm of the keypair to generate (ed25519, rsa)
    pub algorithm: KeyAlgorithm,
    #[structopt(subcommand)]
    pub command: KeysCommand,
}

#[derive(StructOpt)]
pub enum KeysCommand {
    generate,
}

#[derive(StructOpt)]
pub enum KeyAlgorithm {
    Ed25519,
    Rsa,
}

impl FromStr for KeyAlgorithm {
    type Err = anyhow::Error;

    fn from_str(k: &str) -> Result<Self, Self::Err> {
        match k {
            "ed25519" => Ok(KeyAlgorithm::Ed25519),
            _ => Err(anyhow!("Unsupported key type")),
        }
    }
}

/// Cell related options
#[derive(StructOpt)]
pub struct CellOptions {
    #[structopt(long, short = "c", default_value = "config.yaml")]
    pub config: PathBuf,
    #[structopt(long, short)]
    /// Public key of the cell we want to make an action on
    pub public_key: String,
    #[structopt(subcommand)]
    pub command: CellCommand,
}

#[derive(StructOpt)]
pub enum CellCommand {
    create_genesis_block,
    check_chain,
}

/// Configs related options
#[derive(StructOpt)]
pub struct ConfigOptions {
    #[structopt(subcommand)]
    pub command: ConfigCommand,
}

#[derive(StructOpt)]
pub enum ConfigCommand {
    /// Validate a configuration
    validate(ValidateOpts),

    /// Convert a configuration to a standalone configuration
    standalone(StandaloneOpts),
}

#[derive(StructOpt)]
pub struct ValidateOpts {
    /// Path to configuration
    pub config: PathBuf,
}

#[derive(StructOpt)]
pub struct StandaloneOpts {
    /// Path to configuration
    pub config: PathBuf,

    #[structopt(default_value = "json")]
    pub format: String,

    #[structopt(long)]
    pub exclude_app_schemas: bool,
}
