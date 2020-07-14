use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::operation::OperationId;
use segment::DirectorySegment;

use crate::block::{Block, BlockOffset, BlockRef};
use crate::chain::{ChainStore, Error, Segment, StoredBlockIterator};

mod operations_index;
mod segment;

use operations_index::OperationsIndex;

/// Directory based chain persistence. The chain is split in segments with
/// configurable maximum size. This maximum size allows using mmap on 32bit
/// systems by preventing segments from growing over 4gb.
pub struct DirectoryChainStore {
    config: DirectoryChainStoreConfig,
    directory: PathBuf,
    segments: Vec<DirectorySegment>,

    // TODO: Optional because index needs the Store to be initialized to iterate
    // TODO: To be solved in https://github.com/appaquet/exocore/issues/34
    operations_index: Option<OperationsIndex>,
}

impl DirectoryChainStore {
    pub fn create_or_open(
        config: DirectoryChainStoreConfig,
        directory_path: &Path,
    ) -> Result<DirectoryChainStore, Error> {
        let paths = std::fs::read_dir(directory_path).map_err(|err| {
            Error::new_io(
                err,
                format!(
                    "Error listing directory {}",
                    directory_path.to_string_lossy(),
                ),
            )
        })?;

        if paths.count() == 0 {
            Self::create(config, directory_path)
        } else {
            Self::open(config, directory_path)
        }
    }

    pub fn create(
        config: DirectoryChainStoreConfig,
        directory_path: &Path,
    ) -> Result<DirectoryChainStore, Error> {
        if !directory_path.exists() {
            return Err(Error::UnexpectedState(format!(
                "Tried to create directory at {:?}, but it didn't exist",
                directory_path
            )));
        }

        let paths = std::fs::read_dir(directory_path).map_err(|err| {
            Error::new_io(
                err,
                format!(
                    "Error listing directory {}",
                    directory_path.to_string_lossy(),
                ),
            )
        })?;

        if paths.count() > 0 {
            return Err(Error::UnexpectedState(format!(
                "Tried to create directory at {:?}, but it's not empty",
                directory_path
            )));
        }

        let operations_index = OperationsIndex::create(config, directory_path)?;

        Ok(DirectoryChainStore {
            config,
            directory: directory_path.to_path_buf(),
            segments: Vec::new(),
            operations_index: Some(operations_index),
        })
    }

    pub fn open(
        config: DirectoryChainStoreConfig,
        directory_path: &Path,
    ) -> Result<DirectoryChainStore, Error> {
        if !directory_path.exists() {
            return Err(Error::UnexpectedState(format!(
                "Tried to open directory at {:?}, but it didn't exist",
                directory_path
            )));
        }

        let mut segments = Vec::new();
        let paths = std::fs::read_dir(directory_path).map_err(|err| {
            Error::new_io(
                err,
                format!(
                    "Error listing directory {}",
                    directory_path.to_string_lossy(),
                ),
            )
        })?;
        for path in paths {
            let path = path.map_err(|err| Error::new_io(err, "Error getting directory entry"))?;

            if DirectorySegment::is_segment_file(&path.path()) {
                let segment = DirectorySegment::open(config, &path.path())?;
                segments.push(segment);
            }
        }
        segments.sort_by(|a, b| a.first_block_offset().cmp(&b.first_block_offset()));

        let mut store = DirectoryChainStore {
            config,
            directory: directory_path.to_path_buf(),
            segments,
            operations_index: None,
        };

        let operations_index = {
            let mut operations_index = OperationsIndex::open(config, directory_path)?;
            let next_index_offset = operations_index.next_expected_block_offset();
            let blocks_to_index = store.blocks_iter(next_index_offset)?;
            operations_index.index_blocks(blocks_to_index)?;
            operations_index
        };
        store.operations_index = Some(operations_index);

        Ok(store)
    }

    fn get_segment_index_for_block_offset(&self, block_offset: BlockOffset) -> Option<usize> {
        self.segments
            .binary_search_by(|seg| {
                if block_offset >= seg.first_block_offset()
                    && block_offset < seg.next_block_offset()
                {
                    Ordering::Equal
                } else if block_offset < seg.first_block_offset() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .ok()
    }

    fn get_segment_for_block_offset(&self, block_offset: BlockOffset) -> Option<&DirectorySegment> {
        let segment_index = self.get_segment_index_for_block_offset(block_offset)?;
        self.segments.get(segment_index)
    }

    fn get_segment_for_next_block_offset(
        &self,
        next_block_offset: BlockOffset,
    ) -> Option<&DirectorySegment> {
        if next_block_offset > 0 {
            self.get_segment_for_block_offset(next_block_offset - 1)
        } else {
            None
        }
    }
}

impl ChainStore for DirectoryChainStore {
    fn segments(&self) -> Vec<Segment> {
        self.segments
            .iter()
            .map(|segment| Segment {
                range: segment.offset_range(),
            })
            .collect()
    }

    fn write_block<B: Block>(&mut self, block: &B) -> Result<BlockOffset, Error> {
        debug!("Writing block at offset {}", block.offset());

        let (block_segment, written_in_segment) = {
            let need_new_segment = {
                match self.segments.last() {
                    None => true,
                    Some(s) => {
                        let new_block_end_offset =
                            s.next_file_offset() as u64 + block.total_size() as u64;
                        new_block_end_offset > self.config.segment_max_size
                    }
                }
            };

            if need_new_segment {
                let segment = DirectorySegment::create(self.config, &self.directory, block)?;
                self.segments.push(segment);
            }

            (self.segments.last_mut().unwrap(), need_new_segment)
        };

        // when creating new segment, blocks get written right away
        if !written_in_segment {
            block_segment.write_block(block)?;
        }

        let operations_index = self
            .operations_index
            .as_mut()
            .expect("Operations index was none, which shouldn't be possible");
        operations_index.index_block(block)?;

        Ok(block_segment.next_block_offset())
    }

    fn blocks_iter(&self, from_offset: BlockOffset) -> Result<StoredBlockIterator, Error> {
        Ok(Box::new(DirectoryBlockIterator {
            directory: self,
            current_offset: from_offset,
            current_segment: None,
            last_error: None,
            reverse: false,
            done: false,
        }))
    }

    fn blocks_iter_reverse(
        &self,
        from_next_offset: BlockOffset,
    ) -> Result<StoredBlockIterator, Error> {
        let segment = self
            .get_segment_for_next_block_offset(from_next_offset)
            .ok_or_else(|| {
                Error::OutOfBound(format!("No segment with next offset {}", from_next_offset))
            })?;

        let last_block = segment.get_block_from_next_offset(from_next_offset)?;

        Ok(Box::new(DirectoryBlockIterator {
            directory: self,
            current_offset: last_block.offset,
            current_segment: None,
            last_error: None,
            reverse: true,
            done: false,
        }))
    }

    fn get_block(&self, offset: BlockOffset) -> Result<BlockRef, Error> {
        let segment = self.get_segment_for_block_offset(offset).ok_or_else(|| {
            Error::OutOfBound(format!("No segment has block with offset {}", offset))
        })?;

        segment.get_block(offset)
    }

    fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<BlockRef, Error> {
        let segment = self
            .get_segment_for_next_block_offset(next_offset)
            .ok_or_else(|| {
                Error::OutOfBound(format!(
                    "No segment has block with next offset {}",
                    next_offset
                ))
            })?;

        segment.get_block_from_next_offset(next_offset)
    }

    fn get_last_block(&self) -> Result<Option<BlockRef>, Error> {
        let last_segment = if let Some(last_segment) = self.segments.last() {
            last_segment
        } else {
            return Ok(None);
        };

        let last_block =
            last_segment.get_block_from_next_offset(last_segment.next_block_offset())?;
        Ok(Some(last_block))
    }

    fn get_block_by_operation_id(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<BlockRef>, Error> {
        let operations_index = self
            .operations_index
            .as_ref()
            .expect("Operations index was none, which shouldn't be possible");

        if let Some(block_offset) = operations_index.get_operation_block(operation_id)? {
            let block = self.get_block(block_offset)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    fn truncate_from_offset(&mut self, offset: BlockOffset) -> Result<(), Error> {
        let segment_index = self
            .get_segment_index_for_block_offset(offset)
            .ok_or_else(|| Error::OutOfBound(format!("No segment has offset {}", offset)))?;

        let truncate_to = {
            let segment = &mut self.segments[segment_index];
            if offset > segment.first_block_offset() {
                segment.truncate_from_block_offset(offset)?;
                segment_index + 1
            } else {
                segment_index
            }
        };

        if truncate_to < self.segments.len() {
            let removed_segments = self.segments.split_off(truncate_to);
            for segment in removed_segments {
                segment.delete()?;
            }
        }

        // we need to take out the index because it needs a block iterator that depends
        // on the store itself TODO: To be solved in https://github.com/appaquet/exocore/issues/34
        let mut index = self
            .operations_index
            .take()
            .expect("Operations index was none, which shouldn't be possible");
        index.truncate_from_offset(offset)?;
        let next_index_offset = index.next_expected_block_offset();
        let blocks_to_index = self.blocks_iter(next_index_offset)?;
        index.index_blocks(blocks_to_index)?;
        self.operations_index = Some(index);

        Ok(())
    }
}

/// Configuration for directory based chain persistence.
#[derive(Copy, Clone, Debug)]
pub struct DirectoryChainStoreConfig {
    pub segment_over_allocate_size: u64,
    pub segment_min_free_size: u64,
    pub segment_max_size: u64,
    pub operations_index_max_memory_items: usize,
}

impl Default for DirectoryChainStoreConfig {
    fn default() -> Self {
        DirectoryChainStoreConfig {
            segment_over_allocate_size: 100 * 1024 * 1024, // 100mb
            segment_min_free_size: 10 * 1024 * 1024,       // 10mb
            segment_max_size: 1024 * 1024 * 1024,          // 1gb
            operations_index_max_memory_items: 10000,
        }
    }
}

/// Iterator over blocks stored in this directory based chain persistence.
struct DirectoryBlockIterator<'pers> {
    directory: &'pers DirectoryChainStore,
    current_offset: BlockOffset,
    current_segment: Option<&'pers DirectorySegment>,
    last_error: Option<Error>,
    reverse: bool,
    done: bool,
}

impl<'pers> Iterator for DirectoryBlockIterator<'pers> {
    type Item = BlockRef<'pers>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if self.current_segment.is_none() {
            self.current_segment = self
                .directory
                .get_segment_for_block_offset(self.current_offset);
        }

        let (item, data_size, end_of_segment) = match self.current_segment {
            Some(segment) => {
                let block = segment
                    .get_block(self.current_offset)
                    .map_err(|err| {
                        error!("Got an error getting block in iterator: {}", err);
                        self.last_error = Some(err);
                    })
                    .ok()?;

                let data_size = block.total_size() as BlockOffset;
                let end_of_segment = if !self.reverse {
                    (self.current_offset + data_size) >= segment.next_block_offset()
                } else {
                    data_size > self.current_offset
                        || (self.current_offset - data_size) < segment.first_block_offset()
                };

                (block, data_size, end_of_segment)
            }
            None => {
                return None;
            }
        };

        if end_of_segment {
            self.current_segment = None;
        }

        // if we're in reverse and next offset would be lower than 0, we indicate we're
        // done
        if self.reverse && data_size > self.current_offset {
            self.done = true;
        }

        if !self.done {
            if !self.reverse {
                self.current_offset += data_size;
            } else {
                self.current_offset -= data_size;
            }
        }

        Some(item)
    }
}

/// Directory chain store specific errors
#[derive(Clone, Debug, thiserror::Error)]
pub enum DirectoryError {
    #[error("Error building operations index: {0:?}")]
    OperationsIndexBuild(Arc<extindex::BuilderError>),

    #[error("Error reading operations index: {0:?}")]
    OperationsIndexRead(Arc<extindex::ReaderError>),
}

#[cfg(test)]
pub mod tests {
    use exocore_core::cell::LocalNode;
    use exocore_core::utils::range;
    use itertools::Itertools;

    use crate::block::{Block, BlockOperations, BlockOwned};

    use super::*;
    use exocore_core::cell::FullCell;

    #[test]
    fn directory_chain_create_and_open() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let config: DirectoryChainStoreConfig = Default::default();

        let init_segments = {
            let mut directory_chain = DirectoryChainStore::create(config, dir.path())?;

            let block = create_block(&cell, 0);
            let second_offset = directory_chain.write_block(&block)?;

            let block = directory_chain.get_block(0)?;
            assert_eq!(block.offset, 0);
            let block = directory_chain.get_block_from_next_offset(second_offset)?;
            assert_eq!(block.offset, 0);

            let block = create_block(&cell, second_offset);
            let third_offset = directory_chain.write_block(&block)?;
            let block = directory_chain.get_block(second_offset)?;
            assert_eq!(block.offset, second_offset);
            let block = directory_chain.get_block_from_next_offset(third_offset)?;
            assert_eq!(block.offset, second_offset);

            let segments = directory_chain.segments();
            let data_size = (block.total_size() * 2) as BlockOffset;
            assert_eq!(
                segments,
                vec![Segment {
                    range: 0..data_size
                }]
            );
            segments
        };

        {
            // already exists
            assert!(DirectoryChainStore::create(config, dir.path()).is_err());
        }

        {
            let directory_chain = DirectoryChainStore::open(config, dir.path())?;
            assert_eq!(directory_chain.segments(), init_segments);
        }

        Ok(())
    }

    #[test]
    fn directory_chain_invalid_path() -> anyhow::Result<()> {
        let config: DirectoryChainStoreConfig = Default::default();

        {
            assert!(DirectoryChainStore::create(config, &PathBuf::from("/invalid/path")).is_err());
        }

        {
            assert!(DirectoryChainStore::open(config, &PathBuf::from("/invalid/path")).is_err());
        }

        {
            // directory should be empty
            let dir = tempfile::tempdir()?;
            std::fs::write(dir.path().join("some_file"), "hello")?;
            assert!(DirectoryChainStore::create(config, dir.path()).is_err());
        }

        Ok(())
    }

    #[test]
    fn directory_chain_write_until_second_segment() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_max_size = 350_000;

        fn validate_directory(directory_chain: &DirectoryChainStore) -> anyhow::Result<()> {
            let segments = directory_chain
                .segments()
                .iter()
                .map(|seg| seg.range.clone())
                .collect_vec();
            assert!(range::are_continuous(segments.iter()));
            assert_eq!(2, segments.len());

            let block = directory_chain.get_block(0)?;
            assert_eq!(block.offset, 0);

            let block = directory_chain.get_block(segments[0].end)?;
            assert_eq!(block.offset, segments[0].end);

            let block = directory_chain.get_block_from_next_offset(segments[0].end)?;
            assert_eq!(block.next_offset(), segments[0].end);

            let block = directory_chain.get_block_from_next_offset(segments[0].end)?;
            assert_eq!(block.next_offset(), segments[0].end);

            let last_block = directory_chain.get_block_from_next_offset(segments[1].end)?;

            let last_block_offset = last_block.offset;
            let next_block_offset = last_block.next_offset();
            assert_eq!(next_block_offset, segments[1].end);

            // validate data using forward and reverse iterators
            let iterator = directory_chain.blocks_iter(0)?;
            validate_iterator(iterator, 1000, 0, last_block_offset, false);

            let next_block_offset = segments.last().unwrap().end;
            let reverse_iterator = directory_chain.blocks_iter_reverse(next_block_offset)?;
            validate_iterator(reverse_iterator, 1000, last_block_offset, 0, true);

            Ok(())
        }

        let init_segments = {
            let mut directory_chain = DirectoryChainStore::create(config, dir.path())?;

            append_blocks(&cell, &mut directory_chain, 1000, 0);
            validate_directory(&directory_chain)?;
            validate_directory_operations_index(&directory_chain)?;

            directory_chain.segments()
        };

        {
            let directory_chain = DirectoryChainStore::open(config, dir.path())?;
            assert_eq!(directory_chain.segments(), init_segments);

            validate_directory(&directory_chain)?;
            validate_directory_operations_index(&directory_chain)?;
        }

        Ok(())
    }

    #[test]
    fn directory_chain_truncate() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_max_size = 1000;
        config.segment_over_allocate_size = 1500;

        // we cutoff the directory at different position to make sure of its integrity
        for cutoff in 1..30 {
            let dir = tempfile::tempdir()?;

            let (segments_before, block_n_offset, block_n_plus_offset) = {
                let mut directory_chain = DirectoryChainStore::create(config, dir.path())?;
                append_blocks(&cell, &mut directory_chain, 30, 0);
                let segments_before = directory_chain
                    .segments()
                    .iter()
                    .map(|seg| seg.range.clone())
                    .collect_vec();

                let block_n = directory_chain.blocks_iter(0)?.nth(cutoff - 1).unwrap();
                let block_n_offset = block_n.offset;
                let block_n_plus_offset = block_n.next_offset();

                directory_chain.truncate_from_offset(block_n_plus_offset)?;

                let segments_after = directory_chain
                    .segments()
                    .iter()
                    .map(|seg| seg.range.clone())
                    .collect_vec();
                assert_ne!(segments_before, segments_after);
                assert_eq!(segments_after.last().unwrap().end, block_n_plus_offset);
                assert_eq!(
                    directory_chain.get_last_block()?.unwrap().offset,
                    block_n_offset
                );

                let iter = directory_chain.blocks_iter(0)?;
                validate_iterator(iter, cutoff, 0, block_n_offset, false);

                let iter_reverse = directory_chain.blocks_iter_reverse(block_n_plus_offset)?;
                validate_iterator(iter_reverse, cutoff, block_n_offset, 0, true);

                validate_directory_operations_index(&directory_chain)?;

                (segments_before, block_n_offset, block_n_plus_offset)
            };

            {
                let directory_chain = DirectoryChainStore::open(config, dir.path())?;
                let segments_after = directory_chain
                    .segments()
                    .iter()
                    .map(|seg| seg.range.clone())
                    .collect_vec();

                assert_ne!(segments_before, segments_after);
                assert_eq!(segments_after.last().unwrap().end, block_n_plus_offset);

                let iter = directory_chain.blocks_iter(0)?;
                validate_iterator(iter, cutoff, 0, block_n_offset, false);

                let iter_reverse = directory_chain.blocks_iter_reverse(block_n_plus_offset)?;
                validate_iterator(iter_reverse, cutoff, block_n_offset, 0, true);

                validate_directory_operations_index(&directory_chain)?;

                assert_eq!(
                    directory_chain.get_last_block()?.unwrap().offset,
                    block_n_offset
                );
            }
        }

        Ok(())
    }

    #[test]
    fn directory_chain_truncate_all() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_max_size = 3000;
        config.segment_over_allocate_size = 3500;

        {
            let mut directory_chain = DirectoryChainStore::create(config, dir.path())?;
            append_blocks(&cell, &mut directory_chain, 100, 0);

            directory_chain.truncate_from_offset(0)?;

            let segments_after = directory_chain.segments();
            assert!(segments_after.is_empty());
            assert!(directory_chain.get_last_block()?.is_none());
        }

        {
            let directory_chain = DirectoryChainStore::open(config, dir.path())?;
            let segments = directory_chain.segments();
            assert!(segments.is_empty());
            assert!(directory_chain.get_last_block()?.is_none());
        }

        Ok(())
    }

    fn append_blocks(
        cell: &FullCell,
        directory_chain: &mut DirectoryChainStore,
        nb_blocks: usize,
        from_offset: BlockOffset,
    ) {
        let mut next_offset = from_offset;
        for _i in 0..nb_blocks {
            let block = create_block(&cell, next_offset);
            next_offset = directory_chain.write_block(&block).unwrap();
        }
    }

    fn validate_iterator(
        iter: StoredBlockIterator,
        expect_count: usize,
        expect_first_offset: BlockOffset,
        expect_last_offset: BlockOffset,
        reverse: bool,
    ) {
        let mut first_block_offset: Option<BlockOffset> = None;
        let mut last_block_offset: Option<BlockOffset> = None;
        let mut count = 0;

        for stored_block in iter {
            count += 1;

            let block_header_reader = stored_block.header.get_reader().unwrap();
            let current_block_offset = block_header_reader.get_offset();
            assert_eq!(stored_block.offset, current_block_offset);

            if first_block_offset.is_none() {
                first_block_offset = Some(current_block_offset);
            }

            if let Some(last_block_offset) = last_block_offset {
                assert_eq!(
                    current_block_offset > last_block_offset,
                    !reverse,
                    "current offset > last offset"
                );
            }

            last_block_offset = Some(current_block_offset);
        }

        assert_eq!(count, expect_count);
        assert_eq!(first_block_offset.unwrap(), expect_first_offset);
        assert_eq!(last_block_offset.unwrap(), expect_last_offset);
    }

    fn validate_directory_operations_index(store: &DirectoryChainStore) -> anyhow::Result<()> {
        let all_blocks_offsets = store
            .blocks_iter(0)?
            .map(|block| block.offset)
            .collect_vec();

        for block_offset in all_blocks_offsets {
            // `create_block` use the block offset for proposal operation id
            let block = store.get_block_by_operation_id(block_offset)?.unwrap();
            assert_eq!(block_offset, block.offset);

            // `create_block` creates 1 operation in the block with offset +1 as operation
            // id
            let block = store.get_block_by_operation_id(block_offset + 1)?.unwrap();
            assert_eq!(block_offset, block.offset);

            // invalid operation id
            assert!(store.get_block_by_operation_id(block_offset + 2)?.is_none());
        }

        Ok(())
    }

    pub fn create_block(cell: &FullCell, offset: BlockOffset) -> BlockOwned {
        // only true for tests
        let operation_id = offset as u64 + 1;
        let operations = vec![
            crate::operation::OperationBuilder::new_entry(
                operation_id,
                cell.local_node().id(),
                b"some_data",
            )
            .sign_and_build(cell.local_node())
            .unwrap()
            .frame,
        ];

        let proposed_operation_id = offset as u64;
        let block_operations = BlockOperations::from_operations(operations.into_iter()).unwrap();
        BlockOwned::new_with_prev_info(
            &cell,
            offset,
            0,
            0,
            &[],
            proposed_operation_id,
            block_operations,
        )
        .unwrap()
    }
}
