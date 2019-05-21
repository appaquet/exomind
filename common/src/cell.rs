//
// TODO: To be cleaned up in https://github.com/appaquet/exocore/issues/37
// TODO: Encryption/signature ticket: https://github.com/appaquet/exocore/issues/46
//

use crate::node::{LocalNode, Nodes};

/// A Cell represents a private enclosure in which the data and applications of a user
/// are hosted. A Cell resides on multiple nodes.
#[derive(Clone)]
pub struct Cell {
    cell_id: CellID,
    nodes: Nodes,
}

impl Cell {
    pub fn new(local_node: LocalNode, cell_id: CellID) -> Cell {
        Cell {
            cell_id,
            nodes: Nodes::new_with_local(local_node),
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

    #[deprecated]
    pub fn nodes_mut(&mut self) -> &mut Nodes {
        &mut self.nodes
    }
}

/// Unique identifier of a cell, which is built by hashing the public key
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
