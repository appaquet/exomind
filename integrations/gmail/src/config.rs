use exocore::core::utils::path::child_to_abs_path;
use std::path::{Path, PathBuf};

#[derive(Clone, Deserialize)]
pub struct Config {
    pub client_secret: PathBuf,

    pub tokens_directory: PathBuf,

    pub save_fixtures: bool,
}

impl Config {
    pub fn make_abs_path(&mut self, config_dir: &Path) {
        self.client_secret = child_to_abs_path(config_dir, &self.client_secret);
        self.tokens_directory = child_to_abs_path(config_dir, &self.tokens_directory);
    }
}
