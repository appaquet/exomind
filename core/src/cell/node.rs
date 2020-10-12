use super::error::Error;
use crate::protos::generated::exocore_core::{LocalNodeConfig, NodeConfig};
use crate::sec::signature::Signature;
use crate::{
    protos::core::NodeAddresses,
    sec::keys::{Keypair, PublicKey},
};
use libp2p::core::{Multiaddr, PeerId};
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use url::Url;

/// Represents a machine / process on which Exocore runs. A node can host
/// multiple `Cell`.
#[derive(Clone)]
pub struct Node {
    identity: Arc<Identity>,
    inner: Arc<RwLock<SharedInner>>,
}

struct Identity {
    node_id: NodeId,
    peer_id: PeerId,
    consistent_clock_id: u16,
    public_key: PublicKey,
    name: String,
}

struct SharedInner {
    p2p_addresses: HashSet<Multiaddr>,
    http_addresses: HashSet<url::Url>,
}

impl Node {
    pub fn new_from_public_key(public_key: PublicKey) -> Node {
        Self::build(public_key, None)
    }

    pub fn new_from_config(config: NodeConfig) -> Result<Node, Error> {
        let public_key = PublicKey::decode_base58_string(&config.public_key)
            .map_err(|err| Error::Cell(format!("Couldn't decode node public key: {}", err)))?;

        let name = if !config.name.is_empty() {
            Some(config.name)
        } else {
            None
        };

        let node = Self::build(public_key, name);
        parse_node_addresses(&node, &config.addresses.unwrap_or_default())?;

        Ok(node)
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn generate_temporary() -> Node {
        let keypair = Keypair::generate_ed25519();
        Self::build(keypair.public(), None)
    }

    fn build(public_key: PublicKey, name: Option<String>) -> Node {
        let node_id = NodeId::from_public_key(&public_key);
        let peer_id = node_id.to_peer_id().clone();

        let node_id_bytes = node_id.0.as_bytes();
        let node_id_bytes_len = node_id_bytes.len();
        let consistent_clock_id = u16::from_le_bytes([
            node_id_bytes[node_id_bytes_len - 1],
            node_id_bytes[node_id_bytes_len - 2],
        ]);

        let name = name.unwrap_or_else(|| public_key.generate_name());

        Node {
            identity: Arc::new(Identity {
                node_id,
                peer_id,
                consistent_clock_id,
                public_key,
                name,
            }),
            inner: Arc::new(RwLock::new(SharedInner {
                p2p_addresses: HashSet::new(),
                http_addresses: HashSet::new(),
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
        let inner = self.inner.read().expect("Couldn't get inner lock");
        inner.p2p_addresses.iter().cloned().collect()
    }

    pub fn add_p2p_address(&self, address: Multiaddr) {
        let mut inner = self.inner.write().expect("Couldn't get inner lock");
        inner.p2p_addresses.insert(address);
    }

    pub fn http_addresses(&self) -> Vec<Url> {
        let inner = self.inner.read().expect("Couldn't get inner lock");
        inner.http_addresses.iter().cloned().collect()
    }

    pub fn add_http_address(&self, address: Url) {
        let mut inner = self.inner.write().expect("Couldn't get inner lock");
        inner.http_addresses.insert(address);
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
        let inner = self.inner.read().expect("Couldn't get inner lock");
        f.debug_struct("Node")
            .field("name", &self.identity.name)
            .field("node_id", &self.identity.node_id)
            .field(
                "public_key",
                &self.identity.public_key.encode_base58_string(),
            )
            .field("p2p_addresses", &inner.p2p_addresses)
            .field("http_addresses", &inner.http_addresses)
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
    keypair: Arc<Keypair>,
}

impl LocalNode {
    pub fn new_from_keypair(keypair: Keypair) -> LocalNode {
        LocalNode {
            node: Node::new_from_public_key(keypair.public()),
            keypair: Arc::new(keypair),
        }
    }

    pub fn new_from_config(config: LocalNodeConfig) -> Result<Self, Error> {
        let keypair = Keypair::decode_base58_string(&config.keypair)
            .map_err(|err| Error::Cell(format!("Couldn't decode local node keypair: {}", err)))?;

        let node = Self::new_from_keypair(keypair);
        parse_node_addresses(node.node(), &config.addresses.unwrap_or_default())?;

        Ok(node)
    }

    pub fn generate() -> LocalNode {
        LocalNode::new_from_keypair(Keypair::generate_ed25519())
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn sign_message(&self, _message: &[u8]) -> Signature {
        // TODO: Signature ticket: https://github.com/appaquet/exocore/issues/46
        //       Make sure we're local and we have access to private key
        Signature::empty()
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
        self.node.eq(&other)
    }
}

impl Eq for LocalNode {}

impl Debug for LocalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("LocalNode")
            .field("node", &self.node)
            .finish()
    }
}

impl Display for LocalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("LocalNode{")?;
        f.write_str(&self.identity.name)?;
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
        let peer_id = PeerId::from_bytes(id)
            .map_err(|_| Error::Node("Couldn't convert bytes to peer id".to_string()))?;
        Ok(NodeId(peer_id))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
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

fn parse_node_addresses(node: &Node, addresses: &NodeAddresses) -> Result<(), Error> {
    for maddr_str in &addresses.p2p {
        let maddr = maddr_str
            .parse()
            .map_err(|err| Error::Cell(format!("Couldn't parse p2p multi-address: {}", err)))?;
        node.add_p2p_address(maddr);
    }

    for url_str in &addresses.http {
        let url = url_str
            .parse()
            .map_err(|err| Error::Cell(format!("Couldn't parse http url: {}", err)))?;
        node.add_http_address(url);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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

        assert_ne!(node1.id().as_bytes(), node2.id().as_bytes());
        assert_eq!(node1.id().as_bytes(), node1.id().as_bytes());

        let n1_bytes = node1.id().as_bytes();
        let n1_id_bytes = NodeId::from_bytes(n1_bytes.to_vec()).unwrap();
        assert_eq!(n1_id_bytes, *node1.id());
    }

    #[test]
    fn node_deterministic_random_name() {
        let pk = PublicKey::decode_base58_string("pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ")
            .unwrap();
        let node = Node::new_from_public_key(pk);
        assert_eq!("early-settled-ram", node.identity.name);
        assert_eq!("Node{early-settled-ram}", node.to_string());
    }
}
