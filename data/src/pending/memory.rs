use std::collections::{BTreeMap, HashMap};
use std::ops::RangeBounds;
use std::sync::Arc;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::serialization::framed;
use exocore_common::serialization::framed::TypedFrame;

use super::*;

///
/// In memory pending store
///
pub struct MemoryPendingStore {
    operations_timeline: BTreeMap<OperationID, GroupID>,
    groups_operations: HashMap<GroupID, GroupOperations>,
}

impl MemoryPendingStore {
    pub fn new() -> MemoryPendingStore {
        MemoryPendingStore {
            operations_timeline: BTreeMap::new(),
            groups_operations: HashMap::new(),
        }
    }
}

impl Default for MemoryPendingStore {
    fn default() -> Self {
        MemoryPendingStore::new()
    }
}

impl PendingStore for MemoryPendingStore {
    fn put_operation(
        &mut self,
        operation: framed::OwnedTypedFrame<pending_operation::Owned>,
    ) -> Result<bool, Error> {
        let operation_reader: pending_operation::Reader = operation.get_typed_reader()?;
        let operation_type = match operation_reader.get_operation().which()? {
            pending_operation::operation::Which::BlockSign(_) => OperationType::BlockSign,
            pending_operation::operation::Which::BlockPropose(_) => OperationType::BlockPropose,
            pending_operation::operation::Which::BlockRefuse(_) => OperationType::BlockRefuse,
            pending_operation::operation::Which::PendingIgnore(_) => OperationType::PendingIgnore,
            pending_operation::operation::Which::Entry(_) => OperationType::Entry,
        };

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
                frame: Arc::new(operation),
            },
        );

        let existed = self
            .operations_timeline
            .insert(operation_id, group_id)
            .is_some();
        Ok(existed)
    }

    fn get_operation(&self, operation_id: OperationID) -> Result<Option<StoredOperation>, Error> {
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
                frame: Arc::clone(&op.frame),
            });

        Ok(operation)
    }

    fn get_group_operations(
        &self,
        group_id: GroupID,
    ) -> Result<Option<StoredOperationsGroup>, Error> {
        let operations = self.groups_operations.get(&group_id).map(|group_ops| {
            let operations = group_ops
                .operations
                .values()
                .map(|op| StoredOperation {
                    group_id,
                    operation_id: op.operation_id,
                    operation_type: op.operation_type,
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
        R: RangeBounds<OperationID>,
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
}

impl MemoryPendingStore {
    fn get_group_operation(
        &self,
        group_id: GroupID,
        operation_id: OperationID,
    ) -> Option<&GroupOperation> {
        self.groups_operations
            .get(&group_id)
            .and_then(|group_ops| group_ops.operations.get(&operation_id))
    }
}

///
///
///
struct GroupOperations {
    operations: BTreeMap<OperationID, GroupOperation>,
}

impl GroupOperations {
    fn new() -> GroupOperations {
        GroupOperations {
            operations: BTreeMap::new(),
        }
    }
}

struct GroupOperation {
    operation_id: OperationID,
    operation_type: OperationType,
    frame: Arc<framed::OwnedTypedFrame<pending_operation::Owned>>,
}

///
///
///
struct OperationsIterator<'store> {
    store: &'store MemoryPendingStore,
    ids_iterator: Box<dyn Iterator<Item = (OperationID, GroupID)> + 'store>,
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
            frame: Arc::clone(&group_operation.frame),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::engine::testing::create_dummy_new_entry_op;

    use super::*;

    #[test]
    fn put_and_retrieve_operation() -> Result<(), failure::Error> {
        let mut store = MemoryPendingStore::new();

        store.put_operation(create_dummy_new_entry_op(105, 200))?;
        store.put_operation(create_dummy_new_entry_op(100, 200))?;
        store.put_operation(create_dummy_new_entry_op(102, 201))?;

        let timeline: Vec<(OperationID, GroupID)> = store
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
            .map(|op| {
                let reader = op.frame.get_typed_reader()?;
                Ok(reader.get_operation_id())
            })
            .collect::<Result<Vec<OperationID>, failure::Error>>()?;

        assert_eq!(op_ids, vec![100, 105]);

        Ok(())
    }

    #[test]
    fn operations_iteration() -> Result<(), failure::Error> {
        let mut store = MemoryPendingStore::new();

        store
            .put_operation(create_dummy_new_entry_op(105, 200))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(100, 200))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(102, 201))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(107, 202))
            .unwrap();
        store
            .put_operation(create_dummy_new_entry_op(110, 203))
            .unwrap();

        assert_eq!(store.operations_iter(..)?.count(), 5);

        Ok(())
    }
}
