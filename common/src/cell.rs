// TODO: Main ticket for completion of cell: https://github.com/appaquet/exocore/issues/37
// TODO: Encryption/signature ticket: https://github.com/appaquet/exocore/issues/46

use crate::node::Nodes;

// TODO: PublicKey (RSA + secp256k1)
// TODO: NodeID = hash(publickey)
// TODO: Nodes, with their capability
#[derive(Clone)]
pub struct Cell {
    cell_id: CellID,
    nodes: Nodes,
}

impl Cell {
    pub fn new(cell_id: CellID) -> Cell {
        Cell {
            cell_id,
            nodes: Nodes::new(),
        }
    }

    #[inline]
    pub fn id(&self) -> &CellID {
        &self.cell_id
    }

    #[inline]
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct CellID {
    id: String,
}

impl CellID {
    pub fn from_string(id: &str) -> CellID {
        CellID { id: id.to_string() }
    }

    pub fn from_bytes(id: &[u8]) -> CellID {
        // TODO: fix
        CellID {
            id: String::from_utf8_lossy(id).to_string(),
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.id.as_bytes()
    }
}

// TODO: bitflags ?
// TODO: Can it decrypt ?
pub struct NodeCellCapability {}
