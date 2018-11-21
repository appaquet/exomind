use super::*;

use std;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use chain_block_capnp::{block, block_signatures};
use serialize;
use serialize::{FramedMessage, FramedMessageIterator, FramedTypedMessage, MessageType};

// TODO: Segments
// TODO: AVOID COPIES
// TODO: Directory segment + mmap + write header
// TODO: Opening segments could be faster to open by passing last known offset
// TODO: Switch to usize since we should never reach more than 4gb segment to fit on 32bit machines

const SEGMENT_OVER_ALLOCATE_SIZE: u64 = 300 * 1024 * 1024; // 300mb
const SEGMENT_MIN_FREE_SIZE: u64 = 10 * 1024 * 1024; // 10mb
const SEGMENT_MAX_SIZE: u64 = 4 * 1024 * 1024 * 1024; // 4gb

pub trait Persistence {}

#[derive(Debug, Clone, PartialEq)]
enum Error {
    UnexpectedState,
    Serialization(serialize::Error),
    Integrity,
    SegmentFull,
    NotFound,
    EOF,
    IO,
}

impl From<serialize::Error> for Error {
    fn from(err: serialize::Error) -> Self {
        Error::Serialization(err)
    }
}

struct DirectoryPersistence {
    directory: PathBuf,
    opened_file: Vec<DirectorySegment>,
}

impl DirectoryPersistence {
    fn create(directory_path: &Path) -> Result<DirectoryPersistence, Error> {
        if directory_path.exists() {
            error!(
                "Tried to create directory at {:?}, but it already exist",
                directory_path
            );
            return Err(Error::UnexpectedState);
        }

        unimplemented!()
    }

    fn open(directory_path: &Path) -> Result<DirectoryPersistence, Error> {
        if !directory_path.exists() {
            error!(
                "Tried to open directory at {:?}, but it didn't exist",
                directory_path
            );
            return Err(Error::UnexpectedState);
        }

        // TODO: Check continuity of segments. If we are missing offsets, we should unfreeze? should it be up higher ?

        unimplemented!()
    }

    fn write_block<B, S>(&mut self, block: B, block_signatures: S) -> (BlockOffset, BlockSize)
    where
        B: serialize::FramedTypedMessage<block::Owned>,
        S: serialize::FramedTypedMessage<block_signatures::Owned>,
    {
        // TODO: Get current segment.
        // TODO: Try to write. If error "SegmentFull", go to next segment.

        // TODO: Check if current segment is right size (usize)
        // TODO: Maybe grow

        unimplemented!()
    }

    fn block_iterator(&mut self) {
        unimplemented!()
    }

    fn truncate_from_offset(&mut self, block_offset: BlockOffset) {
        unimplemented!()
    }
}

impl Persistence for DirectoryPersistence {
    // TODO:
}

struct DirectorySegment {
    first_block_offset: BlockOffset,
    segment_path: PathBuf,
    segment_file: Option<SegmentFile>,
    last_block_offset: BlockOffset,
    next_file_offset: usize,
}

impl DirectorySegment {
    fn create<B, S>(directory: &Path, block: &B, block_sigs: &S) -> Result<DirectorySegment, Error>
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

        let mut segment_file = SegmentFile::open(&segment_path, SEGMENT_OVER_ALLOCATE_SIZE)?;
        block.copy_into(&mut segment_file.mmap);
        block_sigs.copy_into(&mut segment_file.mmap[block.data_size()..]);

        Ok(DirectorySegment {
            first_block_offset,
            segment_path,
            segment_file: Some(segment_file),
            last_block_offset,
            next_file_offset: block.data_size() + block_sigs.data_size(),
        })
    }

    fn open(directory: &Path, first_offset: BlockOffset) -> Result<DirectorySegment, Error> {
        let segment_path = Self::segment_path(directory, first_offset);
        let segment_file = SegmentFile::open(&segment_path, 0)?;

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

        if first_block_offset != first_offset {
            error!(
                "First block offset != segment first_offset ({} != {})",
                first_block_offset, first_offset
            );
            return Err(Error::Integrity);
        }

        let (last_block_offset, next_file_offset) = {
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

                    (
                        block_reader.get_offset(),
                        sigs_offset + sigs_message.data_size(),
                    )
                }
                _ => {
                    error!("Couldn't find last block of segment: no blocks returned by iterator");
                    return Err(Error::Integrity);
                }
            }
        };

        Ok(DirectorySegment {
            first_block_offset,
            segment_path,
            segment_file: Some(segment_file),
            last_block_offset,
            next_file_offset,
        })
    }

    fn segment_path(directory: &Path, first_offset: BlockOffset) -> PathBuf {
        directory.join(format!("seg_{}", first_offset))
    }

    fn ensure_file_open(&mut self) -> Result<&mut SegmentFile, Error> {
        if self.segment_file.is_none() {
            self.segment_file = Some(SegmentFile::open(&self.segment_path, 0)?); // TODO: Ensure proper size
        }
        Ok(self.segment_file.as_mut().unwrap())
    }

    fn ensure_file_open_for_size(&mut self, write_size: usize) -> Result<&mut SegmentFile, Error> {
        let next_file_offset = self.next_file_offset;

        let segment_file = self.ensure_file_open()?;
        if segment_file.current_size < (next_file_offset + write_size) as u64 {
            let target_size = (next_file_offset + write_size) as u64 + SEGMENT_OVER_ALLOCATE_SIZE;
            if target_size > SEGMENT_MAX_SIZE {
                return Err(Error::SegmentFull);
            }

            segment_file.set_len(target_size)?;
        }
        Ok(segment_file)
    }

    fn write_block<B, S>(&mut self, block: &B, block_sigs: &S) -> Result<(), Error>
    where
        B: serialize::FramedTypedMessage<block::Owned>,
        S: serialize::FramedTypedMessage<block_signatures::Owned>,
    {
        let next_file_offset = self.next_file_offset;
        let block_size = block.data_size();
        let sigs_size = block_sigs.data_size();

        let block_reader = block.get()?;
        let block_offset = block_reader.get_offset();
        if next_file_offset as u64 != block_offset {
            error!("Trying to write a block at an offset that wasn't next offset: next_file_offset={} block_offset={}", next_file_offset, block_offset);
            return Err(Error::Integrity);
        }

        {
            let segment_file = self.ensure_file_open_for_size(block_size + sigs_size)?;
            block.copy_into(&mut segment_file.mmap[next_file_offset..]);
            block_sigs.copy_into(&mut segment_file.mmap[next_file_offset + block_size..]);
        }

        self.next_file_offset += block_size + sigs_size;

        Ok(())
    }

    fn get_block(
        &mut self,
        offset: BlockOffset,
    ) -> Result<
        (
            serialize::FramedSliceTypedMessage<block::Owned>,
            serialize::FramedSliceTypedMessage<block_signatures::Owned>,
        ),
        Error,
    > {
        let first_block_offset = self.first_block_offset;
        let segment_file = self.ensure_file_open()?;

        if offset < first_block_offset {
            error!(
                "Tried to read block at {}, but first offset was at {}",
                offset, first_block_offset
            );
            return Err(Error::NotFound);
        }

        let block_file_offset = (offset - first_block_offset) as usize;
        let block =
            serialize::FramedSliceTypedMessage::new(&segment_file.mmap[block_file_offset..])?;

        let block_sigs_file_offset = block_file_offset + block.data_size();
        let block_sigs =
            serialize::FramedSliceTypedMessage::new(&segment_file.mmap[block_sigs_file_offset..])?;

        Ok((block, block_sigs))
    }

    fn truncate_extra(&mut self) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset as u64;
        let file_segment = self.ensure_file_open()?;
        file_segment.set_len(next_file_offset)
    }

    fn close(&mut self) {
        self.segment_file = None;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serialize::{FramedOwnedTypedMessage, FramedTypedMessage};
    use tempdir;

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

    #[test]
    fn test_directory_segment_create_and_open() {
        let dir = tempdir::TempDir::new("test").unwrap();

        let segment_id = 1234;
        let block_msg = create_block(1234);
        let sig_msg = create_block_sigs();

        {
            let segment = DirectorySegment::create(dir.path(), &block_msg, &sig_msg).unwrap();
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.last_block_offset, 1234);
            assert_eq!(
                segment.next_file_offset,
                block_msg.data_size() + sig_msg.data_size()
            );
        }

        {
            let segment = DirectorySegment::open(dir.path(), segment_id).unwrap();
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.last_block_offset, 1234);
            assert_eq!(
                segment.next_file_offset,
                block_msg.data_size() + sig_msg.data_size()
            );
        }
    }

    #[test]
    fn test_directory_segment_create_already_exist() {
        let dir = tempdir::TempDir::new("test").unwrap();

        {
            let block_msg = create_block(1234);
            let sig_msg = create_block_sigs();
            let segment = DirectorySegment::create(dir.path(), &block_msg, &sig_msg).unwrap();
        }

        {
            let block_msg = create_block(1234);
            let sig_msg = create_block_sigs();
            assert!(DirectorySegment::create(dir.path(), &block_msg, &sig_msg).is_err());
        }
    }

    #[test]
    fn test_directory_segment_append_block() {
        let dir = tempdir::TempDir::new("test").unwrap();

        let offset1 = 0;
        let block = create_block(offset1);
        let block_sigs = create_block_sigs();
        let mut segment = DirectorySegment::create(dir.path(), &block, &block_sigs).unwrap();
        {
            let (read_block, read_block_sigs) = segment.get_block(offset1).unwrap();
            let (read_block, read_block_sigs) =
                (read_block.get().unwrap(), read_block_sigs.get().unwrap());
            assert_eq!(read_block.get_offset(), offset1);
        }

        let offset2 = offset1 + (block.data_size() + block_sigs.data_size()) as u64;
        let block = create_block(offset2);
        let block_sigs = create_block_sigs();
        segment.write_block(&block, &block_sigs).unwrap();
        {
            let (read_block, read_block_sigs) = segment.get_block(offset2).unwrap();
            let (read_block, read_block_sigs) =
                (read_block.get().unwrap(), read_block_sigs.get().unwrap());
            assert_eq!(read_block.get_offset(), offset2);
        }

        let offset3 = offset2 + (block.data_size() + block_sigs.data_size()) as u64;
        let block = create_block(offset3);
        let block_sigs = create_block_sigs();
        segment.write_block(&block, &block_sigs).unwrap();
        {
            let (read_block, read_block_sigs) = segment.get_block(offset3).unwrap();
            let (read_block, read_block_sigs) =
                (read_block.get().unwrap(), read_block_sigs.get().unwrap());
            assert_eq!(read_block.get_offset(), offset3);
        }

        assert!(segment.get_block(10).is_err());
        assert!(segment.get_block(offset3 + 10).is_err());
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
            let mut block_builder = block_msg_builder.init_root();
        }
        block_msg_builder.into_framed().unwrap()
    }
}
