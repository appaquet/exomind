use multiaddr;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    // TODO: PublicKey
    // TODO: NodeID = hash(publickey)
    address: multiaddr::Multiaddr,
    is_me: bool,
}
