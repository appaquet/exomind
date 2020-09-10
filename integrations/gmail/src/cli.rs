#![allow(non_camel_case_types)]

use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "exomind-gmail", about = "Exomind Gmail integration")]
pub struct Options {
    #[structopt(long, short, default_value = "info")]
    /// Logging level (off, error, warn, info, debug, trace)
    pub logging_level: String,

    #[structopt(long, short = "c", default_value = "gmail.yaml")]
    pub config: PathBuf,

    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    start(StartOptions),
    list_accounts,
    login(LoginOptions),
    logout(LogoutOptions),
}

#[derive(StructOpt)]
pub struct StartOptions {
    #[structopt(long)]
    pub save_fixtures: bool,
}

#[derive(StructOpt)]
pub struct LoginOptions {
    pub email: String,
}

#[derive(StructOpt)]
pub struct LogoutOptions {
    pub email: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub node_config: String,

    pub client_secret: String,

    pub tokens_directory: String,
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Config> {
        let file = std::fs::File::open(path)?;
        let mut config: Config = serde_yaml::from_reader(file)?;

        let config_dir = path
            .parent()
            .ok_or_else(|| anyhow!("Couldn't get config parent directory"))?;

        config.node_config = Self::to_abs_path(config_dir, &config.node_config);
        config.client_secret = Self::to_abs_path(config_dir, &config.client_secret);
        config.tokens_directory = Self::to_abs_path(config_dir, &config.tokens_directory);

        Ok(config)
    }

    fn to_abs_path(parent_path: &Path, child_path: &str) -> String {
        let child_path_buf = PathBuf::from(child_path);
        if child_path_buf.is_absolute() {
            return child_path_buf.to_string_lossy().to_string();
        }

        parent_path
            .join(child_path_buf)
            .to_string_lossy()
            .to_string()
    }
}
