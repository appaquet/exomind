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

pub fn convert(
    _opts: &options::Options,
    _conf_opts: &options::ConfigOptions,
    convert_opts: &options::ConvertOpts,
) -> Result<(), failure::Error> {
    let config = exocore_core::cell::node_config_from_yaml_file(&convert_opts.config)?;

    println!("{}", serde_json::to_string_pretty(&config)?);

    Ok(())
}
