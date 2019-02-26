use std::collections::{BTreeMap, HashMap};
use std::ops::RangeBounds;
use std::sync::Arc;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::security::hash::{Sha3Hasher, StreamHasher};
use exocore_common::serialization::framed;
use exocore_common::serialization::framed::{SignedFrame, TypedFrame};
use exocore_common::serialization::protos::{OperationID, PendingID};

use super::*;

///
/// In memory pending store
///
pub struct MemoryStore {
    operations_timeline: BTreeMap<OperationID, PendingID>,
    entries_operations: HashMap<PendingID, EntryOperations>,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore {
            operations_timeline: BTreeMap::new(),
            entries_operations: HashMap::new(),
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

        let pending_id = operation_reader.get_pending_id();
        let pending_entry_operations = self
            .entries_operations
            .entry(pending_id)
            .or_insert_with(EntryOperations::new);

        let operation_id = operation_reader.get_operation_id();
        pending_entry_operations
            .operations
            .insert(operation_id, Arc::new(operation));

        self.operations_timeline.insert(operation_id, pending_id);

        Ok(())
    }

    fn get_entry_operations(
        &self,
        entry_id: PendingID,
    ) -> Result<Option<StoredEntryOperations>, Error> {
        let operations = self.entries_operations.get(&entry_id).map(|entry_ops| {
            let operations = entry_ops
                .operations
                .values()
                .map(|op| Arc::clone(op))
                .collect();

            StoredEntryOperations {
                entry_id,
                operations,
            }
        });

        Ok(operations)
    }

    fn operations_iter<'store, R>(
        &'store self,
        range: R,
    ) -> Result<Box<dyn Iterator<Item = StoredOperation> + 'store>, Error>
    where
        R: RangeBounds<OperationID>,
    {
        let iter = self
            .operations_timeline
            .range(range)
            .map(|(operation_id, entry_id)| StoredOperation {
                entry_id: *entry_id,
                operation_id: *operation_id,
            });

        Ok(Box::new(iter))
    }

    fn operations_range_summary<R>(&self, range: R) -> Result<StoredRangeSummary, Error>
    where
        R: RangeBounds<OperationID>,
    {
        let mut hasher = Sha3Hasher::new_256();
        let mut count = 0;

        for (operation_id, entry_id) in self.operations_timeline.range(range) {
            if let Some(maybe_operation) = self
                .entries_operations
                .get(entry_id)
                .and_then(|entry_ops| entry_ops.operations.get(operation_id))
            {
                count += 1;

                match maybe_operation.signature_data() {
                    Some(sig_data) => hasher.consume(sig_data),
                    None => {
                        warn!(
                            "One pending operation didn't have any signature: entry_id={} op_id={}",
                            entry_id, operation_id
                        );
                    }
                }
            } else {
                warn!(
                    "Couldn't find one of the operation from timeline: entry_id={} op_id={}",
                    entry_id, operation_id
                );
            }
        }

        Ok(StoredRangeSummary {
            count,
            hash: hasher.into_multihash(),
        })
    }
}

struct EntryOperations {
    operations: BTreeMap<OperationID, Arc<framed::OwnedTypedFrame<pending_operation::Owned>>>,
}

impl EntryOperations {
    fn new() -> EntryOperations {
        EntryOperations {
            operations: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use exocore_common::serialization::framed::{FrameBuilder, MultihashFrameSigner};

    use super::*;

    #[test]
    fn put_and_retrieve_operation() {
        let mut store = MemoryStore::new();

        store.put_operation(create_new_entry_op(105, 200)).unwrap();
        store.put_operation(create_new_entry_op(100, 200)).unwrap();
        store.put_operation(create_new_entry_op(102, 201)).unwrap();

        let timeline: Vec<(OperationID, PendingID)> = store
            .operations_iter(..)
            .unwrap()
            .map(|op| (op.operation_id, op.entry_id))
            .collect();
        assert_eq!(timeline, vec![(100, 200), (102, 201), (105, 200),]);

        let entry_operations = store.get_entry_operations(200).unwrap().unwrap();
        assert_eq!(entry_operations.entry_id, 200);

        let op_ids: Vec<OperationID> = entry_operations
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

    fn create_new_entry_op(
        operation_id: OperationID,
        pending_id: PendingID,
    ) -> framed::OwnedTypedFrame<pending_operation::Owned> {
        let mut msg_builder = FrameBuilder::<pending_operation::Owned>::new();

        {
            let mut op_builder: pending_operation::Builder = msg_builder.get_builder_typed();
            op_builder.set_pending_id(pending_id);
            op_builder.set_operation_id(operation_id);
            let inner_op_builder = op_builder.init_operation();

            let new_entry_op_builder = inner_op_builder.init_entry_new();
            let mut entry_header_builder = new_entry_op_builder.init_entry_header();
            entry_header_builder.set_id(pending_id);
        }

        let frame_signer = MultihashFrameSigner::new_sha3256();
        msg_builder.as_owned_framed(frame_signer).unwrap()
    }
}
