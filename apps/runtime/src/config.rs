use std::time::Duration;

use exocore_core::utils::backoff::BackoffConfig;

/// Applications configuration.
#[derive(Clone, Copy)]
pub struct Config {
    pub restart_backoff: BackoffConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            restart_backoff: BackoffConfig {
                normal_constant: Duration::from_secs(1),
                failure_constant: Duration::from_secs(1),
                failure_exp_base: 2.0,
                failure_exp_multiplier: Duration::from_secs(5),
                failure_maximum: Duration::from_secs(30),
            },
        }
    }
}
