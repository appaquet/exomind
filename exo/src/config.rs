use crate::options;

pub fn validate(
    _opts: &options::Options,
    _conf_opts: &options::ConfigOptions,
    validate_opts: &options::ValidateOpts,
) -> Result<(), failure::Error> {
    // parse config
    let config = exocore_core::cell::node_config_from_yaml_file(&validate_opts.config)?;

    // create instance to validate the config
    let (_cells, _node) = exocore_core::cell::Cell::new_from_local_node_config(config)?;

    Ok(())
}

pub fn standalone(
    _opts: &options::Options,
    _conf_opts: &options::ConfigOptions,
    convert_opts: &options::StandaloneOpts,
) -> Result<(), failure::Error> {
    let config = exocore_core::cell::node_config_from_yaml_file(&convert_opts.config)?;
    let config = exocore_core::cell::node_config_to_standalone(config)?;

    if convert_opts.format == "json" {
        println!("{}", exocore_core::cell::node_config_to_json(&config)?);
    } else {
        println!("{}", exocore_core::cell::node_config_to_yaml(&config)?);
    }

    Ok(())
}
