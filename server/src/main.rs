use std::{str::FromStr, time::Duration};

use exomind::ExomindClient;

use log::LevelFilter;
use structopt::StructOpt;
use tokio::time::sleep;

mod cli;
mod exomind;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_derive;

#[tokio::main]
async fn main() {
    let opt: cli::Options = cli::Options::from_args();
    exocore::core::logging::setup::<String>(
        Some(LevelFilter::from_str(&opt.logging_level).unwrap()),
        None,
    );

    let config = cli::Config::from_file(&opt.config)
        .unwrap_or_else(|err| panic!("Couldn't parse config {:?}: {}", &opt.config, err));

    match opt.subcommand {
        cli::SubCommand::Start => {
            start(config).await.unwrap();
        }
        cli::SubCommand::Gmail(gmail_opt) => {
            let gmail_config = config
                .gmail
                .clone()
                .expect("Config didn't contain a gmail section");

            let exm = ExomindClient::new(&config)
                .await
                .expect("Couldn't create exomind client");
            exomind_gmail::cli::exec(exm.store.clone(), gmail_config, gmail_opt)
                .await
                .unwrap();
        }
    }
}

async fn start(config: cli::Config) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config)
        .await
        .expect("Couldn't create exomind client");

    // give some time for client to connect
    sleep(Duration::from_secs(1)).await;

    let gmail_store_handle = exm.store.clone();
    if let Some(gmail_config) = config.gmail {
        exomind_gmail::server::run(gmail_config, gmail_store_handle).await
    } else {
        Err(anyhow!("No gmail configuration found"))
    }
}
