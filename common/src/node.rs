pub type NodeID = String;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    // TODO: PublicKey
    // TODO: NodeID = hash(publickey)
    // TODO: ACLs
    id: NodeID,
    //    address: String,
    //    is_me: bool,
}

impl Node {
    pub fn new(id: String) -> Node {
        Node { id }
    }

    pub fn get_id(&self) -> &NodeID {
        &self.id
    }
}

pub struct Nodes {
    nodes: Vec<Node>,
}

impl Default for Nodes {
    fn default() -> Self {
        Nodes { nodes: Vec::new() }
    }
}

impl Nodes {
    pub fn add(&mut self, node: Node) {
        self.nodes.push(node);
    }
}
