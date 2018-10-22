#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    // TODO: PublicKey
    // TODO: NodeID = hash(publickey)
    address: Address,
    is_me: bool,
}

// TODO: Could be multiaddr
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Address {}
