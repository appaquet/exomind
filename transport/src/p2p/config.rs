use std::time::Duration;

use exocore_core::cell::LocalNode;
use libp2p::Multiaddr;

use crate::Error;

/// `Libp2pTransport` configuration.
#[derive(Clone)]
pub struct Libp2pTransportConfig {
    pub listen_addresses: Vec<Multiaddr>,
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub handles_to_behaviour_channel_size: usize,
    pub swarm_nodes_update_interval: Duration,
}

impl Libp2pTransportConfig {
    pub fn listen_addresses(&self, local_node: &LocalNode) -> Result<Vec<Multiaddr>, Error> {
        let mut conf_addresses = self.listen_addresses.clone();
        let mut node_addresses = local_node.p2p_addresses();

        node_addresses.append(&mut conf_addresses);

        Ok(node_addresses)
    }
}

impl Default for Libp2pTransportConfig {
    fn default() -> Self {
        Libp2pTransportConfig {
            listen_addresses: Vec::new(),
            handle_in_channel_size: 1000,
            handle_out_channel_size: 1000,
            handles_to_behaviour_channel_size: 5000,
            swarm_nodes_update_interval: Duration::from_secs(1),
        }
    }
}
