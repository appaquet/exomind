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
            let client = exocore::client::Client::from_node_directory(opt.node_directory())
                .await
                .expect("Couldn't create client from config");

            exomind_gmail::handle(client, opt.node_directory(), gmail_opt).await
        }
        cli::Command::Version => {
            println!("exomind version={}", env!("CARGO_PKG_VERSION"));
            println!("exocore {}", exocore::core::build::build_info_str());
        }
    };
}
