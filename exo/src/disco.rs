use std::convert::TryInto;

use crate::Context;
use clap::Clap;
use exocore_discovery::{Client, Pin, Server, ServerConfig};

#[derive(Clap)]
pub enum DiscoveryCommand {
    // Start a discovery service daemon.
    Daemon(DaemonOptions),
}

#[derive(Clap)]
pub struct DaemonOptions {
    #[clap(long, default_value = "8085")]
    port: u16,
}

pub async fn cmd_daemon(_ctx: &Context, cmd: &DiscoveryCommand) -> anyhow::Result<()> {
    match cmd {
        DiscoveryCommand::Daemon(daemon_opts) => start_daemon(daemon_opts).await,
    }
}

async fn start_daemon(opts: &DaemonOptions) -> anyhow::Result<()> {
    let server_config = ServerConfig {
        port: opts.port,
        ..Default::default()
    };

    let server = Server::new(server_config);
    server.start().await?;

    Ok(())
}

pub fn get_discovery_client(ctx: &Context) -> Client {
    Client::new(&ctx.options.discovery_service).expect("Couldn't create discovery client")
}

pub fn prompt_discovery_pin(ctx: &Context, text: &str) -> Pin {
    let node_discovery_pin = dialoguer::Input::with_theme(ctx.dialog_theme.as_ref())
        .with_prompt(text)
        .validate_with(|input: &String| {
            if input.chars().all(|c| c.is_digit(10) || c.is_whitespace()) {
                Ok(())
            } else {
                Err("This is not a valid pin")
            }
        })
        .interact_text()
        .expect("Couldn't get pin input from terminal");

    let node_discovery_pin: u32 = node_discovery_pin
        .replace(|c: char| c.is_whitespace(), "")
        .parse()
        .expect("Couldn't parse discovery pin");

    node_discovery_pin
        .try_into()
        .expect("Received an invalid pin from discovery service")
}
