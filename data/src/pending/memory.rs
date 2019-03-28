use std::collections::{BTreeMap, HashMap};
use std::ops::RangeBounds;
use std::sync::Arc;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::security::hash::{Sha3Hasher, StreamHasher};
use exocore_common::serialization::framed;
use exocore_common::serialization::framed::{SignedFrame, TypedFrame};

use super::*;

///
/// In memory pending store
///
pub struct MemoryStore {
    operations_timeline: BTreeMap<OperationID, GroupID>,
    groups_operations: HashMap<GroupID, GroupOperations>,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore {
            operations_timeline: BTreeMap::new(),
            groups_operations: HashMap::new(),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        MemoryStore::new()
    }
}

impl Store for MemoryStore {
    fn put_operation(
        &mut self,
        operation: framed::OwnedTypedFrame<pending_operation::Owned>,
    ) -> Result<(), Error> {
        let operation_reader: pending_operation::Reader = operation.get_typed_reader()?;

        let group_id = operation_reader.get_group_id();
        let group_operations = self
            .groups_operations
            .entry(group_id)
            .or_insert_with(GroupOperations::new);

        let operation_id = operation_reader.get_operation_id();
        group_operations
            .operations
            .insert(operation_id, Arc::new(operation));

        self.operations_timeline.insert(operation_id, group_id);

        Ok(())
    }

    fn get_group_operations(
        &self,
        group_id: GroupID,
    ) -> Result<Option<StoredGroupOperations>, Error> {
        let operations = self.groups_operations.get(&group_id).map(|group_ops| {
            let operations = group_ops
                .operations
                .values()
                .map(|op| Arc::clone(op))
                .collect();

            StoredGroupOperations {
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

    fn operations_range_summary<R>(&self, range: R) -> Result<StoredRangeSummary, Error>
    where
        R: RangeBounds<OperationID>,
    {
        let mut hasher = Sha3Hasher::new_256();
        let mut count = 0;

        for (operation_id, pending_id) in self.operations_timeline.range(range) {
            if let Some(maybe_operation) = self.get_group_operation(pending_id, operation_id) {
                count += 1;

                match maybe_operation.signature_data() {
                    Some(sig_data) => hasher.consume(sig_data),
                    None => {
                        warn!(
                            "One pending operation didn't have any signature: pending_id={} op_id={}",
                            pending_id, operation_id
                        );
                    }
                }
            } else {
                warn!(
                    "Couldn't find one of the operation from timeline: pending_id={} op_id={}",
                    pending_id, operation_id
                );
            }
        }

        Ok(StoredRangeSummary {
            count,
            hash: hasher.into_multihash(),
        })
    }
}

impl MemoryStore {
    fn get_group_operation(
        &self,
        group_id: &GroupID,
        operation_id: &OperationID,
    ) -> Option<&Arc<framed::OwnedTypedFrame<pending_operation::Owned>>> {
        self.groups_operations
            .get(group_id)
            .and_then(|group_ops| group_ops.operations.get(operation_id))
    }
}

///
///
///
struct GroupOperations {
    operations: BTreeMap<OperationID, Arc<framed::OwnedTypedFrame<pending_operation::Owned>>>,
}

impl GroupOperations {
    fn new() -> GroupOperations {
        GroupOperations {
            operations: BTreeMap::new(),
        }
    }
}

///
///
///
struct OperationsIterator<'store> {
    store: &'store MemoryStore,
    ids_iterator: Box<dyn Iterator<Item = (OperationID, GroupID)> + 'store>,
}

impl<'store> Iterator for OperationsIterator<'store> {
    type Item = StoredOperation;

    fn next(&mut self) -> Option<StoredOperation> {
        let (operation_id, group_id) = self.ids_iterator.next()?;
        let operation = self.store.get_group_operation(&group_id, &operation_id)?;

        Some(StoredOperation {
            group_id,
            operation_id,
            operation: Arc::clone(operation),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::pending::tests::create_new_entry_op;

    #[test]
    fn put_and_retrieve_operation() {
        let mut store = MemoryStore::new();

        store.put_operation(create_new_entry_op(105, 200)).unwrap();
        store.put_operation(create_new_entry_op(100, 200)).unwrap();
        store.put_operation(create_new_entry_op(102, 201)).unwrap();

        let timeline: Vec<(OperationID, GroupID)> = store
            .operations_iter(..)
            .unwrap()
            .map(|op| (op.operation_id, op.group_id))
            .collect();
        assert_eq!(timeline, vec![(100, 200), (102, 201), (105, 200),]);

        let group_operations = store.get_group_operations(200).unwrap().unwrap();
        assert_eq!(group_operations.group_id, 200);

        let op_ids: Vec<OperationID> = group_operations
            .operations
            .iter()
            .map(|op| {
                let reader = op.get_typed_reader().unwrap();
                reader.get_operation_id()
            })
            .collect();

        assert_eq!(op_ids, vec![100, 105]);
    }

    #[test]
    fn operations_iteration() {
        let mut store = MemoryStore::new();

        store.put_operation(create_new_entry_op(105, 200)).unwrap();
        store.put_operation(create_new_entry_op(100, 200)).unwrap();
        store.put_operation(create_new_entry_op(102, 201)).unwrap();
        store.put_operation(create_new_entry_op(107, 202)).unwrap();
        store.put_operation(create_new_entry_op(110, 203)).unwrap();

        assert_eq!(store.operations_iter(..).unwrap().count(), 5);
    }

    #[test]
    fn operations_range_summary() {
        let mut store = MemoryStore::new();

        store.put_operation(create_new_entry_op(105, 200)).unwrap();
        store.put_operation(create_new_entry_op(100, 200)).unwrap();
        store.put_operation(create_new_entry_op(102, 201)).unwrap();
        store.put_operation(create_new_entry_op(107, 202)).unwrap();
        store.put_operation(create_new_entry_op(110, 203)).unwrap();

        let range1_summary = store.operations_range_summary(100..=102).unwrap();
        assert_eq!(range1_summary.count, 2);

        let range2_summary = store.operations_range_summary(103..).unwrap();
        assert_eq!(range2_summary.count, 3);

        assert_ne!(range1_summary.hash, range2_summary.hash);
    }
}
