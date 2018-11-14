use super::*;

use std;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use chain_block_capnp::block;
use serialization;
use serialization::{FramedMessage, FramedMessageIterator};

// TODO: Segments
// TODO: AVOID COPIES
// TODO: Since we pre-allocate mmap, we need to know if an incoming block will fit
// TODO: We will have to pre-allocate size in big chunks (10MB ? 50MB ?)
// TODO: directory segment + mmap + write header
// TODO: Opening segments could be faster to open by passing last known offset

const SEGMENT_OVER_ALLOCATE_SIZE: u64 = 300 * 1024 * 1024; // 300mb
const SEGMENT_MIN_FREE_SIZE: u64 = 10 * 1024 * 1024; // 10mb

pub trait Persistence {}

struct PersistedSegment {
    id: SegmentID,
    first_offset: BlockOffset,
    last_offset: BlockOffset,
    size: SegmentSize,
    frozen: bool, // if frozen, means that we have an last offset
}

struct PersistedBlock {
    offset: BlockOffset,
    hash: Hash,
    entries: Vec<PersistedEntry>,
}

struct PersistedEntry {
    hash: Hash,
    data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
enum Error {
    UnexpectedState,
    Serialization(serialization::Error),
    Data,
    EOF,
    IO,
}

impl From<serialization::Error> for Error {
    fn from(err: serialization::Error) -> Self {
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

    fn list_segments(&self) -> &[PersistedSegment] {
        unimplemented!()
    }

    fn freeze_segment(&mut self, segment_id: SegmentID) {}

    fn create_segment(&mut self, segment_id: SegmentID) {
        unimplemented!()
    }

    fn write_block(
        &mut self,
        segment_id: SegmentID,
        block_offset: BlockOffset,
        block: &Block,
    ) -> (BlockOffset, BlockSize) {
        // TODO: Check if current segment is right size (usize)
        // TODO: Maybe grow

        unimplemented!()
    }

    fn block_iterator(&mut self) {
        unimplemented!()
    }
}

impl Persistence for DirectoryPersistence {
    // TODO:
}

struct DirectorySegment {
    //meta: PersistedSegment,
    id: SegmentID,
    segment_file: Option<SegmentFile>,
    first_block_offset: BlockOffset,
    last_block_offset: BlockOffset,
    next_file_offset: usize,
}

impl DirectorySegment {
    fn create<B: serialization::FramedTypedMessage<block::Owned>>(
        directory: &Path,
        id: SegmentID,
        first_block_message: &B,
    ) -> Result<DirectorySegment, Error> {
        let block_reader = first_block_message.get().unwrap();
        let first_block_offset = block_reader.get_offset();
        let last_block_offset = first_block_offset;

        if first_block_offset != id {
            error!(
                "First block offset != segment id ({} != {})",
                first_block_offset, id
            );
            return Err(Error::UnexpectedState);
        }

        let segment_path = Self::segment_path(directory, id);
        let mut segment_file = SegmentFile::open(&segment_path, SEGMENT_OVER_ALLOCATE_SIZE)?; // TODO: Proper first size
        first_block_message.copy_into(&mut segment_file.mmap);

        Ok(DirectorySegment {
            id,
            segment_file: Some(segment_file),
            first_block_offset,
            last_block_offset,
            next_file_offset: first_block_message.data_size(),
        })
    }

    fn open(directory: &Path, id: SegmentID) -> Result<DirectorySegment, Error> {
        let segment_path = Self::segment_path(directory, id);
        let segment_file = SegmentFile::open(&segment_path, 0)?;

        let first_block_offset = {
            let framed_message = serialization::FramedSliceMessage::new(&segment_file.mmap)
                .map_err(|err| {
                    error!(
                        "Couldn't read first block from segment file {:?}: {:?}",
                        segment_path, err
                    );
                    err
                })?;
            let first_block = framed_message.get_root::<block::Reader>()?;
            first_block.get_offset()
        };

        if first_block_offset != id {
            error!(
                "First block offset != segment id ({} != {})",
                first_block_offset, id
            );
            return Err(Error::UnexpectedState);
        }

        let (last_file_offset, last_block_offset, next_file_offset) = {
            let mut last_block_file_offset = None;
            let mut last_data_size = None;
            let block_iter = FramedMessageIterator::new(&segment_file.mmap);
            for message in block_iter {
                last_block_file_offset = Some(message.offset);
                last_data_size = Some(message.framed_message.data_size());
            }

            match (last_block_file_offset, last_data_size) {
                (Some(file_offset), Some(data_size)) => {
                    let framed_message =
                        serialization::FramedSliceMessage::new(&segment_file.mmap[file_offset..])?;
                    let last_block = framed_message.get_root::<block::Reader>()?;
                    (
                        file_offset,
                        last_block.get_offset(),
                        file_offset + data_size,
                    )
                }
                _ => {
                    error!("Couldn't find last block of segment: no blocks returned by iterator");
                    return Err(Error::UnexpectedState);
                }
            }
        };

        Ok(DirectorySegment {
            id,
            segment_file: Some(segment_file),
            first_block_offset,
            last_block_offset,
            next_file_offset: last_file_offset,
        })
    }

    fn segment_path(directory: &Path, id: SegmentID) -> PathBuf {
        directory.join(format!("seg_{}", id))
    }

    fn write_block<B: serialization::FramedTypedMessage<block::Owned>>(
        &mut self,
        block: &B,
    ) -> Result<(), Error> {
        let next_offset = self.next_file_offset;
        let block_size = block.data_size();

        unimplemented!()
    }

    fn ensure_size(&mut self, write_size: usize) {
        unimplemented!()
    }

    fn truncate_extra(&mut self) {
        // TODO: Truncate extra space
        // TODO: Set frozen flag
    }

    fn grow(&mut self) {
        // TODO: If file was open, we update last known offset
        // TODO: Increase capacity
    }

    fn close(&mut self) {
        self.segment_file = None;
    }
}

fn field_read_to_error<E: std::fmt::Debug>(err: E, field_name: &str) -> Error {
    error!("Error reading field {}: {:?}", field_name, err);
    Error::Data
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
    fn test_directory_segment_write_block() {
        let dir = tempdir::TempDir::new("test").unwrap();

        let mut msg_builder = capnp::message::Builder::new_default();
        {
            let mut block_builder = msg_builder.init_root::<block::Builder>();
            block_builder.set_hash("block_hash");
            block_builder.set_offset(1234);
        }

        let block_message = serialization::FramedOwnedMessage::from_builder(&msg_builder)
            .unwrap()
            .into_typed::<block::Owned>();

        let segment_id = 1234;
        {
            let segment = DirectorySegment::create(dir.path(), segment_id, &block_message).unwrap();
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.last_block_offset, 1234);
        }

        {
            let segment = DirectorySegment::open(dir.path(), segment_id).unwrap();
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.last_block_offset, 1234);
        }
    }
}
