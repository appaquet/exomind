use crate::options;
use exocore_core::{
    cell::LocalNodeConfigExt,
    protos::core::{cell_application_config, node_cell_config},
};

pub fn validate(
    exo_opts: &options::ExoOptions,
    _conf_opts: &options::ConfigOptions,
) -> anyhow::Result<()> {
    // parse config
    let config = exo_opts.read_configuration()?;

    // create instance to validate the config
    let (_cells, _node) = exocore_core::cell::Cell::new_from_local_node_config(config)?;

    Ok(())
}

pub fn standalone(
    exo_opts: &options::ExoOptions,
    _conf_opts: &options::ConfigOptions,
    convert_opts: &options::StandaloneOpts,
) -> anyhow::Result<()> {
    let config = exo_opts.read_configuration()?;
    let mut config = config.to_standalone()?;

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
