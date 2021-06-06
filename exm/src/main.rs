use std::str::FromStr;

use log::LevelFilter;
use structopt::StructOpt;

mod cli;

#[tokio::main]
async fn main() {
    let opt: cli::Options = cli::Options::from_args();
    exocore::core::logging::setup::<String>(Some(LevelFilter::from_str(&opt.log).unwrap()), None);

    match &opt.command {
        cli::Command::Gmail(gmail_opt) => {
            let client = exocore::client::Client::from_node_config_file(opt.node_conf_path())
                .await
                .expect("Couldn't create client from config");

            exomind_gmail::handle(client, opt.directory(), gmail_opt).await
        }
        cli::Command::Version => {
            println!("exomind version {}", env!("CARGO_PKG_VERSION"));
            println!("exocore version {}", exocore::core::version());
        }
    };
}
