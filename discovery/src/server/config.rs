use std::time::Duration;

/// Configuration for discovery service server.
#[derive(Copy, Clone)]
pub struct ServerConfig {
    /// Port to listen on.
    pub port: u16,

    /// Maximum number of payloads that the server can stores.
    pub max_payloads: usize,

    /// Maximum payload size.
    pub max_payload_size: usize,

    /// Payloads expiration delay.
    pub expiration: Duration,

    /// Interval at which expired payloads are cleaned up.
    pub cleanup_interval: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8005,
            max_payloads: 5000,
            max_payload_size: 5 << 20, // 5mb
            expiration: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(1),
        }
    }
}
