use std::collections::{BTreeMap, HashMap};
use std::ops::RangeBounds;
use std::sync::Arc;

use super::*;
use crate::operation::Operation;

/// In memory pending store
pub struct MemoryPendingStore {
    operations_timeline: BTreeMap<OperationId, GroupId>,
    groups_operations: HashMap<GroupId, GroupOperations>,
}

impl MemoryPendingStore {
    pub fn new() -> MemoryPendingStore {
        MemoryPendingStore {
            operations_timeline: BTreeMap::new(),
            groups_operations: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn clear(&mut self) {
        self.operations_timeline.clear();
        self.groups_operations.clear();
    }
}

impl Default for MemoryPendingStore {
    fn default() -> Self {
        MemoryPendingStore::new()
    }
}

impl PendingStore for MemoryPendingStore {
    fn put_operation(&mut self, operation: operation::NewOperation) -> Result<bool, Error> {
        let operation_reader = operation.get_operation_reader()?;
        let operation_type = operation.get_type()?;

        let group_id = operation_reader.get_group_id();
        let group_operations = self
            .groups_operations
            .entry(group_id)
            .or_insert_with(GroupOperations::new);

        let operation_id = operation_reader.get_operation_id();
        group_operations.operations.insert(
            operation_id,
            GroupOperation {
                operation_id,
                operation_type,
                commit_status: CommitStatus::Unknown,
                frame: Arc::new(operation.frame),
            },
        );

        let existed = self
            .operations_timeline
            .insert(operation_id, group_id)
            .is_some();
        Ok(existed)
    }

    fn update_operation_commit_status(
        &mut self,
        operation_id: u64,
        status: CommitStatus,
    ) -> Result<(), Error> {
        let group_id = self
            .operations_timeline
            .get(&operation_id)
            .ok_or(Error::NotFound)?;
        let group = self
            .groups_operations
            .get_mut(group_id)
            .ok_or(Error::NotFound)?;
        let group_operation: &mut GroupOperation = group
            .operations
            .get_mut(&operation_id)
            .ok_or(Error::NotFound)?;

        group_operation.commit_status = status;

        Ok(())
    }

    fn get_operation(&self, operation_id: OperationId) -> Result<Option<StoredOperation>, Error> {
        let operation = self
            .operations_timeline
            .get(&operation_id)
            .and_then(|group_id| {
                self.groups_operations
                    .get(group_id)
                    .and_then(|group_operations| {
                        group_operations
                            .operations
                            .get(&operation_id)
                            .map(|op| (*group_id, op))
                    })
            })
            .map(|(group_id, op)| StoredOperation {
                group_id,
                operation_id: op.operation_id,
                operation_type: op.operation_type,
                commit_status: op.commit_status,
                frame: Arc::clone(&op.frame),
            });

        Ok(operation)
    }

    fn get_group_operations(
        &self,
        group_id: GroupId,
    ) -> Result<Option<StoredOperationsGroup>, Error> {
        let operations = self.groups_operations.get(&group_id).map(|group_ops| {
            let operations = group_ops
                .operations
                .values()
                .map(|op| StoredOperation {
                    group_id,
                    operation_id: op.operation_id,
                    operation_type: op.operation_type,
                    commit_status: op.commit_status,
                    frame: Arc::clone(&op.frame),
                })
                .collect();

            StoredOperationsGroup {
                group_id,
                operations,
            }
        });

        Ok(operations)
    }

    fn operations_iter<R>(&self, range: R) -> Result<TimelineIterator, Error>
    where
        R: RangeBounds<OperationId>,
    {
        let ids_iterator = self
            .operations_timeline
            .range(range)
            .map(|(op_id, group_id)| (*op_id, *group_id));

        Ok(Box::new(OperationsIterator {
            store: self,
            ids_iterator: Box::new(ids_iterator),
        }))
    }

    fn operations_count(&self) -> usize {
        self.operations_timeline.len()
    }

    fn delete_operation(&mut self, operation_id: OperationId) -> Result<(), Error> {
        if let Some(group_operations_id) = self
            .groups_operations
            .get(&operation_id)
            .map(|group| group.operations.keys())
        {
            // the operation is a group, we delete all its operations
            for operation_id in group_operations_id {
                self.operations_timeline.remove(&operation_id);
            }
            self.operations_timeline.remove(&operation_id);
            self.groups_operations.remove(&operation_id);
        } else {
            // operation is part of a group, we delete the operation from it
            if let Some(group_id) = self.operations_timeline.get(&operation_id) {
                if let Some(group) = self.groups_operations.get_mut(&group_id) {
                    group.operations.remove(&operation_id);
                }
            }
            self.operations_timeline.remove(&operation_id);
        }

        Ok(())
    }
}

impl MemoryPendingStore {
    fn get_group_operation(
        &self,
        group_id: GroupId,
        operation_id: OperationId,
    ) -> Option<&GroupOperation> {
        self.groups_operations
            .get(&group_id)
            .and_then(|group_ops| group_ops.operations.get(&operation_id))
    }
}

struct GroupOperations {
    operations: BTreeMap<OperationId, GroupOperation>,
}

impl GroupOperations {
    fn new() -> GroupOperations {
        GroupOperations {
            operations: BTreeMap::new(),
        }
    }
}

struct GroupOperation {
    operation_id: OperationId,
    operation_type: operation::OperationType,
    commit_status: CommitStatus,
    frame: Arc<crate::operation::OperationFrame<Vec<u8>>>,
}

struct OperationsIterator<'store> {
    store: &'store MemoryPendingStore,
    ids_iterator: Box<dyn Iterator<Item = (OperationId, GroupId)> + 'store>,
}

impl<'store> Iterator for OperationsIterator<'store> {
    type Item = StoredOperation;

    fn next(&mut self) -> Option<StoredOperation> {
        let (operation_id, group_id) = self.ids_iterator.next()?;
        let group_operation = self.store.get_group_operation(group_id, operation_id)?;

        Some(StoredOperation {
            group_id,
            operation_id,
            operation_type: group_operation.operation_type,
            commit_status: group_operation.commit_status,
            frame: Arc::clone(&group_operation.frame),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::engine::testing::create_dummy_new_entry_op;

    use super::*;
    use exocore_core::cell::LocalNode;

    #[test]
    fn put_and_retrieve_operation() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let mut store = MemoryPendingStore::new();

        store.put_operation(create_dummy_new_entry_op(&local_node, 105, 200))?;
        store.put_operation(create_dummy_new_entry_op(&local_node, 100, 200))?;
        store.put_operation(create_dummy_new_entry_op(&local_node, 102, 201))?;

        let timeline: Vec<(OperationId, GroupId)> = store
            .operations_iter(..)?
            .map(|op| (op.operation_id, op.group_id))
            .collect();
        assert_eq!(timeline, vec![(100, 200), (102, 201), (105, 200),]);

        assert!(store.get_operation(42)?.is_none());

        let group_operations = store.get_group_operations(200)?.unwrap();
        assert_eq!(group_operations.group_id, 200);

        let op_ids = group_operations
            .operations
            .iter()
            .map(|op| op.operation_id)
            .collect::<Vec<OperationId>>();

        assert_eq!(op_ids, vec![100, 105]);

        Ok(())
    }

    #[test]
    fn operations_iteration() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let mut store = MemoryPendingStore::new();

        store
            .put_operation(create_dummy_new_entry_op(&local_node, 105, 200))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(&local_node, 100, 200))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(&local_node, 102, 201))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(&local_node, 107, 202))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(&local_node, 110, 203))
            .unwrap();

        assert_eq!(store.operations_iter(..)?.count(), 5);

        Ok(())
    }

    #[test]
    fn operations_delete() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let mut store = MemoryPendingStore::new();

        store.put_operation(create_dummy_new_entry_op(&local_node, 101, 200))?;
        store.put_operation(create_dummy_new_entry_op(&local_node, 102, 200))?;
        store.put_operation(create_dummy_new_entry_op(&local_node, 103, 200))?;
        store.put_operation(create_dummy_new_entry_op(&local_node, 104, 200))?;

        // delete a single operation within a group
        store.delete_operation(103)?;
        assert!(store.get_operation(103)?.is_none());

        let operations = store.get_group_operations(200)?.unwrap();
        assert!(operations
            .operations
            .iter()
            .find(|op| op.operation_id == 103)
            .is_none());

        // delete a group operation
        store.delete_operation(200)?;
        assert!(store.get_operation(200)?.is_none());
        assert!(store.get_group_operations(200)?.is_none());

        Ok(())
    }

    #[test]
    fn operation_commit_status() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let mut store = MemoryPendingStore::new();

        store.put_operation(create_dummy_new_entry_op(&local_node, 101, 200))?;

        let operation = store.get_operation(101)?.unwrap();
        assert_eq!(CommitStatus::Unknown, operation.commit_status);

        store.update_operation_commit_status(101, CommitStatus::Committed(1, 2))?;
        let operation = store.get_operation(101)?.unwrap();
        assert_eq!(CommitStatus::Committed(1, 2), operation.commit_status);

        assert!(store
            .update_operation_commit_status(666, CommitStatus::Committed(2, 3))
            .is_err());

        Ok(())
    }
}
