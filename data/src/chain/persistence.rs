use super::*;

use std;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use chain_block_capnp::{block, block_signatures};
use serialize;
use serialize::{FramedMessage, FramedMessageIterator, FramedTypedMessage, MessageType};

// TODO: Segments
// TODO: AVOID COPIES
// TODO: Since we pre-allocate mmap, we need to know if an incoming block will fit
// TODO: We will have to pre-allocate size in big chunks (10MB ? 50MB ?)
// TODO: directory segment + mmap + write header
// TODO: Opening segments could be faster to open by passing last known offset

// TODO: Switch to usize since we should never reach more than 4gb segment to fit on 32bit machines

const SEGMENT_OVER_ALLOCATE_SIZE: u64 = 300 * 1024 * 1024; // 300mb
const SEGMENT_MIN_FREE_SIZE: u64 = 10 * 1024 * 1024; // 10mb
const SEGMENT_MAX_SIZE: u64 = 4 * 1024 * 1024 * 1024; // 4gb

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
    Serialization(serialize::Error),
    Integrity,
    Data,
    SegmentFull,
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

    fn write_block<B, S>(
        &mut self,
        segment_id: SegmentID,
        block_offset: BlockOffset,
        block: B,
        block_signatures: S,
    ) -> (BlockOffset, BlockSize)
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
    //meta: PersistedSegment,
    id: SegmentID,
    segment_path: PathBuf,
    segment_file: Option<SegmentFile>,
    first_block_offset: BlockOffset,
    last_block_offset: BlockOffset,
    next_file_offset: usize,
}

impl DirectorySegment {
    fn create<B, S>(
        directory: &Path,
        id: SegmentID,
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

        if first_block_offset != id {
            error!(
                "First block offset != segment id ({} != {})",
                first_block_offset, id
            );
            return Err(Error::UnexpectedState);
        }

        let segment_path = Self::segment_path(directory, id);
        let mut segment_file = SegmentFile::open(&segment_path, SEGMENT_OVER_ALLOCATE_SIZE)?; // TODO: Proper first size

        block.copy_into(&mut segment_file.mmap);
        block_sigs.copy_into(&mut segment_file.mmap[block.data_size()..]);

        Ok(DirectorySegment {
            id,
            segment_path,
            segment_file: Some(segment_file),
            first_block_offset,
            last_block_offset,
            next_file_offset: block.data_size() + block_sigs.data_size(),
        })
    }

    fn open(directory: &Path, id: SegmentID) -> Result<DirectorySegment, Error> {
        let segment_path = Self::segment_path(directory, id);
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

        if first_block_offset != id {
            error!(
                "First block offset != segment id ({} != {})",
                first_block_offset, id
            );
            return Err(Error::UnexpectedState);
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
                    return Err(Error::UnexpectedState);
                }
            }
        };

        Ok(DirectorySegment {
            id,
            segment_path,
            segment_file: Some(segment_file),
            first_block_offset,
            last_block_offset,
            next_file_offset,
        })
    }

    fn segment_path(directory: &Path, id: SegmentID) -> PathBuf {
        directory.join(format!("seg_{}", id))
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

            segment_file.set_len(target_size);
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

        {
            let segment_file = self.ensure_file_open_for_size(block_size + sigs_size)?;

            {
                // TODO: Make sure that block.previous_block == previous_block
                let previous_sig_block = serialize::FramedSliceTypedMessage::<
                    block_signatures::Owned,
                >::new_from_next_offset(
                    &segment_file.mmap, next_file_offset
                )?;
                let previous_block_end_offset = next_file_offset - previous_sig_block.data_size();
                let previous_block = serialize::FramedSliceTypedMessage::<block_signatures::Owned>::new_from_next_offset(&segment_file.mmap, previous_block_end_offset)?;
            }

            block.copy_into(&mut segment_file.mmap[next_file_offset..]);
            block_sigs.copy_into(&mut segment_file.mmap[next_file_offset + block_size..]);
        }

        self.next_file_offset += block_size + sigs_size;

        unimplemented!()
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
    use serialize::FramedTypedMessage;
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

        let mut block_msg_builder = serialize::TypedMessageBuilder::<block::Owned>::new();
        {
            let mut block_builder = block_msg_builder.init_root();
            block_builder.set_hash("block_hash");
            block_builder.set_offset(1234);
        }
        let block_msg = block_msg_builder.into_framed().unwrap();

        let mut sig_msg_builder = serialize::TypedMessageBuilder::<block_signatures::Owned>::new();
        {
            let mut sig_builder = sig_msg_builder.init_root();
            sig_builder.set_offset(0);
        }
        let sig_msg = sig_msg_builder.into_framed().unwrap();

        let segment_id = 1234;
        {
            let segment =
                DirectorySegment::create(dir.path(), segment_id, &block_msg, &sig_msg).unwrap();
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
}
