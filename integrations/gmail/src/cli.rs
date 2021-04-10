use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, short, default_value = "info")]
    /// Logging level (off, error, warn, info, debug, trace)
    pub log: String,

    /// Path to gmail configuration, relative to directory.
    #[structopt(long, short = "c", default_value = "gmail.yaml")]
    pub conf: PathBuf,

    /// Path to node directory.
    #[structopt(long, short = "d")]
    pub dir: Option<PathBuf>,

    /// Path to node configuration, relative to directory.
    #[structopt(long, short = "n", default_value = "node.yaml")]
    pub node: PathBuf,

    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

impl Options {
    pub fn conf_path(&self) -> PathBuf {
        if let Some(dir) = &self.dir {
            dir.join(&self.conf)
        } else {
            self.conf.clone()
        }
    }

    pub fn node_conf_path(&self) -> PathBuf {
        if let Some(dir) = &self.dir {
            dir.join(&self.node)
        } else {
            self.node.clone()
        }
    }
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
