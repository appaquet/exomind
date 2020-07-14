use std::collections::btree_map::BTreeMap;
use std::io::{Read, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use byteorder::LittleEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};
use extindex::{Builder, Encodable, Reader};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use crate::operation::OperationId;
use exocore_core::protos::generated::data_chain_capnp::block_header;
use exocore_core::simple_store::json_disk_store::JsonDiskStore;
use exocore_core::simple_store::SimpleStore;

use crate::block::{Block, BlockOffset};
use crate::chain::Error;

use super::{DirectoryChainStoreConfig, DirectoryError};
use std::sync::Arc;

/// Operation ID to Block offset index. This is used to retrieve the block
/// offset in which a given operation ID has been stored.
///
/// This index has a in-memory buffer, and is flushed to disk into `extindex`
/// immutable index files.
///
/// The in-memory portion of it may be lost if it hadn't been flush. The chain
/// directory make sure that the chain is properly indexed when its initializing
/// using the `next_expected_offset` value.
///
/// The index maintains the list of persisted index in a "Metadata" file.
pub struct OperationsIndex {
    config: DirectoryChainStoreConfig,
    directory: PathBuf,

    metadata_store: JsonDiskStore<Metadata>,

    memory_offset_from: BlockOffset,
    memory_index: BTreeMap<OperationId, BlockOffset>,

    next_expected_offset: BlockOffset,

    stored_indices: Vec<StoredIndex>,
}

impl OperationsIndex {
    /// Creates a new operation index that will be stored in given directory.
    pub fn create(
        config: DirectoryChainStoreConfig,
        directory_path: &Path,
    ) -> Result<OperationsIndex, Error> {
        let metadata_path = Metadata::file_path(directory_path);
        let metadata_store = JsonDiskStore::<Metadata>::new(&metadata_path).map_err(|err| {
            Error::new_io(
                err,
                format!(
                    "Error creating operations index metadata file {:?}",
                    metadata_path
                ),
            )
        })?;

        let operations_index = OperationsIndex {
            config,
            directory: directory_path.to_path_buf(),

            metadata_store,

            memory_offset_from: 0,
            memory_index: BTreeMap::new(),

            next_expected_offset: 0,

            stored_indices: vec![],
        };

        // we write even if it's empty because `open` expects it to exist
        operations_index.write_metadata()?;

        Ok(operations_index)
    }

    /// Open an existing operation index stored in given directory.
    pub fn open(
        config: DirectoryChainStoreConfig,
        directory_path: &Path,
    ) -> Result<OperationsIndex, Error> {
        let metadata_path = Metadata::file_path(directory_path);
        let metadata_store = JsonDiskStore::<Metadata>::new(&metadata_path).map_err(|err| {
            Error::new_io(
                err,
                format!(
                    "Error creating operations index metadata file {:?}",
                    metadata_path
                ),
            )
        })?;

        let metadata = metadata_store
            .read()
            .map_err(|err| {
                Error::new_io(
                    err,
                    format!(
                        "Error reading operations index metadata file {:?}",
                        metadata_path
                    ),
                )
            })?
            .ok_or_else(|| {
                Error::UnexpectedState(String::from("Operations index metadata file didn't exist"))
            })?;

        let mut stored_indices = Vec::new();
        for index_file_metadata in metadata.files.iter() {
            let index_file_path = directory_path.join(&index_file_metadata.file_name);
            let index_reader = Reader::open(index_file_path)
                .map_err(|err| DirectoryError::OperationsIndexRead(Arc::new(err)))?;

            stored_indices.push(StoredIndex {
                range: index_file_metadata.offset_from..index_file_metadata.offset_to,
                index_reader,
            });
        }

        // the next expected offset is the upper bound (excluded) of the last segment we
        // indexed
        let next_expected_offset = stored_indices.last().map_or(0, |index| index.range.end);

        // we have nothing in memory, so memory index is from next expected offset
        let memory_offset_from = next_expected_offset;
        let memory_index = BTreeMap::new();

        Ok(OperationsIndex {
            config,
            directory: directory_path.to_path_buf(),

            metadata_store,

            memory_offset_from,
            memory_index,
            next_expected_offset,
            stored_indices,
        })
    }

    /// Returns the offset that we expect the next block to have. This can be
    /// used to know which operations are missing and need to be re-indexed.
    pub fn next_expected_block_offset(&self) -> BlockOffset {
        self.next_expected_offset
    }

    /// Indexes an iterator of blocks. There is no guarantee that they will be
    /// actually stored to disk if they can still fit in the in-memory
    /// index.
    pub fn index_blocks<I: Iterator<Item = B>, B: Block>(
        &mut self,
        iterator: I,
    ) -> Result<(), Error> {
        for block in iterator {
            if block.offset() >= self.memory_offset_from {
                self.index_block(&block)?;
            }
        }

        Ok(())
    }

    /// Indexes a block. There is no guarantee that it will be actually stored
    /// if it can still fit in the in-memory index.
    pub fn index_block<B: Block>(&mut self, block: &B) -> Result<(), Error> {
        if self.next_expected_offset != block.offset() {
            return Err(Error::Integrity(format!(
                "Tried to index operations from a block with unexpected offset: block={} != expected={}",
                block.offset(),
                self.next_expected_offset
            )));
        }

        let block_header_reader: block_header::Reader = block
            .header()
            .get_reader()
            .map_err(|err| Error::Block(err.into()))?;

        // we add the operation that lead to the block proposal
        let block_propose_op_id = block_header_reader.get_proposed_operation_id();
        self.put_operation_block(block_propose_op_id, block.offset());

        // we add all operations that are in the block
        for operation in block.operations_iter()? {
            let operation_reader = operation.get_reader()?;
            self.put_operation_block(operation_reader.get_operation_id(), block.offset());
        }

        self.next_expected_offset = block.next_offset();

        self.maybe_flush_to_disk()?;

        Ok(())
    }

    /// Retrieves the block offset in which a given operation was stored.
    pub fn get_operation_block(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<BlockOffset>, Error> {
        if let Some(block_offset) = self.memory_index.get(&operation_id) {
            return Ok(Some(*block_offset));
        }

        let needle = StoredIndexKey { operation_id };
        for index in self.stored_indices.iter() {
            let opt_entry = index
                .index_reader
                .find(&needle)
                .map_err(|err| DirectoryError::OperationsIndexRead(Arc::new(err)))?;

            if let Some(entry) = opt_entry {
                return Ok(Some(entry.value().offset));
            }
        }

        Ok(None)
    }

    /// Truncates the index from the given offset. Because of the nature of the
    /// immutable underlying indices, we cannot delete from the exact
    /// offset.

    /// Therefor, we expect `index_blocks` to be called right after to index any
    /// missing blocks that we over-truncated. The
    /// `next_expected_block_offset` method can be used to know from which
    /// offset we need to re-index from.
    pub fn truncate_from_offset(&mut self, from_offset: BlockOffset) -> Result<(), Error> {
        if from_offset >= self.memory_offset_from {
            self.memory_index.clear();
            self.next_expected_offset = self.memory_offset_from;
        } else {
            let mut previous_indices = Vec::new();
            std::mem::swap(&mut self.stored_indices, &mut previous_indices);

            for index in previous_indices {
                if index.range.end >= from_offset {
                    self.next_expected_offset = self.next_expected_offset.min(index.range.start);

                    let index_path = StoredIndex::file_path(&self.directory, &index.range);
                    let _ = std::fs::remove_file(index_path);
                } else {
                    self.stored_indices.push(index);
                }
            }

            self.next_expected_offset = self
                .stored_indices
                .last()
                .map_or(0, |index| index.range.end);
            self.memory_offset_from = self.next_expected_offset;
        }

        self.write_metadata()?;

        Ok(())
    }

    /// Inserts a single operation in the in-memory index
    fn put_operation_block(&mut self, operation_id: OperationId, block_offset: BlockOffset) {
        self.memory_index.insert(operation_id, block_offset);
    }

    /// Checks the size of the in-memory index and flush it to disk if it
    /// exceeds configured maximum.
    fn maybe_flush_to_disk(&mut self) -> Result<(), Error> {
        if self.memory_index.len() > self.config.operations_index_max_memory_items {
            debug!(
                "Storing in-memory index of operations to disk ({} items)",
                self.memory_index.len()
            );

            let from_offset = self.memory_offset_from;
            let to_offset = self.next_expected_offset;
            let range = from_offset..to_offset;
            let index_file = StoredIndex::file_path(&self.directory, &range);

            // build the index from in-memory index, which is already sorted because it's in
            // a tree
            let ops_count = self.memory_index.len() as u64;
            let ops_iter = self.memory_index.iter().map(|(operation_id, offset)| {
                let key = StoredIndexKey {
                    operation_id: *operation_id,
                };
                let value = StoredIndexValue { offset: *offset };

                extindex::Entry::new(key, value)
            });
            let index_builder =
                Builder::<StoredIndexKey, StoredIndexValue>::new(index_file.clone());
            index_builder
                .build_from_sorted(ops_iter, ops_count)
                .map_err(|err| DirectoryError::OperationsIndexBuild(Arc::new(err)))?;

            // open the index we just created
            let index_reader = Reader::open(index_file)
                .map_err(|err| DirectoryError::OperationsIndexRead(Arc::new(err)))?;
            let stored_index = StoredIndex {
                range,
                index_reader,
            };
            self.stored_indices.push(stored_index);

            self.write_metadata()?;

            // memory index now starts at next expected offset
            self.memory_offset_from = self.next_expected_offset;
            self.memory_index.clear();
        }

        Ok(())
    }

    /// Writes metadata to disk
    fn write_metadata(&self) -> Result<(), Error> {
        let files = self
            .stored_indices
            .iter()
            .map(|index| {
                let file_name = StoredIndex::file_name(&index.range);
                MetadataIndexFile {
                    offset_from: index.range.start,
                    offset_to: index.range.end,
                    file_name,
                }
            })
            .collect_vec();
        let metadata = Metadata { files };

        self.metadata_store
            .write(&metadata)
            .map_err(|err| Error::new_io(err, "Error storing into operations index metadata file"))
    }
}

/// Represents an immutable on-disk index for a given range of offsets.
struct StoredIndex {
    range: Range<BlockOffset>,
    index_reader: Reader<StoredIndexKey, StoredIndexValue>,
}

impl StoredIndex {
    fn file_path(directory: &Path, range: &Range<BlockOffset>) -> PathBuf {
        directory.join(Self::file_name(range))
    }

    fn file_name(range: &Range<BlockOffset>) -> String {
        format!("opsidx_{}.bin", range.start)
    }
}

/// Metadata stored on disk to describe segments of the block that are indexed.
#[derive(Serialize, Deserialize)]
struct Metadata {
    files: Vec<MetadataIndexFile>,
}

impl Metadata {
    fn file_path(directory: &Path) -> PathBuf {
        directory.join("ops_idx.json")
    }
}

#[derive(Serialize, Deserialize)]
struct MetadataIndexFile {
    offset_from: BlockOffset,
    offset_to: BlockOffset,
    file_name: String,
}

/// Wraps the key stored in the on-disk index.
/// This is needed for encoding / decoding.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct StoredIndexKey {
    operation_id: OperationId,
}

impl Encodable<StoredIndexKey> for StoredIndexKey {
    fn encode_size(_item: &StoredIndexKey) -> Option<usize> {
        Some(8) // u64
    }

    fn encode<W: Write>(item: &StoredIndexKey, write: &mut W) -> Result<(), std::io::Error> {
        write.write_u64::<LittleEndian>(item.operation_id)
    }

    fn decode<R: Read>(data: &mut R, _size: usize) -> Result<StoredIndexKey, std::io::Error> {
        let operation_id = data.read_u64::<LittleEndian>()?;
        Ok(StoredIndexKey { operation_id })
    }
}

/// Wraps the value stored in the on-disk index.
/// This is needed for encoding / decoding.
struct StoredIndexValue {
    offset: BlockOffset,
}

impl Encodable<StoredIndexValue> for StoredIndexValue {
    fn encode_size(_item: &StoredIndexValue) -> Option<usize> {
        Some(8) // u64
    }

    fn encode<W: Write>(item: &StoredIndexValue, write: &mut W) -> Result<(), std::io::Error> {
        write.write_u64::<LittleEndian>(item.offset)
    }

    fn decode<R: Read>(data: &mut R, _size: usize) -> Result<StoredIndexValue, std::io::Error> {
        let offset = data.read_u64::<LittleEndian>()?;
        Ok(StoredIndexValue { offset })
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::directory::tests::create_block;

    use super::*;
    use exocore_core::cell::FullCell;
    use exocore_core::cell::LocalNode;

    #[test]
    fn create_from_iterator() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let config = DirectoryChainStoreConfig {
            operations_index_max_memory_items: 100,
            ..DirectoryChainStoreConfig::default()
        };

        let mut index = OperationsIndex::create(config, dir.path())?;
        let generated_ops = generate_index_blocks(&cell, &mut index, 0, 1000)?;

        // 19 because there is 2 ops per block (block itself + op inside)
        assert_eq!(19, index.stored_indices.len());

        // make sure we can find all stored operations
        for (op, offset) in &generated_ops {
            assert_eq!(Some(*offset), index.get_operation_block(*op)?);
        }

        // try to find a missing operation
        assert_eq!(None, index.get_operation_block(435_874_985)?);

        Ok(())
    }

    #[test]
    fn open_existing() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let config = DirectoryChainStoreConfig {
            operations_index_max_memory_items: 100,
            ..DirectoryChainStoreConfig::default()
        };

        let (memory_offset_from, generated_ops) = {
            let mut index = OperationsIndex::create(config, dir.path())?;
            let generated_ops = generate_index_blocks(&cell, &mut index, 0, 1000)?;
            (index.memory_offset_from, generated_ops)
        };

        let mut index = OperationsIndex::open(config, dir.path())?;

        // all data that was previously stored in memory is lost
        assert_eq!(memory_offset_from, index.memory_offset_from);
        assert_eq!(memory_offset_from, index.next_expected_block_offset());

        assert_eq!(19, index.stored_indices.len());

        // make sure we can find all stored operations
        for (op, offset) in &generated_ops {
            if *offset < memory_offset_from {
                assert_eq!(Some(*offset), index.get_operation_block(*op)?);
            }
        }

        // we append some more operations, we expect all of them to be there
        let new_ops = generate_index_blocks(&cell, &mut index, memory_offset_from, 200)?;
        for (op, offset) in &new_ops {
            assert_eq!(Some(*offset), index.get_operation_block(*op)?);
        }
        assert_eq!(22, index.stored_indices.len());

        Ok(())
    }

    #[test]
    fn truncate_from_offset_memory() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let config = DirectoryChainStoreConfig {
            operations_index_max_memory_items: 100,
            ..DirectoryChainStoreConfig::default()
        };

        let mut index = OperationsIndex::create(config, dir.path())?;
        generate_index_blocks(&cell, &mut index, 0, 1000)?;

        let files_count_before = index.stored_indices.len();
        index.truncate_from_offset(index.memory_offset_from)?;
        assert_eq!(index.memory_offset_from, index.next_expected_offset);
        assert_eq!(index.stored_indices.len(), files_count_before);

        Ok(())
    }

    #[test]
    fn truncate_from_offset_disk() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let config = DirectoryChainStoreConfig {
            operations_index_max_memory_items: 100,
            ..DirectoryChainStoreConfig::default()
        };

        let next_expected_offset = {
            let mut index = OperationsIndex::create(config, dir.path())?;
            let generated_ops = generate_index_blocks(&cell, &mut index, 0, 1000)?;

            let operation_ids = generated_ops.keys().collect_vec();
            let middle_block_offset = generated_ops[operation_ids[operation_ids.len() / 2]];

            let files_count_before = index.stored_indices.len();
            index.truncate_from_offset(middle_block_offset)?;

            assert!(index.next_expected_offset <= middle_block_offset);
            assert_eq!(index.memory_offset_from, index.next_expected_offset);
            assert!(index.stored_indices.len() <= files_count_before / 2);

            index.next_expected_offset
        };

        {
            let index = OperationsIndex::open(config, dir.path())?;
            assert_eq!(next_expected_offset, index.next_expected_offset);
        }

        Ok(())
    }

    fn generate_index_blocks(
        full_cell: &FullCell,
        index: &mut OperationsIndex,
        from_offset: BlockOffset,
        count: usize,
    ) -> Result<BTreeMap<OperationId, BlockOffset>, Error> {
        let mut generated_ops = BTreeMap::new();

        let mut next_offset = from_offset;
        let blocks_iter = (0..count).map(|_i| {
            // create_block will use offset as proposed operation id and will create 1 op
            // inside
            let block = create_block(full_cell, next_offset);
            generated_ops.insert(next_offset, next_offset);
            generated_ops.insert(next_offset + 1, next_offset);

            next_offset = block.next_offset();
            block
        });

        index.index_blocks(blocks_iter)?;

        Ok(generated_ops)
    }
}
