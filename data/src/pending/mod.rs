use exocore_common::data_chain_capnp::pending_operation;
use exocore_common::serialization::msg;
use exocore_common::serialization::protos::{EntryID, OperationID, OperationTime};

use failure::ResultExt;

// TODO: WAL stored on disk, rotated
// TODO: Or in memory, with spilling to disk?

use std::collections::{BTreeMap, HashMap};
use std::vec::Vec;

pub trait Store {
    fn put_operation<O>(&mut self, operation: &O) -> Result<(), Error>
    where
        O: msg::FramedTypedMessage<pending_operation::Owned>;
}

pub struct MemoryStore {
    operations_timeline: BTreeMap<OperationTime, (EntryID, OperationID)>,
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
    fn put_operation<O>(&mut self, operation: &O) -> Result<(), Error>
    where
        O: msg::FramedTypedMessage<pending_operation::Owned>,
    {
        let operation_reader: pending_operation::Reader = operation.get_typed_reader()?;

        let entry_uid = operation_reader.get_entry_uid();
        let entry_operations = self
            .operations
            .entry(entry_uid)
            .or_insert_with(EntryOperations::new);

        let operation_uid = operation_reader.get_operation_uid();
        entry_operations
            .operations
            .insert(operation_uid, operation.to_owned());

        // TODO:
        Ok(())
    }
}

struct EntryOperations {
    operations: HashMap<OperationID, msg::FramedOwnedTypedMessage<pending_operation::Owned>>,
}

impl EntryOperations {
    fn new() -> EntryOperations {
        EntryOperations {
            operations: HashMap::new(),
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
    use exocore_common::serialization::msg::MessageBuilder;

    use super::*;
    use failure::Fail;

    #[test]
    fn test() {
        let mut store = MemoryStore::new();

        let mut msg_builder = MessageBuilder::<pending_operation::Owned>::new();
        {
            let mut op_builder: pending_operation::Builder = msg_builder.get_builder_typed();
            op_builder.set_time(2133);
            op_builder.set_entry_uid(10);
            let mut inner_op_builder = op_builder.init_operation();

            let mut new_entry_op_builder = inner_op_builder.init_new_entry();
            let mut entry_builder = new_entry_op_builder.init_entry();
            let mut entry_header_builder = entry_builder.init_header();
            entry_header_builder.set_uid(10);
        }

        let op = msg_builder.as_owned_framed().unwrap();
        store.put_operation(&op).unwrap();
    }
}
