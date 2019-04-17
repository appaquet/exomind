use std;
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::ops::Range;
use std::path::{Path, PathBuf};

use exocore_common::data_chain_capnp::block_signatures;
use exocore_common::serialization::framed;
use exocore_common::serialization::framed::TypedFrame;

use super::*;

///
/// Directory based chain persistence. The chain is split in segments with configurable maximum size.
/// This maximum size allows using mmap on 32bit systems by preventing segments from growing over 4gb.
///
pub struct DirectoryChainStore {
    config: DirectoryChainStoreConfig,
    directory: PathBuf,
    segments: Vec<DirectorySegment>,
}

impl DirectoryChainStore {
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
            Error::IO(
                err.kind(),
                format!("Error listing directory {:?}: {:?}", directory_path, err),
            )
        })?;

        if paths.count() > 0 {
            return Err(Error::UnexpectedState(format!(
                "Tried to create directory at {:?}, but it's not empty",
                directory_path
            )));
        }

        Ok(DirectoryChainStore {
            config,
            directory: directory_path.to_path_buf(),
            segments: Vec::new(),
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
            Error::IO(
                err.kind(),
                format!("Error listing directory {:?}: {:?}", directory_path, err),
            )
        })?;
        for path in paths {
            let path = path.map_err(|err| {
                Error::IO(
                    err.kind(),
                    format!("Error getting directory entry {:?}", err),
                )
            })?;

            let segment = DirectorySegment::open(config, &path.path())?;
            segments.push(segment);
        }
        segments.sort_by(|a, b| a.first_block_offset.cmp(&b.first_block_offset));

        Ok(DirectoryChainStore {
            config,
            directory: directory_path.to_path_buf(),
            segments,
        })
    }

    fn get_segment_index_for_block_offset(&self, block_offset: BlockOffset) -> Option<usize> {
        self.segments
            .binary_search_by(|seg| {
                if block_offset >= seg.first_block_offset && block_offset < seg.next_block_offset {
                    Ordering::Equal
                } else if block_offset < seg.first_block_offset {
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
        let (block_segment, written_in_segment) = {
            let need_new_segment = {
                match self.segments.last() {
                    None => true,
                    Some(s) => s.next_file_offset as u64 > self.config.segment_max_size,
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

        Ok(block_segment.next_block_offset)
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

        let last_block = last_segment.get_block_from_next_offset(last_segment.next_block_offset)?;
        Ok(Some(last_block))
    }

    fn get_block_by_operation_id(
        &self,
        _operation_id: OperationID,
    ) -> Result<Option<BlockRef>, Error> {
        // TODO: Implement index by operation id: https://github.com/appaquet/exocore/issues/43
        Ok(None)
    }

    fn truncate_from_offset(&mut self, offset: BlockOffset) -> Result<(), Error> {
        let segment_index = self
            .get_segment_index_for_block_offset(offset)
            .ok_or_else(|| Error::OutOfBound(format!("No segment has offset {}", offset)))?;

        let truncate_to = {
            let segment = &mut self.segments[segment_index];
            if offset > segment.first_block_offset {
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

        Ok(())
    }
}

///
/// Configuration for directory based chain persistence.
///
#[derive(Copy, Clone, Debug)]
pub struct DirectoryChainStoreConfig {
    pub segment_over_allocate_size: u64,
    pub segment_min_free_size: u64,
    pub segment_max_size: u64,
}

impl Default for DirectoryChainStoreConfig {
    fn default() -> Self {
        DirectoryChainStoreConfig {
            segment_over_allocate_size: 300 * 1024 * 1024, // 300mb
            segment_min_free_size: 10 * 1024 * 1024,       // 10mb
            segment_max_size: 4 * 1024 * 1024 * 1024,      // 4gb
        }
    }
}

///
/// Iterator over blocks stored in this directory based chain persistence.
///
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
                        error!("Got an error getting block in iterator: {:?}", err);
                        self.last_error = Some(err);
                    })
                    .ok()?;

                let data_size = block.total_size() as BlockOffset;
                let end_of_segment = if !self.reverse {
                    (self.current_offset + data_size) >= segment.next_block_offset
                } else {
                    data_size > self.current_offset
                        || (self.current_offset - data_size) < segment.first_block_offset
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

        // if we're in reverse and next offset would be lower than 0, we indicate we're done
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

///
/// A segment of the chain, stored in its own file (`segment_file`) and that should not exceed a size
/// specified by configuration.
///
/// As mmap can only accessed allocated space in the file, we need to pre-allocate space in the file.
/// When a block would exceed this pre-allocated space, we re-size and re-open the file.
///
struct DirectorySegment {
    config: DirectoryChainStoreConfig,
    first_block_offset: BlockOffset,
    segment_path: PathBuf,
    segment_file: SegmentFile,
    next_block_offset: BlockOffset,
    next_file_offset: usize,
}

impl DirectorySegment {
    fn create<B: Block>(
        config: DirectoryChainStoreConfig,
        directory: &Path,
        block: &B,
    ) -> Result<DirectorySegment, Error> {
        let block_reader = block.block().get_typed_reader().unwrap();
        let first_block_offset = block_reader.get_offset();

        let segment_path = Self::segment_path(directory, first_block_offset);
        if segment_path.exists() {
            return Err(Error::UnexpectedState(format!(
                "Tried to create a new segment at path {:?}, but already existed",
                segment_path
            )));
        }

        info!(
            "Creating new segment at {:?} for offset {}",
            directory, first_block_offset
        );
        let mut segment_file = SegmentFile::open(&segment_path, config.segment_over_allocate_size)?;
        block.copy_data_into(&mut segment_file.mmap[0..]);
        let written_data_size = block.total_size();

        Ok(DirectorySegment {
            config,
            first_block_offset,
            segment_path,
            segment_file,
            next_block_offset: first_block_offset + written_data_size as BlockOffset,
            next_file_offset: written_data_size,
        })
    }

    #[cfg(test)]
    fn open_with_first_offset(
        config: DirectoryChainStoreConfig,
        directory: &Path,
        first_offset: BlockOffset,
    ) -> Result<DirectorySegment, Error> {
        let segment_path = Self::segment_path(directory, first_offset);
        let segment = Self::open(config, &segment_path)?;

        if segment.first_block_offset != first_offset {
            return Err(Error::Integrity(format!(
                "First block offset != segment first_offset ({} != {})",
                segment.first_block_offset, first_offset
            )));
        }

        Ok(segment)
    }

    fn open(
        config: DirectoryChainStoreConfig,
        segment_path: &Path,
    ) -> Result<DirectorySegment, Error> {
        info!("Opening segment at {:?}", segment_path);

        let segment_file = SegmentFile::open(&segment_path, 0)?;

        // read first block to validate it has the same offset as segment
        let first_block = BlockRef::new(&segment_file.mmap[..]).map_err(|err| {
            error!(
                "Couldn't read first block from segment file {:?}: {:?}",
                segment_path, err
            );
            err
        })?;

        // iterate through segments and find the last block and its offset
        let blocks_iterator = ChainBlockIterator::new(&segment_file.mmap[..]);
        let last_block = blocks_iterator.last().ok_or_else(|| {
            Error::Integrity(
                "Couldn't find last block of segment: no blocks returned by iterator".to_string(),
            )
        })?;

        let next_block_offset = last_block.offset + last_block.total_size() as BlockOffset;
        let next_file_offset = (next_block_offset - first_block.offset) as usize;

        Ok(DirectorySegment {
            config,
            first_block_offset: first_block.offset,
            segment_path: segment_path.to_path_buf(),
            segment_file,
            next_block_offset,
            next_file_offset,
        })
    }

    fn segment_path(directory: &Path, first_offset: BlockOffset) -> PathBuf {
        directory.join(format!("seg_{}", first_offset))
    }

    fn offset_range(&self) -> Range<BlockOffset> {
        self.first_block_offset..self.next_block_offset
    }

    fn ensure_file_size(&mut self, write_size: usize) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset;

        if self.segment_file.current_size < (next_file_offset + write_size) as u64 {
            let target_size =
                (next_file_offset + write_size) as u64 + self.config.segment_over_allocate_size;
            if target_size > self.config.segment_max_size {
                return Err(Error::SegmentFull);
            }

            self.segment_file.set_len(target_size)?;
        }

        Ok(())
    }

    fn write_block<B: Block>(&mut self, block: &B) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset;
        let next_block_offset = self.next_block_offset;
        let block_size = block.total_size();

        let block_offset = block.offset();
        if next_block_offset != block_offset {
            return Err(Error::Integrity(format!("Trying to write a block at an offset that wasn't next offset: next_block_offset={} block_offset={}", next_block_offset, block_offset)));
        }

        {
            self.ensure_file_size(block_size)?;
            block.copy_data_into(&mut self.segment_file.mmap[next_file_offset..]);
        }

        self.next_file_offset += block_size;
        self.next_block_offset += block_size as BlockOffset;

        Ok(())
    }

    fn get_block(&self, offset: BlockOffset) -> Result<BlockRef, Error> {
        let first_block_offset = self.first_block_offset;
        if offset < first_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block at {}, but first offset was at {}",
                offset, first_block_offset
            )));
        }

        if offset >= self.next_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block at {}, but next offset was at {}",
                offset, self.next_block_offset
            )));
        }

        let block_file_offset = (offset - first_block_offset) as usize;
        BlockRef::new(&self.segment_file.mmap[block_file_offset..])
    }

    fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<BlockRef, Error> {
        let first_block_offset = self.first_block_offset;
        if next_offset < first_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block from next offset {}, but first offset was at {}",
                next_offset, first_block_offset
            )));
        }

        if next_offset > self.next_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block from next offset {}, but next offset was at {}",
                next_offset, self.next_block_offset
            )));
        }

        let next_file_offset = (next_offset - first_block_offset) as usize;
        let signatures = framed::TypedSliceFrame::new_from_next_offset(
            &self.segment_file.mmap[..],
            next_file_offset,
        )?;
        let signatures_reader: block_signatures::Reader = signatures.get_typed_reader()?;
        let signatures_offset = next_file_offset - signatures.frame_size();

        let operations_size = signatures_reader.get_operations_size() as usize;
        if operations_size > signatures_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block from next offset {}, but its operations size would exceed beginning of file (operations_size={} signatures_offset={})",
                next_offset, operations_size, signatures_offset,
            )));
        }

        let operations_offset = signatures_offset - operations_size;
        let operations_data =
            &self.segment_file.mmap[operations_offset..operations_offset + operations_size];

        let block = framed::TypedSliceFrame::new_from_next_offset(
            &self.segment_file.mmap[..],
            operations_offset,
        )?;

        let offset = first_block_offset + (signatures_offset as BlockOffset)
            - (block.frame_size() as BlockOffset)
            - (operations_size as BlockOffset);
        Ok(BlockRef {
            offset,
            operations_data,
            block,
            signatures,
        })
    }

    #[cfg(test)]
    fn truncate_extra(&mut self) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset as u64;
        self.segment_file.set_len(next_file_offset)
    }

    fn truncate_from_block_offset(&mut self, block_offset: BlockOffset) -> Result<(), Error> {
        if block_offset < self.first_block_offset {
            return Err(Error::OutOfBound(format!(
                "Offset {} is before first block offset of segment {}",
                block_offset, self.first_block_offset
            )));
        }
        self.next_block_offset = block_offset;
        let keep_len = block_offset - self.first_block_offset;
        self.segment_file.set_len(keep_len)
    }

    fn delete(self) -> Result<(), Error> {
        let segment_path = self.segment_path.clone();
        drop(self);
        std::fs::remove_file(&segment_path).map_err(|err| {
            Error::IO(
                err.kind(),
                format!("Couldn't delete segment file {:?}: {:?}", segment_path, err),
            )
        })?;
        Ok(())
    }
}

///
/// Wraps a mmap'ed file stored on disk. As mmap cannot access content that is beyond the file size,
/// the segment is over-allocated so that we can write via mmap. If writing would exceed the size,
/// we re-allocate the file and re-open the mmap.
///
struct SegmentFile {
    path: PathBuf,
    file: File,
    mmap: memmap::MmapMut,
    current_size: u64,
}

impl SegmentFile {
    fn open(path: &Path, minimum_size: u64) -> Result<SegmentFile, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|err| {
                Error::IO(
                    err.kind(),
                    format!("Error opening/creating segment file {:?}: {:?}", path, err),
                )
            })?;

        let mut current_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if current_size < minimum_size {
            current_size = minimum_size;
            file.set_len(minimum_size).map_err(|err| {
                Error::IO(
                    err.kind(),
                    format!("Error setting len of segment file {:?}: {:?}", path, err),
                )
            })?;
        }

        let mmap = unsafe {
            memmap::MmapOptions::new().map_mut(&file).map_err(|err| {
                Error::IO(
                    err.kind(),
                    format!("Error mmaping segment file {:?}: {:?}", path, err),
                )
            })?
        };

        Ok(SegmentFile {
            path: path.to_path_buf(),
            file,
            mmap,
            current_size,
        })
    }

    fn set_len(&mut self, new_size: u64) -> Result<(), Error> {
        self.file.set_len(new_size).map_err(|err| {
            Error::IO(
                err.kind(),
                format!(
                    "Error setting len of segment file {:?}: {:?}",
                    self.path, err
                ),
            )
        })?;

        self.mmap = unsafe {
            memmap::MmapOptions::new()
                .map_mut(&self.file)
                .map_err(|err| {
                    Error::IO(
                        err.kind(),
                        format!("Error mmaping segment file {:?}: {:?}", self.path, err),
                    )
                })?
        };

        self.current_size = new_size;
        Ok(())
    }
}

///
/// Block iterator over a slice of data.
///
struct ChainBlockIterator<'a> {
    current_offset: usize,
    data: &'a [u8],
    last_error: Option<Error>,
}

impl<'a> ChainBlockIterator<'a> {
    fn new(data: &'a [u8]) -> ChainBlockIterator<'a> {
        ChainBlockIterator {
            current_offset: 0,
            data,
            last_error: None,
        }
    }
}

impl<'a> Iterator for ChainBlockIterator<'a> {
    type Item = BlockRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset >= self.data.len() {
            return None;
        }

        let block_res = BlockRef::new(&self.data[self.current_offset..]);
        match block_res {
            Ok(block) => {
                self.current_offset += block.total_size();
                Some(block)
            }
            Err(Error::Framing(framed::Error::EOF(_))) => None,
            Err(other) => {
                self.last_error = Some(other);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tempdir;

    use super::*;
    use exocore_common::range;
    use exocore_common::serialization::framed::TypedFrame;
    use itertools::Itertools;

    #[test]
    fn directory_chain_create_and_open() -> Result<(), failure::Error> {
        let dir = tempdir::TempDir::new("test")?;
        let config: DirectoryChainStoreConfig = Default::default();

        let init_segments = {
            let mut directory_chain = DirectoryChainStore::create(config, dir.path())?;

            let block = create_block(0);
            let second_offset = directory_chain.write_block(&block)?;

            let block = directory_chain.get_block(0)?;
            assert_eq!(block.offset, 0);
            let block = directory_chain.get_block_from_next_offset(second_offset)?;
            assert_eq!(block.offset, 0);

            let block = create_block(second_offset);
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
    fn directory_chain_write_until_second_segment() -> Result<(), failure::Error> {
        let dir = tempdir::TempDir::new("test")?;
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_max_size = 300_000;

        fn validate_directory(directory_chain: &DirectoryChainStore) -> Result<(), failure::Error> {
            let segments = directory_chain
                .segments()
                .iter()
                .map(|seg| seg.range.clone())
                .collect_vec();
            assert!(range::are_continuous(segments.iter()));
            assert_eq!(segments.len(), 2);

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

            append_blocks_to_directory(&mut directory_chain, 1000, 0);
            validate_directory(&directory_chain)?;

            directory_chain.segments()
        };

        {
            let directory_chain = DirectoryChainStore::open(config, dir.path())?;
            assert_eq!(directory_chain.segments(), init_segments);

            validate_directory(&directory_chain)?;
        }

        Ok(())
    }

    #[test]
    fn directory_chain_truncate() -> Result<(), failure::Error> {
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_max_size = 1000;
        config.segment_over_allocate_size = 1500;

        // we cutoff the directory at different position to make sure of its integrity
        for cutoff in 1..50 {
            let dir = tempdir::TempDir::new("test")?;

            let (segments_before, block_n_offset, block_n_plus_offset) = {
                let mut directory_chain = DirectoryChainStore::create(config, dir.path())?;
                append_blocks_to_directory(&mut directory_chain, 50, 0);
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

                assert_eq!(
                    directory_chain.get_last_block()?.unwrap().offset,
                    block_n_offset
                );
            }
        }

        Ok(())
    }

    #[test]
    fn directory_chain_truncate_all() -> Result<(), failure::Error> {
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_max_size = 3000;
        config.segment_over_allocate_size = 3500;
        let dir = tempdir::TempDir::new("test")?;

        {
            let mut directory_chain = DirectoryChainStore::create(config, dir.path())?;
            append_blocks_to_directory(&mut directory_chain, 100, 0);

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

    #[test]
    fn directory_segment_create_and_open() -> Result<(), failure::Error> {
        let dir = tempdir::TempDir::new("test")?;

        let segment_id = 1234;
        let block = create_block(1234);

        {
            let segment = DirectorySegment::create(Default::default(), dir.path(), &block)?;
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.next_file_offset as usize, block.total_size());
            assert_eq!(
                segment.next_block_offset as usize,
                1234 + block.total_size()
            );
        }

        {
            let segment = DirectorySegment::open_with_first_offset(
                Default::default(),
                dir.path(),
                segment_id,
            )?;
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.next_file_offset as usize, block.total_size());
            assert_eq!(
                segment.next_block_offset as usize,
                1234 + block.total_size()
            );
        }

        Ok(())
    }

    #[test]
    fn directory_segment_create_already_exist() -> Result<(), failure::Error> {
        let dir = tempdir::TempDir::new("test")?;

        {
            let block = create_block(1234);
            let _segment = DirectorySegment::create(Default::default(), dir.path(), &block)?;
        }

        {
            let block = create_block(1234);
            assert!(DirectorySegment::create(Default::default(), dir.path(), &block).is_err());
        }

        Ok(())
    }

    #[test]
    fn directory_segment_append_block() -> Result<(), failure::Error> {
        let dir = tempdir::TempDir::new("test")?;

        let offset1 = 0;
        let block = create_block(offset1);
        let mut segment = DirectorySegment::create(Default::default(), dir.path(), &block)?;
        {
            let block = segment.get_block(offset1)?;
            assert_eq!(block.offset, offset1);
        }

        let offset2 = offset1 + block.total_size() as BlockOffset;
        assert_eq!(segment.next_block_offset, offset2);
        let block = create_block(offset2);
        segment.write_block(&block)?;
        {
            let block = segment.get_block(offset2)?;
            assert_eq!(block.offset, offset2);
        }

        let offset3 = offset2 + block.total_size() as BlockOffset;
        assert_eq!(segment.next_block_offset, offset3);
        let block = create_block(offset3);
        segment.write_block(&block)?;
        {
            let block = segment.get_block(offset3)?;
            assert_eq!(block.offset, offset3);
        }

        assert!(segment.get_block(10).is_err());
        assert!(segment.get_block(offset3 + 10).is_err());

        {
            let last_block = segment.get_block_from_next_offset(segment.next_block_offset)?;
            assert_eq!(last_block.offset, offset3);
        }

        Ok(())
    }

    #[test]
    fn directory_segment_non_zero_offset_write() -> Result<(), failure::Error> {
        let dir = tempdir::TempDir::new("test")?;
        let config = Default::default();
        let segment_first_block_offset = 1234;

        {
            let first_block = create_block(segment_first_block_offset);
            let mut segment = DirectorySegment::create(config, dir.path(), &first_block)?;
            let next_block_offset = segment.next_block_offset;
            assert_eq!(
                next_block_offset,
                segment_first_block_offset + first_block.total_size() as BlockOffset
            );
            append_blocks_to_segment(&mut segment, next_block_offset, 999);
        }

        {
            let segment = DirectorySegment::open_with_first_offset(
                config,
                dir.path(),
                segment_first_block_offset,
            )?;
            assert!(segment.get_block(0).is_err());
            assert!(segment.get_block(1234).is_ok());
            assert!(segment.get_block(segment.next_block_offset).is_err());
            assert!(segment
                .get_block_from_next_offset(segment.next_block_offset)
                .is_ok());

            let iter = ChainBlockIterator::new(&segment.segment_file.mmap[0..]);
            assert_eq!(iter.count(), 1000);
        }

        Ok(())
    }

    #[test]
    fn directory_segment_grow_and_truncate() -> Result<(), failure::Error> {
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_over_allocate_size = 100_000;

        let dir = tempdir::TempDir::new("test")?;
        let mut next_offset = 0;

        let block = create_block(next_offset);
        let mut segment = DirectorySegment::create(config, dir.path(), &block)?;
        next_offset += block.total_size() as u64;
        assert_eq!(segment.next_block_offset, next_offset);
        assert_eq!(segment.next_file_offset, block.total_size());

        let init_segment_size = segment.segment_file.current_size;
        append_blocks_to_segment(&mut segment, next_offset, 999);
        let end_segment_size = segment.segment_file.current_size;
        let next_file_offset = segment.next_file_offset;

        assert_eq!(init_segment_size, 100_000);
        assert!(end_segment_size >= 200_000);

        segment.truncate_extra()?;

        let truncated_segment_size = segment.segment_file.current_size;
        assert_eq!(truncated_segment_size, next_file_offset as u64);

        let iter = ChainBlockIterator::new(&segment.segment_file.mmap[0..]);
        assert_eq!(iter.count(), 1000);

        Ok(())
    }

    #[test]
    fn segment_file_create() -> Result<(), failure::Error> {
        let dir = tempdir::TempDir::new("test")?;
        let segment_path = dir.path().join("segment_0.seg");

        let segment_file = SegmentFile::open(&segment_path, 1000)?;
        assert_eq!(segment_file.current_size, 1000);
        drop(segment_file);

        let mut segment_file = SegmentFile::open(&segment_path, 10)?;
        assert_eq!(segment_file.current_size, 1000);

        segment_file.set_len(2000)?;
        assert_eq!(segment_file.current_size, 2000);

        Ok(())
    }

    fn create_block(offset: BlockOffset) -> BlockOwned {
        let mut nodes = Nodes::new();
        let node1 = Node::new("node1".to_string());
        nodes.add(node1.clone());

        // only true for tests
        let operation_id = offset as u64;
        let operations =
            vec![
                crate::pending::PendingOperation::new_entry(operation_id, "node1", b"some_data")
                    .as_owned_framed(node1.frame_signer())
                    .unwrap(),
            ];

        let block_operations = BlockOperations::from_operations(operations.into_iter()).unwrap();
        BlockOwned::new_with_prev_info(&nodes, &node1, offset, 0, 0, &[], 0, block_operations)
            .unwrap()
    }

    fn append_blocks_to_directory(
        directory_chain: &mut DirectoryChainStore,
        nb_blocks: usize,
        from_offset: BlockOffset,
    ) {
        let mut next_offset = from_offset;
        for _i in 0..nb_blocks {
            let block = create_block(next_offset);
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

            let block_reader = stored_block.block.get_typed_reader().unwrap();
            let current_block_offset = block_reader.get_offset();
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

    fn append_blocks_to_segment(
        segment: &mut DirectorySegment,
        first_block_offset: BlockOffset,
        nb_blocks: usize,
    ) {
        let mut next_offset = first_block_offset;
        for _i in 0..nb_blocks {
            assert_eq!(next_offset, segment.next_block_offset);
            let block = create_block(next_offset);
            segment.write_block(&block).unwrap();
            next_offset += block.total_size() as u64;
        }
    }
}
