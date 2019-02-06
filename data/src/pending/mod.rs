use std::collections::{BTreeMap, HashMap};
use std::ops::RangeBounds;
use std::sync::Arc;
use std::vec::Vec;

use failure::ResultExt;

use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::serialization::msg;
use exocore_common::serialization::msg::FramedTypedMessage;
use exocore_common::serialization::protos::{EntryID, OperationID};

pub trait Store {
    fn put_operation(
        &mut self,
        operation: msg::FramedOwnedTypedMessage<pending_operation::Owned>,
    ) -> Result<(), Error>;

    fn get_entry_operations(
        &self,
        entry_id: EntryID,
    ) -> Result<Option<StoredEntryOperations>, Error>;

    fn operations_iter<'store, R>(
        &'store self,
        range: R,
    ) -> Result<Box<Iterator<Item = StoredOperation> + 'store>, Error>
    where
        R: RangeBounds<OperationID>;
}

pub type TimelineIterator<'store> = Box<dyn Iterator<Item = StoredOperation> + 'store>;

pub struct StoredOperation {
    pub entry_id: EntryID,
    pub operation_id: OperationID,
}

pub struct StoredEntryOperations {
    pub entry_id: EntryID,
    pub operations: Vec<Arc<msg::FramedOwnedTypedMessage<pending_operation::Owned>>>,
}

///
///
///
pub struct MemoryStore {
    operations_timeline: BTreeMap<OperationID, EntryID>,
    operations: HashMap<EntryID, EntryOperations>,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore {
            operations_timeline: BTreeMap::new(),
            operations: HashMap::new(),
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
        operation: msg::FramedOwnedTypedMessage<pending_operation::Owned>,
    ) -> Result<(), Error> {
        let operation_reader: pending_operation::Reader = operation.get_typed_reader()?;

        let entry_id = operation_reader.get_entry_id();
        let entry_operations = self
            .operations
            .entry(entry_id)
            .or_insert_with(EntryOperations::new);

        let operation_id = operation_reader.get_id();
        entry_operations
            .operations
            .insert(operation_id, Arc::new(operation));

        self.operations_timeline.insert(operation_id, entry_id);

        Ok(())
    }

    fn get_entry_operations(
        &self,
        entry_id: EntryID,
    ) -> Result<Option<StoredEntryOperations>, Error> {
        let operations = self.operations.get(&entry_id).map(|entry_ops| {
            let operations = entry_ops.operations.values().map(|op| Arc::clone(op)).collect();

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
    ) -> Result<Box<Iterator<Item = StoredOperation> + 'store>, Error>
    where
        R: RangeBounds<OperationID>,
    {
        let iter = self
            .operations_timeline
            .range(range)
            .flat_map(|(operation_id, entity_id)| {
                dbg!((entity_id, operation_id));
                Some(StoredOperation {
                    entry_id: *entity_id,
                    operation_id: *operation_id,
                })
            });

        Ok(Box::new(iter))
    }
}

struct EntryOperations {
    operations: BTreeMap<OperationID, Arc<msg::FramedOwnedTypedMessage<pending_operation::Owned>>>,
}

impl EntryOperations {
    fn new() -> EntryOperations {
        EntryOperations {
            operations: BTreeMap::new(),
        }
    }
}

///
///
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error in message serialization")]
    Serialization(#[fail(cause)] msg::Error),
}

impl From<msg::Error> for Error {
    fn from(err: msg::Error) -> Self {
        Error::Serialization(err)
    }
}

#[cfg(test)]
mod test {
    use failure::Fail;

    use exocore_common::serialization::msg::MessageBuilder;

    use super::*;

    #[test]
    fn put_and_retrieve_operation() {
        let mut store = MemoryStore::new();

        store.put_operation(create_new_entry_op(105, 200)).unwrap();
        store.put_operation(create_new_entry_op(100, 200)).unwrap();
        store.put_operation(create_new_entry_op(102, 201)).unwrap();

        let timeline: Vec<(OperationID, EntryID)> = store
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
                reader.get_id()
            })
            .collect();

        assert_eq!(op_ids, vec![100, 105]);
    }

    fn create_new_entry_op(
        operation_id: OperationID,
        entry_id: EntryID,
    ) -> msg::FramedOwnedTypedMessage<pending_operation::Owned> {
        let mut msg_builder = MessageBuilder::<pending_operation::Owned>::new();
        {
            let mut op_builder: pending_operation::Builder = msg_builder.get_builder_typed();
            op_builder.set_id(operation_id);
            op_builder.set_entry_id(entry_id);
            let mut inner_op_builder = op_builder.init_operation();

            let mut new_entry_op_builder = inner_op_builder.init_new_entry();
            let mut entry_builder = new_entry_op_builder.init_entry();
            let mut entry_header_builder = entry_builder.init_header();
            entry_header_builder.set_id(entry_id);
        }
        msg_builder.as_owned_framed().unwrap()
    }
}
