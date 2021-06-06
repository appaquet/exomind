use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, short, default_value = "info")]
    /// Logging level (off, error, warn, info, debug, trace)
    pub log: String,

    /// Path to node directory.
    #[structopt(long, short = "d")]
    pub dir: Option<PathBuf>,

    /// Path to node configuration, relative to directory.
    #[structopt(long, short = "n", default_value = "node.yaml")]
    pub node: PathBuf,

    #[structopt(subcommand)]
    pub command: Command,
}

impl Options {
    pub fn directory(&self) -> PathBuf {
        self.dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().expect("Couldn't get current directory"))
    }

    pub fn node_conf_path(&self) -> PathBuf {
        self.directory().join(&self.node)
    }
}

#[derive(StructOpt)]
pub enum Command {
    /// Gmail integration related commands.
    Gmail(exomind_gmail::cli::Options),

    /// Print version.
    Version,
}
