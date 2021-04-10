use exocore::core::utils::path::child_to_abs_path;
use std::path::{Path, PathBuf};

#[derive(Clone, Deserialize)]
pub struct Config {
    pub client_secret: PathBuf,

    pub tokens_directory: PathBuf,

    pub save_fixtures: bool,
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Config> {
        let file = std::fs::File::open(path)?;
        let mut config: Config = serde_yaml::from_reader(file)?;

        let config_dir = path
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
