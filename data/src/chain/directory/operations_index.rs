use std::cmp::Ordering;
use std::collections::btree_map::BTreeMap;
use std::io::{Read, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use byteorder::LittleEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};
use extindex::{Builder, Encodable, Reader};
use serde_derive::{Deserialize, Serialize};

use exocore_common::serialization::framed::TypedFrame;
use exocore_common::serialization::protos::data_chain_capnp::{block, pending_operation};
use exocore_common::serialization::protos::OperationID;
use exocore_common::simple_store::json_disk_store::JsonDiskStore;

use crate::chain::{Block, BlockOffset, BlockRef, ChainStore, Error};

use super::{DirectoryChainStoreConfig, DirectoryError};

pub struct OperationsIndex {
    config: DirectoryChainStoreConfig,
    directory: PathBuf,

    memory_offset_from: BlockOffset,
    memory_index: BTreeMap<OperationID, BlockOffset>,

    last_index_offset: BlockOffset,

    stored_indices: Vec<StoredIndex>,
}

impl OperationsIndex {
    pub fn create(
        config: DirectoryChainStoreConfig,
        directory_path: &Path,
    ) -> Result<OperationsIndex, Error> {
        let memory_index = BTreeMap::new();
        Ok(OperationsIndex {
            config,
            directory: directory_path.to_path_buf(),

            memory_offset_from: 0,
            memory_index,

            last_index_offset: 0,

            stored_indices: vec![],
        })
    }

    pub fn open(
        config: DirectoryChainStoreConfig,
        directory_path: &Path,
    ) -> Result<OperationsIndex, Error> {
        // TODO: Open meta store
        // TODO: Load indices

        // TODO: Update memory_from_offset & last_index_offset with last next offset

        let memory_index = BTreeMap::new();
        Ok(OperationsIndex {
            config,
            directory: directory_path.to_path_buf(),

            memory_offset_from: 0,
            memory_index,
            last_index_offset: 0,
            stored_indices: vec![],
        })
    }

    pub fn last_indexed_block_offset(&self) -> BlockOffset {
        self.memory_offset_from
    }

    pub fn index_blocks<I: Iterator<Item = B>, B: Block>(
        &mut self,
        iterator: I,
    ) -> Result<(), Error> {
        for block in iterator {
            if block.offset() >= self.memory_offset_from {
                println!("Initial indexing of block {}", block.offset());
                self.index_block(&block)?;
            }
        }

        Ok(())
    }

    pub fn index_block<B: Block>(&mut self, block: &B) -> Result<(), Error> {
        if self.last_index_offset > block.offset() {
            return Err(Error::Integrity(format!("Tried to index operations from a block that has an offset smaller than last indexed block: block={} > last={}", block.offset(), self.last_index_offset)));
        }

        let block_reader: block::Reader = block.block().get_typed_reader()?;

        // we add the operation that lead to the block proposal
        let block_propose_op_id = block_reader.get_proposed_operation_id();
        self.put_operation_block(block_propose_op_id, block.offset());

        // we add all operations that are in the block
        for operation in block.operations_iter()? {
            let operation_reader: pending_operation::Reader = operation.get_typed_reader()?;
            self.put_operation_block(operation_reader.get_operation_id(), block.offset());
        }

        // TODO: This may be problematic if an error occurred... We may have operations that should not have been there
        //       Perhaps we should just make sure we don't store them?
        self.last_index_offset = block.offset();

        self.maybe_store_to_disk()?;

        Ok(())
    }

    pub fn get_operation_block(
        &self,
        operation_id: OperationID,
    ) -> Result<Option<BlockOffset>, Error> {
        if let Some(block_offset) = self.memory_index.get(&operation_id) {
            return Ok(Some(*block_offset));
        }

        let needle = StoredIndexKey { operation_id };
        for index in self.stored_indices.iter() {
            let opt_entry = index
                .index_reader
                .find(&needle)
                .map_err(|err| DirectoryError::OperationsIndexRead(err))?;

            if let Some(entry) = opt_entry {
                return Ok(Some(entry.value().offset));
            }
        }

        Ok(None)
    }

    pub fn truncate_from_offset(&mut self) -> Result<(), Error> {
        // TODO: If it's somewhere in memory, just drop memory all along
        // TODO: If it's somewhere in stored index, we drop the whole index as we cannot mutate
        // TODO: Reset last operation

        unimplemented!()
    }

    fn put_operation_block(&mut self, operation_id: OperationID, block_offset: BlockOffset) {
        println!("STORING {} {}", operation_id, block_offset);
        self.memory_index.insert(operation_id, block_offset);
    }

    fn maybe_store_to_disk(&mut self) -> Result<(), Error> {
        if self.memory_index.len() > self.config.operations_index_max_memory_items {
            // TODO: STORE
        }

        Ok(())
    }
}

///
///
///
struct StoredIndex {
    range: Range<BlockOffset>,
    index_reader: Reader<StoredIndexKey, StoredIndexValue>,
}

///
///
///
#[derive(Serialize, Deserialize)]
struct Metadata {
    files: Vec<MetadataFile>,
}

#[derive(Serialize, Deserialize)]
struct MetadataFile {
    offset_from: BlockOffset,
    offset_to: BlockOffset,
    file_name: String,
}

///
///
///
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct StoredIndexKey {
    operation_id: OperationID,
}

impl Encodable<StoredIndexKey> for StoredIndexKey {
    fn encode_size(_item: &StoredIndexKey) -> Option<usize> {
        Some(8) // u64
    }

    fn encode(item: &StoredIndexKey, write: &mut dyn Write) -> Result<(), std::io::Error> {
        write.write_u64::<LittleEndian>(item.operation_id)
    }

    fn decode(data: &mut dyn Read, _size: usize) -> Result<StoredIndexKey, std::io::Error> {
        let operation_id = data.read_u64::<LittleEndian>()?;
        Ok(StoredIndexKey { operation_id })
    }
}

struct StoredIndexValue {
    offset: BlockOffset,
}

impl Encodable<StoredIndexValue> for StoredIndexValue {
    fn encode_size(item: &StoredIndexValue) -> Option<usize> {
        Some(8) // u64
    }

    fn encode(item: &StoredIndexValue, write: &mut dyn Write) -> Result<(), std::io::Error> {
        write.write_u64::<LittleEndian>(item.offset)
    }

    fn decode(data: &mut dyn Read, size: usize) -> Result<StoredIndexValue, std::io::Error> {
        let offset = data.read_u64::<LittleEndian>()?;
        Ok(StoredIndexValue { offset })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
