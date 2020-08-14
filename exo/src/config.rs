use crate::options;
use exocore_core::protos::core::{cell_application_config, node_cell_config};

pub fn validate(
    _exo_opts: &options::ExoOptions,
    _conf_opts: &options::ConfigOptions,
    validate_opts: &options::ValidateOpts,
) -> anyhow::Result<()> {
    // parse config
    let config = exocore_core::cell::node_config_from_yaml_file(&validate_opts.config)?;

    // create instance to validate the config
    let (_cells, _node) = exocore_core::cell::Cell::new_from_local_node_config(config)?;

    Ok(())
}

pub fn standalone(
    _exo_opts: &options::ExoOptions,
    _conf_opts: &options::ConfigOptions,
    convert_opts: &options::StandaloneOpts,
) -> anyhow::Result<()> {
    let config = exocore_core::cell::node_config_from_yaml_file(&convert_opts.config)?;
    let mut config = exocore_core::cell::node_config_to_standalone(config)?;

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
        println!("{}", exocore_core::cell::node_config_to_json(&config)?);
    } else {
        println!("{}", exocore_core::cell::node_config_to_yaml(&config)?);
    }

    Ok(())
}
