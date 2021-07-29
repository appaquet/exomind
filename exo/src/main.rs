mod app;
mod cell;
mod config;
mod daemon;
mod disco;
mod node;
mod sec;
mod term;
mod utils;

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use std::{path::PathBuf, str::FromStr};

use clap::Clap;
use exocore_core::cell::LocalNodeConfigExt;
use exocore_protos::core::LocalNodeConfig;
use log::LevelFilter;
use term::*;
use utils::expand_tild;

#[derive(Clap)]
#[clap(name = "exocore-cli", about = "Exocore Command Line Interface")]
pub struct Options {
    /// Logging level (off, error, warn, info, debug, trace)
    #[clap(long, short, default_value = "info", env = "EXO_LOG")]
    pub log: String,

    /// Directory where config, cells and data will be stored.
    #[clap(long, short = 'd', default_value = "~/.exocore", env = "EXO_DIR")]
    pub dir: PathBuf,

    /// Configuration of the node to use, relative to the directory.
    #[clap(long, short = 'c', default_value = "node.yaml", env = "EXO_CONF")]
    pub conf: PathBuf,

    /// URL of the discovery service to use for configuration exchange when
    /// joining a cell or adding a new node to a cell.
    #[clap(
        long,
        default_value = "https://disco.exocore.io",
        env = "EXO_DISCOVERY"
    )]
    pub discovery_service: String,

    #[clap(subcommand)]
    subcommand: Command,
}

impl Options {
    pub fn validate(&mut self) -> anyhow::Result<()> {
        self.dir = expand_tild(&self.dir)?;
        self.conf = expand_tild(&self.conf)?;

        Ok(())
    }

    pub fn dir_path(&self) -> PathBuf {
        self.dir.clone()
    }

    pub fn conf_path(&self) -> PathBuf {
        self.dir.join(&self.conf)
    }

    pub fn read_configuration(&self) -> LocalNodeConfig {
        let config_path = self.conf_path();

        print_info(format!(
            "Using node in directory {}",
            style_value(config_path.to_string_lossy()),
        ));

        LocalNodeConfig::from_yaml_file(&config_path).expect("Couldn't read node config")
    }
}

pub struct Context {
    options: Options,

    dialog_theme: Box<dyn dialoguer::theme::Theme>,
}

impl Context {
    fn get_discovery_client(&self) -> exocore_discovery::Client {
        disco::get_discovery_client(self)
    }
}

#[derive(Clap)]
pub enum Command {
    /// Nodes related commands.
    Node(node::NodeOptions),

    /// Cells related commands.
    Cell(cell::CellOptions),

    /// Applications related commands.
    App(app::AppOptions),

    /// Security related commands.
    Sec(sec::SecOptions),

    /// Node configuration related commands.
    Config(config::ConfigOptions),

    /// Start the node daemon, with all its cells and roles.
    Daemon,

    /// Discovery service related commands.
    Discovery(disco::DiscoveryCommand),

    /// Print version and build information.
    Version,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let mut options: Options = Options::parse();
    options.validate()?;

    exocore_core::logging::setup::<String>(Some(LevelFilter::from_str(&options.log)?), None);
    std::panic::set_hook(Box::new(|info| {
        error!("Panic occurred: {}", info);
    }));

    let ctx = Context {
        options,
        dialog_theme: Box::new(dialoguer::theme::ColorfulTheme::default()),
    };

    let result = match &ctx.options.subcommand {
        Command::Node(node_opts) => node::handle_cmd(&ctx, node_opts),
        Command::Cell(cell_opts) => cell::handle_cmd(&ctx, cell_opts).await,
        Command::App(app_opts) => {
            app::handle_cmd(&ctx, app_opts).await;
            Ok(())
        }
        Command::Sec(keys_opts) => {
            sec::handle_cmd(&ctx, keys_opts);
            Ok(())
        }
        Command::Config(config_opts) => config::handle_cmd(&ctx, config_opts),
        Command::Daemon => daemon::cmd_daemon(&ctx).await,
        Command::Discovery(disco_opts) => disco::cmd_daemon(&ctx, disco_opts).await,
        Command::Version => {
            println!("{}", exocore_core::build::build_info_str());
            Ok(())
        }
    };

    if let Err(err) = result {
        println!("Error: {}", err);
    }

    Ok(())
}
