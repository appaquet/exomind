use std::{net::SocketAddr, time::Duration};

/// Configuration for HTTP transport.
#[derive(Clone)]
pub struct HTTPTransportConfig {
    pub listen_addresses: Vec<SocketAddr>,
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub request_timeout: Duration,
}

impl Default for HTTPTransportConfig {
    fn default() -> Self {
        HTTPTransportConfig {
            listen_addresses: Vec::new(),
            handle_in_channel_size: 1000,
            handle_out_channel_size: 1000,
            request_timeout: Duration::from_secs(5),
        }
    }
}
