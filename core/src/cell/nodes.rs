use super::{Cell, LocalNode, Node, NodeId};
use std::collections::HashMap;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// Common methods collection of nodes of a `Cell`
pub trait CellNodes {
    fn cell(&self) -> &Cell;
    fn nodes_map(&self) -> &HashMap<NodeId, Node>;

    #[inline]
    fn local_node(&self) -> &LocalNode {
        self.cell().local_node()
    }

    #[inline]
    fn len(&self) -> usize {
        self.nodes_map().len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn get(&self, node_id: &NodeId) -> Option<&Node> {
        self.nodes_map().get(node_id)
    }

    fn is_quorum(&self, count: usize) -> bool {
        if self.is_empty() {
            false
        } else if self.len() == 1 {
            count == 1
        } else if self.len() == 2 {
            count == 2
        } else {
            count > self.len() / 2
        }
    }

    fn to_owned(&self) -> CellNodesOwned {
        CellNodesOwned {
            cell: self.cell().clone(),
            nodes: self.nodes_map().clone(),
        }
    }
}

/// Wraps a `CellNodes` to expose iterator methods. This is needed because of
/// the complexity of return types of iterators which require `impl` to be used,
/// but cannot be used in traits.
pub struct CellNodesIter<'cn, N: CellNodes> {
    nodes: &'cn N,
}

impl<'cn, N: CellNodes> CellNodesIter<'cn, N> {
    pub fn all(&self) -> impl Iterator<Item = &Node> {
        self.nodes.nodes_map().values()
    }

    pub fn all_except<'a>(&'a self, node_id: &'a NodeId) -> impl Iterator<Item = &'a Node> + 'a {
        self.nodes
            .nodes_map()
            .values()
            .filter(move |n| n.id() != node_id)
    }

    pub fn all_except_local<'a>(&'a self) -> impl Iterator<Item = &'a Node> + 'a {
        let local_node = self.nodes.cell().local_node();
        self.all_except(local_node.id())
    }
}

/// Read reference to nodes of a `Cell`
pub struct CellNodesRead<'cell> {
    pub(crate) cell: &'cell Cell,
    pub(crate) nodes: RwLockReadGuard<'cell, HashMap<NodeId, Node>>,
}

impl<'cell> CellNodesRead<'cell> {
    pub fn iter(&self) -> CellNodesIter<CellNodesRead> {
        CellNodesIter { nodes: self }
    }
}

impl<'cell> CellNodes for CellNodesRead<'cell> {
    #[inline]
    fn cell(&self) -> &Cell {
        &self.cell
    }

    #[inline]
    fn nodes_map(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }
}

/// Write reference to nodes of a `Cell`
pub struct CellNodesWrite<'cell> {
    pub(crate) cell: &'cell Cell,
    pub(crate) nodes: RwLockWriteGuard<'cell, HashMap<NodeId, Node>>,
}

impl<'cell> CellNodesWrite<'cell> {
    pub fn iter(&self) -> CellNodesIter<CellNodesWrite> {
        CellNodesIter { nodes: self }
    }

    pub fn add(&mut self, node: Node) {
        self.nodes.insert(node.id().clone(), node);
    }
}

impl<'cell> CellNodes for CellNodesWrite<'cell> {
    #[inline]
    fn cell(&self) -> &Cell {
        &self.cell
    }

    #[inline]
    fn nodes_map(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }
}

/// Owned copy of nodes of a `Cell`
pub struct CellNodesOwned {
    pub(crate) cell: Cell,
    pub(crate) nodes: HashMap<NodeId, Node>,
}

impl CellNodesOwned {
    pub fn iter(&self) -> CellNodesIter<CellNodesOwned> {
        CellNodesIter { nodes: self }
    }
}

impl CellNodes for CellNodesOwned {
    #[inline]
    fn cell(&self) -> &Cell {
        &self.cell
    }

    #[inline]
    fn nodes_map(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }
}

#[cfg(test)]
mod tests {
    use super::super::FullCell;
    use super::*;

    #[test]
    fn nodes_add_get() {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node.clone());

        {
            let nodes = cell.nodes();
            assert!(!nodes.is_empty());
            assert_eq!(nodes.len(), 1); // self
        }

        {
            let mut nodes = cell.nodes_mut();
            nodes.add(Node::generate_temporary());
            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes.iter().all().count(), 2);
        }

        {
            let nodes = cell.nodes();
            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes.iter().all().count(), 2);
            assert_eq!(nodes.iter().all_except(local_node.id()).count(), 1);
            assert_ne!(
                nodes.iter().all_except_local().next().unwrap().id(),
                local_node.id()
            );

            assert!(nodes.get(local_node.id()).is_some());

            let other_node = Node::generate_temporary();
            assert!(nodes.get(other_node.id()).is_none());
        }
    }

    #[test]
    fn nodes_quorum() {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);

        {
            // only have 1 node (local_node)
            let nodes = cell.nodes();
            assert!(!nodes.is_quorum(0));
            assert!(nodes.is_quorum(1));
        }

        {
            let mut nodes = cell.nodes_mut();
            nodes.add(Node::generate_temporary());
            assert!(!nodes.is_quorum(1));
            assert!(nodes.is_quorum(2));
        }

        {
            let mut nodes = cell.nodes_mut();
            nodes.add(Node::generate_temporary());
            assert!(!nodes.is_quorum(1));
            assert!(nodes.is_quorum(2));
        }
    }
}
