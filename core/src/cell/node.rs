use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
    sync::{Arc, RwLock},
};

use exocore_protos::{
    core::NodeAddresses,
    generated::exocore_core::{LocalNodeConfig, NodeConfig},
};
use libp2p::core::{Multiaddr, PeerId};
use url::Url;

use super::error::Error;
use crate::sec::{
    keys::{Keypair, PublicKey},
    signature::Signature,
};

/// Represents a machine / process on which Exocore runs. A node can host
/// multiple `Cell`.
#[derive(Clone)]
pub struct Node {
    identity: Arc<NodeIdentity>,
    addresses: Arc<RwLock<Addresses>>,
}

struct NodeIdentity {
    node_id: NodeId,
    peer_id: PeerId,
    consistent_clock_id: u16,
    public_key: PublicKey,
    name: String,
}

impl Node {
    pub fn new_from_public_key(public_key: PublicKey) -> Node {
        Self::build(public_key, None)
    }

    pub fn new_from_config(config: NodeConfig) -> Result<Node, Error> {
        let public_key = PublicKey::decode_base58_string(&config.public_key)
            .map_err(|err| Error::Cell(anyhow!("Couldn't decode node public key: {}", err)))?;

        let name = if !config.name.is_empty() {
            Some(config.name)
        } else {
            None
        };

        let node = Self::build(public_key, name);

        {
            let mut addresses = node.addresses.write().unwrap();
            *addresses = Addresses::parse(&config.addresses.unwrap_or_default())?;
        }

        Ok(node)
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn generate_temporary() -> Node {
        let keypair = Keypair::generate_ed25519();
        Self::build(keypair.public(), None)
    }

    fn build(public_key: PublicKey, name: Option<String>) -> Node {
        let node_id = NodeId::from_public_key(&public_key);
        let peer_id = *node_id.to_peer_id();

        let node_id_bytes = node_id.0.to_bytes();
        let node_id_bytes_len = node_id_bytes.len();
        let consistent_clock_id = u16::from_le_bytes([
            node_id_bytes[node_id_bytes_len - 1],
            node_id_bytes[node_id_bytes_len - 2],
        ]);

        let name = name.unwrap_or_else(|| public_key.generate_name());

        Node {
            identity: Arc::new(NodeIdentity {
                node_id,
                peer_id,
                consistent_clock_id,
                public_key,
                name,
            }),
            addresses: Arc::new(RwLock::new(Addresses {
                p2p: HashSet::new(),
                http: HashSet::new(),
            })),
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.identity.node_id
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.identity.public_key
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.identity.peer_id
    }

    pub fn name(&self) -> &str {
        &self.identity.name
    }

    pub fn consistent_clock_id(&self) -> u16 {
        self.identity.consistent_clock_id
    }

    pub fn p2p_addresses(&self) -> Vec<Multiaddr> {
        let addresses = self.addresses.read().expect("Couldn't get addresses lock");
        addresses.p2p.iter().cloned().collect()
    }

    pub fn add_p2p_address(&self, address: Multiaddr) {
        let mut addresses = self.addresses.write().expect("Couldn't get addresses lock");
        addresses.p2p.insert(address);
    }

    pub fn http_addresses(&self) -> Vec<Url> {
        let addresses = self.addresses.read().expect("Couldn't get addresses lock");
        addresses.http.iter().cloned().collect()
    }

    pub fn add_http_address(&self, address: Url) {
        let mut addresses = self.addresses.write().expect("Couldn't get addresses lock");
        addresses.http.insert(address);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.identity.node_id.eq(&other.identity.node_id)
    }
}

impl Eq for Node {}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let addresses = self.addresses.read().expect("Couldn't get addresses lock");
        f.debug_struct("Node")
            .field("name", &self.identity.name)
            .field("node_id", &self.identity.node_id)
            .field(
                "public_key",
                &self.identity.public_key.encode_base58_string(),
            )
            .field("p2p_addresses", &addresses.p2p)
            .field("http_addresses", &addresses.http)
            .finish()
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Node{")?;
        f.write_str(&self.identity.name)?;
        f.write_str("}")
    }
}

/// Represents the local `Node` being run in the current process. Contrarily to
/// other nodes, we have a full private+public keypair that we can sign messages
/// with.
#[derive(Clone)]
pub struct LocalNode {
    node: Node,
    identity: Arc<LocalNodeIdentity>,
}

struct LocalNodeIdentity {
    keypair: Keypair,
    config: LocalNodeConfig,
    addresses: Addresses,
}

impl LocalNode {
    pub fn generate() -> LocalNode {
        let keypair = Keypair::generate_ed25519();
        let node = Node::new_from_public_key(keypair.public());
        let node_name = node.name().to_string();

        let config = LocalNodeConfig {
            keypair: keypair.encode_base58_string(),
            public_key: keypair.public().encode_base58_string(),
            id: node.id().to_string(),
            name: node_name,
            ..Default::default()
        };

        Self::new_from_config(config).expect("Couldn't create node config generated config")
    }

    pub fn new_from_config(config: LocalNodeConfig) -> Result<Self, Error> {
        let keypair = Keypair::decode_base58_string(&config.keypair)
            .map_err(|err| Error::Cell(anyhow!("Couldn't decode local node keypair: {}", err)))?;

        let listen_addresses =
            Addresses::parse(&config.listen_addresses.clone().unwrap_or_default())?;
        let local_node = LocalNode {
            node: Node::new_from_config(NodeConfig {
                public_key: config.public_key.clone(),
                name: config.name.clone(),
                id: config.id.clone(),
                addresses: config.addresses.clone(),
            })?,
            identity: Arc::new(LocalNodeIdentity {
                keypair,
                config,
                addresses: listen_addresses,
            }),
        };

        Ok(local_node)
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn keypair(&self) -> &Keypair {
        &self.identity.keypair
    }

    pub fn sign_message(&self, _message: &[u8]) -> Signature {
        // TODO: Signature ticket: https://github.com/appaquet/exocore/issues/46
        //       Make sure we're local and we have access to private key
        Signature::empty()
    }

    pub fn config(&self) -> &LocalNodeConfig {
        &self.identity.config
    }

    pub fn p2p_listen_addresses(&self) -> Vec<Multiaddr> {
        if !self.identity.addresses.p2p.is_empty() {
            self.identity.addresses.p2p.iter().cloned().collect()
        } else {
            self.p2p_addresses()
        }
    }

    pub fn http_listen_addresses(&self) -> Vec<Url> {
        if !self.identity.addresses.http.is_empty() {
            self.identity.addresses.http.iter().cloned().collect()
        } else {
            self.http_addresses()
        }
    }
}

impl Deref for LocalNode {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl PartialEq for LocalNode {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(other)
    }
}

impl Eq for LocalNode {}

impl Debug for LocalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("LocalNode")
            .field("node", &self.node)
            .field("p2p_listen_addresses", &self.identity.addresses.p2p)
            .field("http_listen_addresses", &self.identity.addresses.http)
            .finish()
    }
}

impl Display for LocalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("LocalNode{")?;
        f.write_str(&self.identity.config.name)?;
        f.write_str("}")
    }
}

/// Unique identifier of a node, which is built by hashing the public key of the
/// node.
///
/// For now, it has a one to one correspondence with libp2p's PeerId, which is a
/// base58 encoded version of the public key of the node encoded in protobuf.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct NodeId(PeerId);

impl NodeId {
    /// Create a Node ID from a public key by using libp2p method to support
    /// compatibility with PeerId
    pub fn from_public_key(public_key: &PublicKey) -> NodeId {
        let peer_id = PeerId::from_public_key(public_key.to_libp2p().clone());
        NodeId(peer_id)
    }

    pub fn from_peer_id(peer_id: PeerId) -> NodeId {
        NodeId(peer_id)
    }

    pub fn to_peer_id(&self) -> &PeerId {
        &self.0
    }

    pub fn from_bytes(id: Vec<u8>) -> Result<NodeId, Error> {
        let peer_id = PeerId::from_bytes(id.as_ref())
            .map_err(|_| Error::Node(anyhow!("Couldn't convert bytes to peer id")))?;
        Ok(NodeId(peer_id))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

impl std::fmt::Display for NodeId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for NodeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let peer_id = PeerId::from_str(s).map_err(|_| ())?;
        Ok(NodeId(peer_id))
    }
}

/// Addresses of a node.
struct Addresses {
    p2p: HashSet<Multiaddr>,
    http: HashSet<url::Url>,
}

impl Addresses {
    fn parse(config: &NodeAddresses) -> Result<Addresses, Error> {
        let mut addresses = Addresses {
            p2p: HashSet::new(),
            http: HashSet::new(),
        };
        for maddr_str in &config.p2p {
            let maddr = maddr_str
                .parse()
                .map_err(|err| Error::Cell(anyhow!("Couldn't parse p2p multi-address: {}", err)))?;
            addresses.p2p.insert(maddr);
        }

        for url_str in &config.http {
            let url = url_str
                .parse()
                .map_err(|err| Error::Cell(anyhow!("Couldn't parse http url: {}", err)))?;
            addresses.http.insert(url);
        }

        Ok(addresses)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::eq_op)] // since we test node's equality

    use super::*;

    #[test]
    fn node_equality() {
        let node1 = LocalNode::generate();
        let node2 = LocalNode::generate();

        assert_eq!(node1, node1);
        assert_eq!(node1, node1.clone());
        assert_ne!(node1, node2);
    }

    #[test]
    fn node_id_bytes() {
        let node1 = LocalNode::generate();
        let node2 = LocalNode::generate();

        assert_ne!(node1.id().to_bytes(), node2.id().to_bytes());
        assert_eq!(node1.id().to_bytes(), node1.id().to_bytes());

        let n1_bytes = node1.id().to_bytes();
        let n1_id_bytes = NodeId::from_bytes(n1_bytes.to_vec()).unwrap();
        assert_eq!(n1_id_bytes, *node1.id());
    }

    #[test]
    fn node_deterministic_random_name() {
        let pk = PublicKey::decode_base58_string("pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ")
            .unwrap();
        let node = Node::new_from_public_key(pk);
        assert_eq!("wholly-proud-gannet", node.identity.name);
        assert_eq!("Node{wholly-proud-gannet}", node.to_string());
    }

    #[test]
    fn local_node_from_generated_config() {
        let node1 = LocalNode::generate();
        let node2 = LocalNode::new_from_config(node1.config().clone()).unwrap();

        assert_eq!(node1.keypair().public(), node2.keypair().public());
        assert_eq!(node1.config(), node2.config());
    }
}
