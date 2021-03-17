#![allow(non_camel_case_types)]

use exocore::core::utils::path::child_to_abs_path;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "exomind-server", about = "Exomind server")]
pub struct Options {
    #[structopt(long, short, default_value = "info")]
    /// Logging level (off, error, warn, info, debug, trace)
    pub logging_level: String,

    #[structopt(long, short = "c", default_value = "exomind.yaml")]
    pub config: PathBuf,

    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    Start,
    Gmail(exomind_gmail::cli::Options),
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub node_config: PathBuf,

    pub gmail: Option<exomind_gmail::config::Config>,
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Config> {
        let file = std::fs::File::open(path)?;
        let mut config: Config = serde_yaml::from_reader(file)?;

        let config_dir = path
            .parent()
            .ok_or_else(|| anyhow!("Couldn't get config parent directory"))?;

        config.node_config = child_to_abs_path(config_dir, &config.node_config);

        if let Some(gmail_config) = config.gmail.as_mut() {
            gmail_config.make_abs_path(config_dir);
        }

        Ok(config)
    }
}
