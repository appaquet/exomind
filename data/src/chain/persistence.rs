use std;
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use crate::chain_block_capnp::{block, block_signatures};
use crate::serialize;
use crate::serialize::{FramedMessage, FramedMessageIterator, FramedTypedMessage, MessageType};
use exocore_common::range;

use super::*;

// TODO TONIGHT
//   * Get block from next offset

// TODO: Opening segments could be faster to open by passing last known offset
// TODO: Caching of segments metadata
// TODO: Segments hash & sign hashes using in-memory key ==> Makes sure that nobody changed the file while we were offline

pub trait Persistence {
    fn write_block<B, S>(&mut self, block: &B, block_signatures: &S) -> Result<BlockOffset, Error>
    where
        B: serialize::FramedTypedMessage<block::Owned>,
        S: serialize::FramedTypedMessage<block_signatures::Owned>;

    fn available_segments(&self) -> Vec<range::Range<BlockOffset>>;

    fn block_iter(&self, from_offset: BlockOffset) -> Result<StoredBlockIterator, Error>;

    fn block_iter_reverse(
        &self,
        from_next_offset: BlockOffset,
    ) -> Result<StoredBlockIterator, Error>;

    fn get_block(&self, offset: BlockOffset) -> Result<StoredBlock, Error>;

    fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<StoredBlock, Error>;

    fn truncate_from_offset(&mut self, block_offset: BlockOffset);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    UnexpectedState,
    Serialization(serialize::Error),
    Integrity,
    SegmentFull,
    OutOfBound,
    IO,
}

///
/// Directory persistence
///
#[derive(Copy, Clone, Debug)]
struct DirectoryPersistenceConfig {
    segment_over_allocate_size: u64,
    segment_min_free_size: u64,
    segment_max_size: u64,
}

impl Default for DirectoryPersistenceConfig {
    fn default() -> Self {
        DirectoryPersistenceConfig {
            segment_over_allocate_size: 300 * 1024 * 1024, // 300mb
            segment_min_free_size: 10 * 1024 * 1024,       // 10mb
            segment_max_size: 4 * 1024 * 1024 * 1024,      // 4gb
        }
    }
}

struct DirectoryPersistence {
    config: DirectoryPersistenceConfig,
    directory: PathBuf,
    segments: Vec<DirectorySegment>,
}

impl DirectoryPersistence {
    fn create(
        config: DirectoryPersistenceConfig,
        directory_path: &Path,
    ) -> Result<DirectoryPersistence, Error> {
        if !directory_path.exists() {
            error!(
                "Tried to create directory at {:?}, but it didn't exist",
                directory_path
            );
            return Err(Error::UnexpectedState);
        }

        let paths = std::fs::read_dir(directory_path).map_err(|err| {
            error!("Error listing directory {:?}: {:?}", directory_path, err);
            Error::IO
        })?;

        if paths.count() > 0 {
            error!(
                "Tried to create directory at {:?}, but it's not empty",
                directory_path
            );
            return Err(Error::UnexpectedState);
        }

        Ok(DirectoryPersistence {
            config,
            directory: directory_path.to_path_buf(),
            segments: Vec::new(),
        })
    }

    fn open(
        config: DirectoryPersistenceConfig,
        directory_path: &Path,
    ) -> Result<DirectoryPersistence, Error> {
        if !directory_path.exists() {
            error!(
                "Tried to open directory at {:?}, but it didn't exist",
                directory_path
            );
            return Err(Error::UnexpectedState);
        }

        let mut segments = Vec::new();
        let paths = std::fs::read_dir(directory_path).map_err(|err| {
            error!("Error listing directory {:?}: {:?}", directory_path, err);
            Error::IO
        })?;
        for path in paths {
            let path = path.map_err(|err| {
                error!("Error getting directory entry {:?}", err);
                Error::IO
            })?;

            let segment = DirectorySegment::open(config, &path.path())?;
            segments.push(segment);
        }
        segments.sort_by(|a, b| a.first_block_offset.cmp(&b.first_block_offset));

        Ok(DirectoryPersistence {
            config,
            directory: directory_path.to_path_buf(),
            segments,
        })
    }

    fn get_segment_for_block_offset(&self, block_offset: BlockOffset) -> Option<&DirectorySegment> {
        let segment_position = self
            .segments
            .binary_search_by(|seg| {
                if block_offset >= seg.first_block_offset && block_offset < seg.next_block_offset {
                    Ordering::Equal
                } else if block_offset < seg.first_block_offset {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .ok()?;

        self.segments.get(segment_position)
    }
}

impl Persistence for DirectoryPersistence {
    fn write_block<B, S>(&mut self, block: &B, block_signatures: &S) -> Result<BlockOffset, Error>
    where
        B: serialize::FramedTypedMessage<block::Owned>,
        S: serialize::FramedTypedMessage<block_signatures::Owned>,
    {
        let (block_segment, written_in_segment) = {
            let need_new_segment = {
                let last_segment = self.segments.last();
                last_segment.is_none()
                    || last_segment.as_ref().unwrap().next_file_offset
                        > self.config.segment_max_size as usize
            };

            if need_new_segment {
                let segment = DirectorySegment::create(
                    self.config,
                    &self.directory,
                    block,
                    block_signatures,
                )?;
                self.segments.push(segment);
            }

            (self.segments.last_mut().unwrap(), need_new_segment)
        };

        // when creating new segment, blocks get written right away
        if !written_in_segment {
            block_segment.write_block(block, block_signatures)?;
        }

        Ok(block_segment.next_block_offset)
    }

    fn available_segments(&self) -> Vec<range::Range<BlockOffset>> {
        self.segments
            .iter()
            .map(|segment| segment.offset_range())
            .collect()
    }

    fn block_iter(&self, from_offset: BlockOffset) -> Result<StoredBlockIterator, Error> {
        Ok(Box::new(DirectoryBlockIterator {
            directory: self,
            current_offset: from_offset,
            current_segment: None,
            last_error: None,
            reverse: false,
            done: false,
        }))
    }

    fn block_iter_reverse(
        &self,
        from_next_offset: BlockOffset,
    ) -> Result<StoredBlockIterator, Error> {
        if from_next_offset == 0 {
            return Err(Error::OutOfBound);
        }

        let segment = self
            .get_segment_for_block_offset(from_next_offset - 1)
            .ok_or(Error::OutOfBound)?;
        let (last_block, _last_sigs) = segment.get_block_from_next_offset(from_next_offset)?;

        let last_block_reader = last_block.get()?;
        let last_block_offset = last_block_reader.get_offset();

        Ok(Box::new(DirectoryBlockIterator {
            directory: self,
            current_offset: last_block_offset,
            current_segment: None,
            last_error: None,
            reverse: true,
            done: false,
        }))
    }

    fn get_block(&self, _offset: BlockOffset) -> Result<StoredBlock, Error> {
        // TODO: Should return block + sig
        unimplemented!()
    }

    fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<StoredBlock, Error> {
        unimplemented!()
    }

    fn truncate_from_offset(&mut self, _block_offset: BlockOffset) {
        // TODO: Find the segment corresponding to block offset (first that is >=)
        // TODO: Any segments after should be deleted
        unimplemented!()
    }
}

struct DirectoryBlockIterator<'pers> {
    directory: &'pers DirectoryPersistence,
    current_offset: BlockOffset,
    current_segment: Option<&'pers DirectorySegment>,
    last_error: Option<Error>,
    reverse: bool,
    done: bool,
}

impl<'pers> Iterator for DirectoryBlockIterator<'pers> {
    type Item = StoredBlock<'pers>;

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
                let (block, signature) = segment
                    .get_block(self.current_offset)
                    .map_err(|err| {
                        error!("Got an error getting block in iterator: {:?}", err);
                        self.last_error = Some(err);
                    })
                    .ok()?;

                let data_size = (block.data_size() + signature.data_size()) as BlockOffset;
                let end_of_segment = if !self.reverse {
                    (self.current_offset + data_size) >= segment.next_block_offset
                } else {
                    data_size > self.current_offset
                        || (self.current_offset - data_size) < segment.first_block_offset
                };

                let item = StoredBlock {
                    block,
                    signatures: signature,
                };
                (item, data_size, end_of_segment)
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

struct DirectorySegment {
    config: DirectoryPersistenceConfig,
    first_block_offset: BlockOffset,
    segment_path: PathBuf,
    segment_file: SegmentFile,
    last_block_offset: BlockOffset,
    next_block_offset: BlockOffset,
    next_file_offset: usize,
}

impl DirectorySegment {
    fn create<B, S>(
        config: DirectoryPersistenceConfig,
        directory: &Path,
        block: &B,
        block_sigs: &S,
    ) -> Result<DirectorySegment, Error>
    where
        B: serialize::FramedTypedMessage<block::Owned>,
        S: serialize::FramedTypedMessage<block_signatures::Owned>,
    {
        let block_reader = block.get().unwrap();
        let first_block_offset = block_reader.get_offset();
        let last_block_offset = first_block_offset;

        let segment_path = Self::segment_path(directory, first_block_offset);
        if segment_path.exists() {
            error!(
                "Tried to create a new segment at path {:?}, but already existed",
                segment_path
            );
            return Err(Error::UnexpectedState);
        }

        info!(
            "Creating new segment at {:?} for offset {}",
            directory, first_block_offset
        );
        let mut segment_file = SegmentFile::open(&segment_path, config.segment_over_allocate_size)?;
        block.copy_into(&mut segment_file.mmap);
        block_sigs.copy_into(&mut segment_file.mmap[block.data_size()..]);
        let written_data_size = block.data_size() + block_sigs.data_size();

        Ok(DirectorySegment {
            config,
            first_block_offset,
            segment_path,
            segment_file,
            last_block_offset,
            next_block_offset: first_block_offset + written_data_size as BlockOffset,
            next_file_offset: written_data_size,
        })
    }

    fn open_with_first_offset(
        config: DirectoryPersistenceConfig,
        directory: &Path,
        first_offset: BlockOffset,
    ) -> Result<DirectorySegment, Error> {
        let segment_path = Self::segment_path(directory, first_offset);
        let segment = Self::open(config, &segment_path)?;

        if segment.first_block_offset != first_offset {
            error!(
                "First block offset != segment first_offset ({} != {})",
                segment.first_block_offset, first_offset
            );
            return Err(Error::Integrity);
        }

        Ok(segment)
    }

    fn open(
        config: DirectoryPersistenceConfig,
        segment_path: &Path,
    ) -> Result<DirectorySegment, Error> {
        info!("Opening segment at {:?}", segment_path);

        let segment_file = SegmentFile::open(&segment_path, 0)?;

        // read first block to validate it has the same offset as segment
        let first_block_offset = {
            let framed_message =
                serialize::FramedSliceMessage::new(&segment_file.mmap).map_err(|err| {
                    error!(
                        "Couldn't read first block from segment file {:?}: {:?}",
                        segment_path, err
                    );
                    err
                })?;
            let first_block = framed_message.get_root::<block::Reader>()?;
            first_block.get_offset()
        };

        // iterate through the segment and find the last block and its offset
        let (last_block_offset, next_block_offset, next_file_offset) = {
            let mut last_block_file_offset = None;
            let block_iter = FramedMessageIterator::new(&segment_file.mmap)
                .filter(|msg| msg.framed_message.message_type() == block::Owned::message_type());
            for message in block_iter {
                last_block_file_offset = Some(message.offset);
            }

            match last_block_file_offset {
                Some(file_offset) => {
                    let block_message =
                        serialize::FramedSliceMessage::new(&segment_file.mmap[file_offset..])?;
                    let block_reader = block_message.get_root::<block::Reader>()?;
                    let sigs_offset = file_offset + block_message.data_size();
                    let sigs_message =
                        serialize::FramedSliceMessage::new(&segment_file.mmap[sigs_offset..])?;

                    let written_data_size = block_message.data_size() + sigs_message.data_size();
                    (
                        block_reader.get_offset(),
                        block_reader.get_offset() + written_data_size as BlockOffset,
                        file_offset + written_data_size,
                    )
                }
                _ => {
                    error!("Couldn't find last block of segment: no blocks returned by iterator");
                    return Err(Error::Integrity);
                }
            }
        };

        Ok(DirectorySegment {
            config,
            first_block_offset,
            segment_path: segment_path.to_path_buf(),
            segment_file,
            last_block_offset,
            next_block_offset,
            next_file_offset,
        })
    }

    fn segment_path(directory: &Path, first_offset: BlockOffset) -> PathBuf {
        directory.join(format!("seg_{}", first_offset))
    }

    fn offset_range(&self) -> range::Range<BlockOffset> {
        range::Range::new(self.first_block_offset, self.next_block_offset)
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

    fn write_block<B, S>(&mut self, block: &B, block_sigs: &S) -> Result<(), Error>
    where
        B: serialize::FramedTypedMessage<block::Owned>,
        S: serialize::FramedTypedMessage<block_signatures::Owned>,
    {
        let next_file_offset = self.next_file_offset;
        let next_block_offset = self.next_block_offset;
        let block_size = block.data_size();
        let sigs_size = block_sigs.data_size();

        let block_reader = block.get()?;
        let block_offset = block_reader.get_offset();
        if next_block_offset != block_offset {
            error!("Trying to write a block at an offset that wasn't next offset: next_block_offset={} block_offset={}", next_block_offset, block_offset);
            return Err(Error::Integrity);
        }

        {
            self.ensure_file_size(block_size + sigs_size)?;
            block.copy_into(&mut self.segment_file.mmap[next_file_offset..]);
            block_sigs.copy_into(&mut self.segment_file.mmap[next_file_offset + block_size..]);
        }

        self.next_file_offset += block_size + sigs_size;
        self.next_block_offset += (block_size + sigs_size) as BlockOffset;

        Ok(())
    }

    fn get_block(
        &self,
        offset: BlockOffset,
    ) -> Result<
        (
            serialize::FramedSliceTypedMessage<block::Owned>,
            serialize::FramedSliceTypedMessage<block_signatures::Owned>,
        ),
        Error,
    > {
        let first_block_offset = self.first_block_offset;
        if offset < first_block_offset {
            error!(
                "Tried to read block at {}, but first offset was at {}",
                offset, first_block_offset
            );
            return Err(Error::OutOfBound);
        }

        if offset >= self.next_block_offset {
            error!(
                "Tried to read block at {}, but next offset was at {}",
                offset, self.next_block_offset
            );
            return Err(Error::OutOfBound);
        }

        let block_file_offset = (offset - first_block_offset) as usize;
        let block =
            serialize::FramedSliceTypedMessage::new(&self.segment_file.mmap[block_file_offset..])?;

        let block_sigs_file_offset = block_file_offset + block.data_size();
        let block_sigs = serialize::FramedSliceTypedMessage::new(
            &self.segment_file.mmap[block_sigs_file_offset..],
        )?;

        Ok((block, block_sigs))
    }

    fn get_block_from_next_offset(
        &self,
        next_offset: BlockOffset,
    ) -> Result<
        (
            serialize::FramedSliceTypedMessage<block::Owned>,
            serialize::FramedSliceTypedMessage<block_signatures::Owned>,
        ),
        Error,
    > {
        let first_block_offset = self.first_block_offset;
        if next_offset < first_block_offset {
            error!(
                "Tried to read block from next offset {}, but first offset was at {}",
                next_offset, first_block_offset
            );
            return Err(Error::OutOfBound);
        }

        if next_offset > self.next_block_offset {
            error!(
                "Tried to read block from next offset {}, but next offset was at {}",
                next_offset, self.next_block_offset
            );
            return Err(Error::OutOfBound);
        }

        let next_file_offset = (next_offset - first_block_offset) as usize;
        let block_sigs = serialize::FramedSliceTypedMessage::new_from_next_offset(
            &self.segment_file.mmap[0..],
            next_file_offset,
        )?;
        let block_sigs_file_offset = next_file_offset - block_sigs.data_size();

        let block = serialize::FramedSliceTypedMessage::new_from_next_offset(
            &self.segment_file.mmap[0..],
            block_sigs_file_offset,
        )?;

        Ok((block, block_sigs))
    }

    fn truncate_extra(&mut self) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset as u64;
        self.segment_file.set_len(next_file_offset)
    }
}

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
                error!("Error opening/creating segment file {:?}: {:?}", path, err);
                Error::IO
            })?;

        let mut current_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if current_size < minimum_size {
            current_size = minimum_size;
            file.set_len(minimum_size).map_err(|err| {
                error!("Error setting len of segment file {:?}: {:?}", path, err);
                Error::IO
            })?;
        }

        let mmap = unsafe {
            memmap::MmapOptions::new().map_mut(&file).map_err(|err| {
                error!("Error mmaping segment file {:?}: {:?}", path, err);
                Error::IO
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
            error!(
                "Error setting len of segment file {:?}: {:?}",
                self.path, err
            );
            Error::IO
        })?;

        self.mmap = unsafe {
            memmap::MmapOptions::new()
                .map_mut(&self.file)
                .map_err(|err| {
                    error!("Error mmaping segment file {:?}: {:?}", self.path, err);
                    Error::IO
                })?
        };

        self.current_size = new_size;
        Ok(())
    }
}

impl From<serialize::Error> for Error {
    fn from(err: serialize::Error) -> Self {
        Error::Serialization(err)
    }
}

#[cfg(test)]
mod tests {
    use tempdir;

    use super::*;
    use crate::serialize::{FramedOwnedTypedMessage, FramedTypedMessage};

    #[test]
    fn test_directory_persistence_create_and_open() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let config: DirectoryPersistenceConfig = Default::default();

        let init_segments = {
            let mut directory_persistence =
                DirectoryPersistence::create(config, dir.path()).unwrap();

            let block_msg = create_block(0);
            let sig_msg = create_block_sigs();
            let next_offset = directory_persistence
                .write_block(&block_msg, &sig_msg)
                .unwrap();

            let block_msg = create_block(next_offset);
            let sig_msg = create_block_sigs();
            directory_persistence
                .write_block(&block_msg, &sig_msg)
                .unwrap();

            let segments = directory_persistence.available_segments();
            let data_size = ((block_msg.data_size() + sig_msg.data_size()) * 2) as BlockOffset;
            assert_eq!(segments, vec![range::Range::new(0, data_size)]);
            segments
        };

        {
            // already exists
            assert!(DirectoryPersistence::create(config, dir.path()).is_err());
        }

        {
            let directory_persistence = DirectoryPersistence::open(config, dir.path()).unwrap();
            assert_eq!(directory_persistence.available_segments(), init_segments);
        }
    }

    #[test]
    fn test_directory_persistence_write_until_second_segment() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let mut config: DirectoryPersistenceConfig = Default::default();
        config.segment_max_size = 100_000;

        let init_segments = {
            let mut directory_persistence =
                DirectoryPersistence::create(config, dir.path()).unwrap();

            let mut next_offset = 0;
            for _i in 0..1000 {
                let block_msg = create_block(next_offset);
                let sig_msg = create_block_sigs();
                next_offset = directory_persistence
                    .write_block(&block_msg, &sig_msg)
                    .unwrap();
            }

            let segments = directory_persistence.available_segments();
            assert_eq!(segments.len(), 2);

            {
                let segment = directory_persistence
                    .get_segment_for_block_offset(0)
                    .unwrap();
                assert_eq!(segment.first_block_offset, 0);

                let segment = directory_persistence
                    .get_segment_for_block_offset(segments[0].to)
                    .unwrap();
                assert_eq!(segment.first_block_offset, segments[0].to);
            }

            // validate data using forward and reverse iterators
            let mut iterator = directory_persistence.block_iter(0).unwrap();
            validate_iterator(iterator, 1000, 0, 1000, false);

            let next_block_offset = segments.last().unwrap().to;
            let mut reverse_iterator = directory_persistence
                .block_iter_reverse(next_block_offset)
                .unwrap();
            validate_iterator(reverse_iterator, 1000, 0, 1000, true);

            segments
        };

        {
            let directory_persistence = DirectoryPersistence::open(config, dir.path()).unwrap();
            assert_eq!(directory_persistence.available_segments(), init_segments);

            // validate data using forward and reverse iterators
            let mut iterator = directory_persistence.block_iter(0).unwrap();
            validate_iterator(iterator, 1000, 0, 1000, false);

            let next_block_offset = init_segments.last().unwrap().to;
            let mut reverse_iterator = directory_persistence
                .block_iter_reverse(next_block_offset)
                .unwrap();
            validate_iterator(reverse_iterator, 1000, 0, 1000, true);
        }
    }

    #[test]
    fn test_directory_segment_create_and_open() {
        let dir = tempdir::TempDir::new("test").unwrap();

        let segment_id = 1234;
        let block_msg = create_block(1234);
        let sig_msg = create_block_sigs();

        {
            let segment =
                DirectorySegment::create(Default::default(), dir.path(), &block_msg, &sig_msg)
                    .unwrap();
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.last_block_offset, 1234);
            assert_eq!(
                segment.next_file_offset,
                block_msg.data_size() + sig_msg.data_size()
            );
            assert_eq!(
                segment.next_block_offset,
                1234 + (block_msg.data_size() + sig_msg.data_size()) as BlockOffset
            );
        }

        {
            let segment = DirectorySegment::open_with_first_offset(
                Default::default(),
                dir.path(),
                segment_id,
            )
            .unwrap();
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.last_block_offset, 1234);
            assert_eq!(
                segment.next_file_offset,
                block_msg.data_size() + sig_msg.data_size()
            );
            assert_eq!(
                segment.next_block_offset,
                1234 + (block_msg.data_size() + sig_msg.data_size()) as BlockOffset
            );
        }
    }

    #[test]
    fn test_directory_segment_create_already_exist() {
        let dir = tempdir::TempDir::new("test").unwrap();

        {
            let block_msg = create_block(1234);
            let sig_msg = create_block_sigs();
            let _segment =
                DirectorySegment::create(Default::default(), dir.path(), &block_msg, &sig_msg)
                    .unwrap();
        }

        {
            let block_msg = create_block(1234);
            let sig_msg = create_block_sigs();
            assert!(
                DirectorySegment::create(Default::default(), dir.path(), &block_msg, &sig_msg)
                    .is_err()
            );
        }
    }

    #[test]
    fn test_directory_segment_append_block() {
        let dir = tempdir::TempDir::new("test").unwrap();

        let offset1 = 0;
        let block = create_block(offset1);
        let block_sigs = create_block_sigs();
        let mut segment =
            DirectorySegment::create(Default::default(), dir.path(), &block, &block_sigs).unwrap();
        {
            let (read_block, read_block_sigs) = segment.get_block(offset1).unwrap();
            let (read_block, _read_block_sigs) =
                (read_block.get().unwrap(), read_block_sigs.get().unwrap());
            assert_eq!(read_block.get_offset(), offset1);
        }

        let offset2 = offset1 + (block.data_size() + block_sigs.data_size()) as u64;
        assert_eq!(segment.next_block_offset, offset2);
        let block = create_block(offset2);
        let block_sigs = create_block_sigs();
        segment.write_block(&block, &block_sigs).unwrap();
        {
            let (read_block, read_block_sigs) = segment.get_block(offset2).unwrap();
            let (read_block, _read_block_sigs) =
                (read_block.get().unwrap(), read_block_sigs.get().unwrap());
            assert_eq!(read_block.get_offset(), offset2);
        }

        let offset3 = offset2 + (block.data_size() + block_sigs.data_size()) as u64;
        assert_eq!(segment.next_block_offset, offset3);
        let block = create_block(offset3);
        let block_sigs = create_block_sigs();
        segment.write_block(&block, &block_sigs).unwrap();
        {
            let (read_block, read_block_sigs) = segment.get_block(offset3).unwrap();
            let (read_block, _read_block_sigs) =
                (read_block.get().unwrap(), read_block_sigs.get().unwrap());
            assert_eq!(read_block.get_offset(), offset3);
        }

        assert!(segment.get_block(10).is_err());
        assert!(segment.get_block(offset3 + 10).is_err());

        {
            let (last_block, last_sigs) = segment
                .get_block_from_next_offset(segment.next_block_offset)
                .unwrap();
            let block_reader = last_block.get().unwrap();
            assert_eq!(block_reader.get_offset(), offset3);
        }
    }

    #[test]
    fn test_directory_segment_non_zero_offset_write() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let config = Default::default();
        let segment_first_block_offset = 1234;

        {
            let first_block = create_block(segment_first_block_offset);
            let first_block_sigs = create_block_sigs();
            let mut segment =
                DirectorySegment::create(config, dir.path(), &first_block, &first_block_sigs)
                    .unwrap();
            let next_block_offset = segment.next_block_offset;
            assert_eq!(
                next_block_offset,
                segment_first_block_offset
                    + (first_block.data_size() + first_block_sigs.data_size()) as BlockOffset
            );
            append_blocks_and_sigs_directory_segment(&mut segment, next_block_offset, 999);
        }

        {
            let segment = DirectorySegment::open_with_first_offset(
                config,
                dir.path(),
                segment_first_block_offset,
            )
            .unwrap();
            assert!(segment.get_block(0).is_err());
            assert!(segment.get_block(1234).is_ok());
            assert!(segment.get_block(segment.next_block_offset).is_err());
            assert!(segment
                .get_block_from_next_offset(segment.next_block_offset)
                .is_ok());

            let iter = serialize::FramedMessageIterator::new(&segment.segment_file.mmap[0..]);
            assert_eq!(iter.count(), 2000); // blocks + sigs
        }
    }

    #[test]
    fn test_directory_segment_grow_and_truncate() {
        let mut config: DirectoryPersistenceConfig = Default::default();
        config.segment_over_allocate_size = 100_000;

        let dir = tempdir::TempDir::new("test").unwrap();
        let mut next_offset = 0;

        let block = create_block(next_offset);
        let block_sigs = create_block_sigs();
        let mut segment =
            DirectorySegment::create(config, dir.path(), &block, &block_sigs).unwrap();
        next_offset += (block.data_size() + block_sigs.data_size()) as u64;

        let init_segment_size = segment.segment_file.current_size;
        append_blocks_and_sigs_directory_segment(&mut segment, next_offset, 999);
        let end_segment_size = segment.segment_file.current_size;

        assert_eq!(init_segment_size, 100_000);
        assert!(end_segment_size >= 200_000);

        segment.truncate_extra().unwrap();

        let truncated_segment_size = segment.segment_file.current_size;
        assert!(truncated_segment_size < 200_000);

        let iter = serialize::FramedMessageIterator::new(&segment.segment_file.mmap[0..]);
        assert_eq!(iter.count(), 2000); // blocks + sigs
    }

    #[test]
    fn test_segment_file_create() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let segment_path = dir.path().join("segment_0.seg");

        let segment_file = SegmentFile::open(&segment_path, 1000).unwrap();
        assert_eq!(segment_file.current_size, 1000);
        drop(segment_file);

        let mut segment_file = SegmentFile::open(&segment_path, 10).unwrap();
        assert_eq!(segment_file.current_size, 1000);

        segment_file.set_len(2000).unwrap();
        assert_eq!(segment_file.current_size, 2000);
    }

    fn create_block(offset: u64) -> FramedOwnedTypedMessage<block::Owned> {
        let mut block_msg_builder = serialize::TypedMessageBuilder::<block::Owned>::new();
        {
            let mut block_builder = block_msg_builder.init_root();
            block_builder.set_hash("block_hash");
            block_builder.set_offset(offset);
        }
        block_msg_builder.into_framed().unwrap()
    }

    fn create_block_sigs() -> FramedOwnedTypedMessage<block_signatures::Owned> {
        let mut block_msg_builder =
            serialize::TypedMessageBuilder::<block_signatures::Owned>::new();
        {
            let _block_builder = block_msg_builder.init_root();
        }
        block_msg_builder.into_framed().unwrap()
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

        for stored_block in iter {
            let block_reader = stored_block.block.get().unwrap();
            let current_block_offset = block_reader.get_offset();
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
    }

    fn append_blocks_and_sigs_directory_segment(
        segment: &mut DirectorySegment,
        first_block_offset: BlockOffset,
        nb_blocks: usize,
    ) {
        let mut next_offset = first_block_offset;
        for _i in 0..nb_blocks {
            assert_eq!(next_offset, segment.next_block_offset);
            let block = create_block(next_offset);
            let block_sigs = create_block_sigs();
            segment.write_block(&block, &block_sigs).unwrap();
            next_offset += (block.data_size() + block_sigs.data_size()) as u64;
        }
    }
}
