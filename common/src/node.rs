
use multiaddr;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    address: multiaddr::Multiaddr,
    is_me: bool,
}

