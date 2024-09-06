use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    /// Logging level (off, error, warn, info, debug, trace)
    #[structopt(long, short, default_value = "info")]
    pub log: String,

    /// Path to node directory.
    #[structopt(long, short = "d")]
    pub dir: Option<PathBuf>,

    #[structopt(subcommand)]
    pub command: Command,
}

impl Options {
    pub fn node_directory(&self) -> PathBuf {
        self.dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().expect("Couldn't get current directory"))
    }
}

#[derive(StructOpt)]
pub enum Command {
    /// Gmail integration related commands.
    Gmail(exomind_gmail::cli::Options),

    /// Print version.
    Version,
}
