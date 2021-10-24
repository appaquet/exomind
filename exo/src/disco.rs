use exocore_discovery::{Client, Pin, Server, ServerConfig};

use crate::Context;

#[derive(clap::Parser)]
pub struct DiscoveryOptions {
    #[clap(subcommand)]
    pub command: DiscoveryCommand,
}

#[derive(clap::Parser)]
pub enum DiscoveryCommand {
    // Starts a discovery service daemon.
    Daemon(DaemonOptions),
}

#[derive(clap::Parser)]
pub struct DaemonOptions {
    #[clap(long, default_value = "8085")]
    port: u16,
}

pub async fn cmd_daemon(_ctx: &Context, cmd: &DiscoveryOptions) -> anyhow::Result<()> {
    match &cmd.command {
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
        .validate_with(|input: &String| -> Result<(), &str> {
            input
                .parse::<Pin>()
                .map_err(|_| "This is not a valid pin")?;

            Ok(())
        })
        .interact_text()
        .expect("Couldn't get pin input from terminal");

    node_discovery_pin
        .parse()
        .expect("Received an invalid pin from discovery service")
}
