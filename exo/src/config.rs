use crate::{utils::edit_file, Options};
use clap::Clap;
use exocore_core::{
    cell::LocalNodeConfigExt,
    protos::core::{cell_application_config, node_cell_config, LocalNodeConfig},
};
use std::time::Duration;

#[derive(Clap)]
pub struct ConfigOptions {
    #[clap(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Clap)]
pub enum ConfigCommand {
    /// Edit the node's configuration
    Edit,

    /// Validate the node's configuration
    Validate,

    /// Convert the node's configuration to a standalone configuration
    Standalone(StandaloneOpts),
}

#[derive(Clap)]
pub struct StandaloneOpts {
    #[clap(default_value = "json")]
    pub format: String,

    #[clap(long)]
    pub exclude_app_schemas: bool,
}

pub fn handle_cmd(exo_opts: &Options, config_opts: &ConfigOptions) -> anyhow::Result<()> {
    match &config_opts.command {
        ConfigCommand::Edit => cmd_edit(&exo_opts, config_opts),
        ConfigCommand::Validate => cmd_validate(&exo_opts, config_opts),
        ConfigCommand::Standalone(standalone_opts) => {
            cmd_standalone(&exo_opts, config_opts, standalone_opts)
        }
    }
}

fn cmd_edit(exo_opts: &Options, _conf_opts: &ConfigOptions) -> anyhow::Result<()> {
    let config_path = exo_opts.conf_path();

    edit_file(config_path, |temp_path| -> bool {
        if let Err(err) = LocalNodeConfig::from_yaml_file(temp_path) {
            println!("Error parsing config: {:?}", err);
            std::thread::sleep(Duration::from_secs(2));
            false
        } else {
            true
        }
    });

    Ok(())
}

fn cmd_validate(exo_opts: &Options, _conf_opts: &ConfigOptions) -> anyhow::Result<()> {
    // parse config
    let config = exo_opts.read_configuration();

    // create instance to validate the config
    let (_cells, _node) = exocore_core::cell::Cell::new_from_local_node_config(config)?;

    Ok(())
}

fn cmd_standalone(
    exo_opts: &Options,
    _conf_opts: &ConfigOptions,
    convert_opts: &StandaloneOpts,
) -> anyhow::Result<()> {
    let config = exo_opts.read_configuration();
    let mut config = config
        .to_standalone()
        .expect("Couldn't convert config to standalone");

    if convert_opts.exclude_app_schemas {
        for cell in &mut config.cells {
            if let Some(node_cell_config::Location::Instance(cell_config)) = &mut cell.location {
                for app in &mut cell_config.apps {
                    if let Some(cell_application_config::Location::Instance(app_manifest)) =
                        &mut app.location
                    {
                        app_manifest.schemas.clear();
                    }
                }
            }
        }
    }

    if convert_opts.format == "json" {
        println!("{}", config.to_json()?);
    } else {
        println!("{}", config.to_yaml()?);
    }

    Ok(())
}
