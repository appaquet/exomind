use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, short, default_value = "info")]
    /// Logging level (off, error, warn, info, debug, trace)
    pub logging_level: String,

    /// Path to gmail configuration.
    #[structopt(long, short = "c", default_value = "gmail.yaml")]
    pub config: PathBuf,

    /// Path to node configuration.
    #[structopt(long, short = "n", default_value = "node.yaml")]
    pub node_config: PathBuf,

    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    Daemon,
    ListAccounts,
    Login(LoginOptions),
    Logout(LogoutOptions),
}

#[derive(StructOpt)]
pub struct LoginOptions {
    pub email: String,
}

#[derive(StructOpt)]
pub struct LogoutOptions {
    pub email: String,
}
