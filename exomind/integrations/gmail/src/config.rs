use std::path::{Path, PathBuf};

use duration_string::DurationString;
use exocore::core::utils::path::child_to_abs_path;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub client_secret: PathBuf,

    pub tokens_directory: PathBuf,

    pub save_fixtures: bool,

    pub full_sync_interval: DurationString,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut config: Config = serde_yaml::from_reader(file)?;

        let config_dir = path
            .as_ref()
            .parent()
            .ok_or_else(|| anyhow!("Couldn't get config parent directory"))?;

        config.make_abs_path(config_dir);

        Ok(config)
    }

    pub fn make_abs_path(&mut self, config_dir: &Path) {
        self.client_secret = child_to_abs_path(config_dir, &self.client_secret);
        self.tokens_directory = child_to_abs_path(config_dir, &self.tokens_directory);
    }
}
