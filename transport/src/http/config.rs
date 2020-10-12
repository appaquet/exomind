use std::time::Duration;

use exocore_core::cell::LocalNode;
use url::Url;

use crate::Error;

/// Configuration for HTTP transport.
#[derive(Clone)]
pub struct HTTPTransportConfig {
    pub listen_addresses: Vec<Url>,
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub request_timeout: Duration,
}

impl HTTPTransportConfig {
    pub fn listen_addresses(&self, local_node: &LocalNode) -> Result<Vec<Url>, Error> {
        let mut conf_addresses = self.listen_addresses.clone();
        let mut node_addresses = local_node.http_addresses();

        node_addresses.append(&mut conf_addresses);

        Ok(node_addresses)
    }
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
