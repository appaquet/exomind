use std::{
    collections::{HashMap, HashSet},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use exocore_protos::generated::exocore_core::cell_node_config;

use super::{Cell, Error, LocalNode, Node, NodeId};

/// Common methods collection of nodes of a `Cell`
pub trait CellNodes {
    fn cell(&self) -> &Cell;
    fn nodes_map(&self) -> &HashMap<NodeId, CellNode>;

    fn local_node(&self) -> &LocalNode {
        self.cell().local_node()
    }

    fn local_cell_node(&self) -> &CellNode {
        let local_node = self.cell().local_node();
        self.nodes_map()
            .get(local_node.id())
            .expect("Local node couldn't be found in cell nodes")
    }

    fn count(&self) -> usize {
        self.nodes_map().len()
    }

    fn count_with_role(&self, role: CellNodeRole) -> usize {
        self.nodes_map()
            .values()
            .filter(|cn| cn.has_role(role))
            .count()
    }

    fn is_empty(&self) -> bool {
        self.count() == 0
    }

    fn get(&self, node_id: &NodeId) -> Option<&CellNode> {
        self.nodes_map().get(node_id)
    }

    fn has_quorum(&self, count: usize, role: Option<CellNodeRole>) -> bool {
        let nb_nodes = if let Some(role) = role {
            self.count_with_role(role)
        } else {
            self.count()
        };

        if nb_nodes == 0 {
            false
        } else {
            count > nb_nodes / 2
        }
    }

    fn to_owned(&self) -> CellNodesOwned {
        CellNodesOwned {
            cell: self.cell().clone(),
            nodes: self.nodes_map().clone(),
        }
    }
}

/// Node that is part of a cell.
#[derive(Clone)]
pub struct CellNode {
    node: Node,
    roles: HashSet<CellNodeRole>,
}

impl CellNode {
    pub fn new(node: Node) -> CellNode {
        CellNode {
            node,
            roles: HashSet::new(),
        }
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn add_role(&mut self, role: CellNodeRole) {
        self.roles.insert(role);
    }

    pub fn remove_role(&mut self, role: CellNodeRole) {
        self.roles.remove(&role);
    }

    pub fn roles(&self) -> Vec<CellNodeRole> {
        self.roles.iter().cloned().collect()
    }

    pub fn has_role(&self, role: CellNodeRole) -> bool {
        self.roles.contains(&role)
    }
}

/// Wraps a `CellNodes` to expose iterator methods. This is needed because of
/// the complexity of return types of iterators which require `impl` to be used,
/// but cannot be used in traits.
pub struct CellNodesIter<'cn, N: CellNodes> {
    nodes: &'cn N,
}

impl<N: CellNodes> CellNodesIter<'_, N> {
    pub fn all(&self) -> impl Iterator<Item = &CellNode> {
        self.nodes.nodes_map().values()
    }

    pub fn all_except<'a>(
        &'a self,
        node_id: &'a NodeId,
    ) -> impl Iterator<Item = &'a CellNode> + 'a {
        self.nodes
            .nodes_map()
            .values()
            .filter(move |n| n.node.id() != node_id)
    }

    pub fn all_except_local(&self) -> impl Iterator<Item = &CellNode> {
        let local_node = self.nodes.cell().local_node();
        self.all_except(local_node.id())
    }

    pub fn with_role(&self, role: CellNodeRole) -> impl Iterator<Item = &CellNode> {
        self.nodes
            .nodes_map()
            .values()
            .filter(move |cn| cn.has_role(role))
    }
}

/// Read reference to nodes of a `Cell`
pub struct CellNodesRead<'cell> {
    pub(crate) cell: &'cell Cell,
    pub(crate) nodes: RwLockReadGuard<'cell, HashMap<NodeId, CellNode>>,
}

impl CellNodesRead<'_> {
    pub fn iter(&self) -> CellNodesIter<CellNodesRead> {
        CellNodesIter { nodes: self }
    }
}

impl CellNodes for CellNodesRead<'_> {
    fn cell(&self) -> &Cell {
        self.cell
    }

    fn nodes_map(&self) -> &HashMap<NodeId, CellNode> {
        &self.nodes
    }
}

/// Write reference to nodes of a `Cell`
pub struct CellNodesWrite<'cell> {
    pub(crate) cell: &'cell Cell,
    pub(crate) nodes: RwLockWriteGuard<'cell, HashMap<NodeId, CellNode>>,
}

impl CellNodesWrite<'_> {
    pub fn iter(&self) -> CellNodesIter<CellNodesWrite> {
        CellNodesIter { nodes: self }
    }

    pub fn add(&mut self, node: Node) {
        self.add_cell_node(CellNode {
            node,
            roles: HashSet::new(),
        });
    }

    pub fn add_cell_node(&mut self, cell_node: CellNode) {
        self.nodes.insert(cell_node.node.id().clone(), cell_node);
    }

    pub fn get_mut(&mut self, node_id: &NodeId) -> Option<&mut CellNode> {
        self.nodes.get_mut(node_id)
    }

    pub fn local_cell_node_mut(&mut self) -> &mut CellNode {
        let id = {
            let local_node = self.cell().local_node();
            local_node.id().clone()
        };
        self.nodes
            .get_mut(&id)
            .expect("Local node couldn't be found in cell nodes")
    }
}

impl CellNodes for CellNodesWrite<'_> {
    fn cell(&self) -> &Cell {
        self.cell
    }

    fn nodes_map(&self) -> &HashMap<NodeId, CellNode> {
        &self.nodes
    }
}

/// Owned copy of nodes of a `Cell`
pub struct CellNodesOwned {
    pub(crate) cell: Cell,
    pub(crate) nodes: HashMap<NodeId, CellNode>,
}

impl CellNodesOwned {
    pub fn iter(&self) -> CellNodesIter<CellNodesOwned> {
        CellNodesIter { nodes: self }
    }
}

impl CellNodes for CellNodesOwned {
    fn cell(&self) -> &Cell {
        &self.cell
    }

    fn nodes_map(&self) -> &HashMap<NodeId, CellNode> {
        &self.nodes
    }
}

/// Role of node in a cell, indicating the capabilities of a node in the cell.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CellNodeRole {
    /// Indicates that the node participates in the chain storage & replication.
    Chain,

    /// Indicates that the node is running a full entities store.
    Store,

    /// Indicates that the node is an applications host.
    AppHost,
}

impl CellNodeRole {
    pub fn from_config(config: cell_node_config::Role) -> Result<CellNodeRole, Error> {
        match config {
            cell_node_config::Role::ChainRole => Ok(CellNodeRole::Chain),
            cell_node_config::Role::StoreRole => Ok(CellNodeRole::Store),
            cell_node_config::Role::AppHostRole => Ok(CellNodeRole::AppHost),
            cell_node_config::Role::InvalidRole => {
                Err(Error::Cell(anyhow!("Invalid cell node role")))
            }
        }
    }

    pub fn to_config(&self) -> cell_node_config::Role {
        match self {
            CellNodeRole::Chain => cell_node_config::Role::ChainRole,
            CellNodeRole::Store => cell_node_config::Role::StoreRole,
            CellNodeRole::AppHost => cell_node_config::Role::AppHostRole,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::FullCell, *};

    #[test]
    fn nodes_add_get() {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node.clone()).unwrap();

        {
            let nodes = full_cell.cell().nodes();
            assert!(!nodes.is_empty());
            assert_eq!(nodes.count(), 1); // self
        }

        {
            let mut nodes = full_cell.cell().nodes_mut();
            nodes.add(Node::generate_temporary());
            assert_eq!(nodes.count(), 2);
            assert_eq!(nodes.iter().all().count(), 2);
        }

        {
            let nodes = full_cell.cell().nodes();
            assert_eq!(nodes.count(), 2);
            assert_eq!(nodes.iter().all().count(), 2);
            assert_eq!(nodes.iter().all_except(local_node.id()).count(), 1);
            assert_ne!(
                nodes.iter().all_except_local().next().unwrap().node.id(),
                local_node.id()
            );

            assert!(nodes.get(local_node.id()).is_some());

            let other_node = Node::generate_temporary();
            assert!(nodes.get(other_node.id()).is_none());
        }

        {
            let nodes = full_cell.cell().nodes().to_owned();
            assert_eq!(nodes.count(), 2);
        }
    }

    #[test]
    fn nodes_quorum() {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node).unwrap();

        {
            // 1 node
            let nodes = full_cell.cell().nodes();
            assert!(!nodes.has_quorum(0, None));
            assert!(nodes.has_quorum(1, None));
            assert!(!nodes.has_quorum(0, Some(CellNodeRole::Chain)));
            assert!(!nodes.has_quorum(1, Some(CellNodeRole::Chain)));
        }

        {
            // 2 nodes
            let mut nodes = full_cell.cell().nodes_mut();
            nodes.add(Node::generate_temporary());
            assert!(!nodes.has_quorum(1, None));
            assert!(nodes.has_quorum(2, None));
        }

        {
            // 3 nodes
            let mut nodes = full_cell.cell().nodes_mut();
            nodes.add(Node::generate_temporary());
            assert!(!nodes.has_quorum(1, None));
            assert!(nodes.has_quorum(2, None));
        }

        {
            // 3 nodes with roles
            let mut nodes = full_cell.cell().nodes_mut();
            let ids = nodes
                .iter()
                .all()
                .map(|n| n.node.id())
                .cloned()
                .collect::<Vec<_>>();
            nodes
                .get_mut(&ids[0])
                .unwrap()
                .add_role(CellNodeRole::Chain);
            nodes
                .get_mut(&ids[1])
                .unwrap()
                .add_role(CellNodeRole::Chain);

            assert!(!nodes.has_quorum(1, Some(CellNodeRole::Chain)));
            assert!(nodes.has_quorum(2, Some(CellNodeRole::Chain)));
        }
    }
}
