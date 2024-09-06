use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    /// Path to gmail configuration, relative to directory.
    #[structopt(long, short = "c", default_value = "gmail.yaml")]
    pub conf: PathBuf,

    #[structopt(subcommand)]
    pub subcommand: Command,
}

#[derive(StructOpt)]
pub enum Command {
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
