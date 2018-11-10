use super::*;

use std;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use byteorder;
use byteorder::ByteOrder;
use memmap;

use capnp;
use chain_block_capnp::{block, block_entry, segment_header};

// TODO: Segments
// TODO: AVOID COPIES
// TODO: Since we pre-allocate mmap, we need to know if an incoming block will fit
// TODO: We will have to pre-allocate size in big chunks (10MB ? 50MB ?)
// TODO: directory segment + mmap + write header
// TODO: Use ScratchSpace to reuse memory buffer for writes

// TODO: usize is not compatible with 32bit

const FRAMING_DATA_SIZE_BYTES: usize = 4;

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
    Data,
    EOF,
    IO,
}

struct DirectoryPersistence {
    directory: PathBuf,
    opened_file: Vec<DirectorySegment>,
}

impl DirectoryPersistence {
    fn create(directory_path: PathBuf) -> Result<DirectoryPersistence, Error> {
        if directory_path.exists() {
            error!(
                "Tried to create directory at {:?}, but it already exist",
                directory_path
            );
            return Err(Error::UnexpectedState);
        }

        unimplemented!()
    }

    fn open(directory_path: PathBuf) -> Result<DirectoryPersistence, Error> {
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
        block: Block,
    ) -> (BlockOffset, BlockSize) {
        // TODO: Maybe grow

        unimplemented!()
    }

    fn block_iterator(&mut self) {
        unimplemented!()
    }

    fn block_bytes_iterator(&mut self) {
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
    first_offset: Option<BlockOffset>,
    last_offset: Option<BlockOffset>,
    next_offset: Option<BlockOffset>,
}

impl DirectorySegment {
    fn create(directory: &Path, id: SegmentID) -> Result<DirectorySegment, Error> {
        unimplemented!()
    }

    fn open(directory: &Path, id: SegmentID) -> Result<DirectorySegment, Error> {
        let segment_path = directory.join(format!("seg_{}", id));
        let segment_file = SegmentFile::open(&segment_path, 0)?;
        let mut first_offset = None;
        let mut last_offset = None;
        let mut last_size = None;

        {
            let block_iter = FramedMessageIterator::new(&segment_file.mmap);
            for message in block_iter {
                if first_offset.is_none() {
                    first_offset = Some(message.offset as BlockOffset)
                }
                last_offset = Some(message.offset as BlockOffset);
                last_size = Some(message.framed_message.total_size as BlockOffset);
            }
        }

        let next_offset = match (last_offset, last_size) {
            (Some(offset), Some(size)) => Some(offset + size),
            _ => None,
        };
        Ok(DirectorySegment {
            id,
            segment_file: Some(segment_file),
            first_offset,
            last_offset,
            next_offset,
        })
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

    fn grow(&mut self, new_size: u64) -> Result<(), Error> {
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

struct FramedMessageIterator<'a> {
    buffer: &'a [u8],
    current_offset: usize,
    pub last_error: Option<Error>,
}

impl<'a> FramedMessageIterator<'a> {
    fn new(buffer: &'a [u8]) -> FramedMessageIterator<'a> {
        FramedMessageIterator {
            buffer,
            current_offset: 0,
            last_error: None,
        }
    }
}

impl<'a> Iterator for FramedMessageIterator<'a> {
    type Item = IteratedFramedSliceMessage<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.current_offset;
        let slice = &self.buffer[offset..];
        match read_framed_message(slice) {
            Ok(framed_message) => {
                self.current_offset += framed_message.total_size;
                Some(IteratedFramedSliceMessage {
                    offset,
                    framed_message,
                })
            }
            Err(Error::EOF) => None,
            Err(err) => {
                self.last_error = Some(err);
                None
            }
        }
    }
}

struct IteratedFramedSliceMessage<'a> {
    pub offset: usize,
    pub framed_message: FramedSliceMessage<'a>,
}

struct FramedSliceMessage<'a> {
    pub message_size: usize,
    pub total_size: usize,
    pub data: &'a [u8],
    pub reader: CapnSliceMessageReader<'a>,
}

impl<'a> FramedSliceMessage<'a> {
    fn to_owned(&self) -> FramedOwnedMessage {
        FramedOwnedMessage {
            message_size: self.message_size,
            total_size: self.total_size,
            message: OwnedMessage {
                data: self.data.to_vec(),
            },
        }
    }

    fn get_reader(&self) -> Result<&CapnSliceMessageReader, Error> {
        Ok(&self.reader)
    }
}

struct FramedOwnedMessage {
    pub message_size: usize,
    pub total_size: usize,
    pub message: OwnedMessage,
}

struct OwnedMessage {
    pub data: Vec<u8>,
}

impl OwnedMessage {
    fn get_reader(&self) -> Result<CapnSliceMessageReader, Error> {
        read_message(self.data.as_ref())
    }
}

#[inline]
fn pack_u32(value: u32, into: &mut [u8]) {
    debug_assert!(into.len() >= 4);
    byteorder::LittleEndian::write_u32(into, value);
}

#[inline]
fn unpack_u32(from: &[u8]) -> u32 {
    debug_assert!(from.len() >= 4);
    byteorder::LittleEndian::read_u32(from)
}

fn write_framed_message<A: capnp::message::Allocator>(
    buffer: &mut [u8],
    message_builder: &capnp::message::Builder<A>,
) -> std::io::Result<(usize, usize)> {
    let mut mmap_io = FramedMessageWriter::new(buffer);
    capnp::serialize::write_message(&mut mmap_io, &message_builder)?;
    let (msg_size, total_size) = mmap_io.finish();
    Ok((msg_size, total_size))
}

fn get_frame_size(buffer: &[u8]) -> Result<(usize, usize), Error> {
    let begin_size = unpack_u32(&buffer[0..FRAMING_DATA_SIZE_BYTES]) as usize;
    if buffer.len() > begin_size + (2 * FRAMING_DATA_SIZE_BYTES) {
        let offset = FRAMING_DATA_SIZE_BYTES + begin_size;
        let end_size = unpack_u32(&buffer[offset..(offset + 4)]) as usize;
        if begin_size == 0 {
            Err(Error::EOF)
        } else if begin_size == end_size && begin_size != 0 {
            Ok((begin_size, begin_size + 2 * FRAMING_DATA_SIZE_BYTES))
        } else {
            warn!(
                "Invalid frame size information: begin {} != end {}",
                begin_size, end_size
            );
            Err(Error::Data)
        }
    } else {
        warn!(
            "Invalid frame size: would exceed buffer size: size={} buffer_size={}",
            begin_size,
            buffer.len()
        );
        Err(Error::Data)
    }
}

type CapnSliceMessageReader<'a> = capnp::message::Reader<capnp::serialize::SliceSegments<'a>>;

fn read_framed_message(data: &[u8]) -> Result<FramedSliceMessage, Error> {
    let (message_size, total_size) = get_frame_size(data)?;

    let offset_from = FRAMING_DATA_SIZE_BYTES;
    let offset_to = offset_from + message_size;
    let reader = read_message(&data[offset_from..offset_to])?;

    Ok(FramedSliceMessage {
        message_size,
        total_size,
        data,
        reader,
    })
}

fn read_message(buffer: &[u8]) -> Result<CapnSliceMessageReader, Error> {
    let words = unsafe { capnp::Word::bytes_to_words(buffer) };
    let opts = capnp::message::ReaderOptions::new();
    capnp::serialize::read_message_from_words(&words, opts).map_err(|err| {
        warn!("Couldn't deserialize message reader: {:?}", err);
        Error::Data
    })
}

struct FramedMessageWriter<'a> {
    buffer: &'a mut [u8],
    count: usize,
    finished: bool,
}

impl<'a> FramedMessageWriter<'a> {
    fn new(buffer: &'a mut [u8]) -> FramedMessageWriter<'a> {
        FramedMessageWriter {
            buffer,
            count: FRAMING_DATA_SIZE_BYTES,
            finished: false,
        }
    }

    fn finish(&mut self) -> (usize, usize) {
        let message_size = self.count - FRAMING_DATA_SIZE_BYTES;
        pack_u32(message_size as u32, self.buffer);
        pack_u32(
            message_size as u32,
            &mut self.buffer[self.count..(self.count + FRAMING_DATA_SIZE_BYTES)],
        );
        self.finished = true;
        (message_size, message_size + 2 * FRAMING_DATA_SIZE_BYTES)
    }

    fn message_size(&self) -> usize {
        debug_assert!(self.finished, ".finish() needs to be called first");
        self.count - FRAMING_DATA_SIZE_BYTES
    }
}

impl<'a> Drop for FramedMessageWriter<'a> {
    fn drop(&mut self) {
        if !self.finished {
            self.finish();
        }
    }
}

impl<'a> std::io::Write for FramedMessageWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let offset_from = self.count;
        let len = buf.len();
        let offset_to = offset_from + len;

        if offset_to > self.buffer.len() {
            error!(
                "Tried to write a message that exceeded size of buffer: offset_to={} buffer_len={}",
                offset_to,
                self.buffer.len()
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Message bigger than buffer len",
            ));
        }

        self.buffer[offset_from..offset_to].copy_from_slice(buf);
        self.count += len;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir;

    #[test]
    fn test_pack_unpack_u32() {
        let mut buf = vec![0, 0, 0, 0, 0, 0];
        pack_u32(44323213, &mut buf);
        assert_eq!(44323213, unpack_u32(&buf));
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

        segment_file.grow(2000).unwrap();
        assert_eq!(segment_file.current_size, 2000);
    }

    #[test]
    fn test_write_read_block() {
        let mut data = [0u8; 1000];

        let mut message_builder = capnp::message::Builder::new_default();
        build_test_block(&mut message_builder);

        let (write_msg_size, write_total_size) =
            write_framed_message(&mut data, &message_builder).unwrap();
        let framed_message = read_framed_message(&data).unwrap();

        assert_eq!(framed_message.message_size, write_msg_size);
        assert_eq!(framed_message.total_size, write_total_size);

        let block_reader = framed_message.reader.get_root::<block::Reader>().unwrap();

        assert_eq!(block_reader.get_hash().unwrap(), "block_hash");
        assert_eq!(block_reader.get_entries().unwrap().len(), 1);
        assert_eq!(
            block_reader
                .get_entries()
                .unwrap()
                .get(0)
                .get_hash()
                .unwrap(),
            "entry_hash"
        );
    }

    #[test]
    fn test_read_invalid_block() {
        // no data found
        let data = [0u8; 1000];
        assert_eq!(read_framed_message(&data).err(), Some(Error::EOF));

        // invalid size
        let mut data = [0u8; 1000];
        pack_u32(10, &mut data);
        assert!(read_framed_message(&data).is_err());

        // overflow size
        let mut data = [0u8; 1000];
        pack_u32(10000, &mut data);
        assert!(read_framed_message(&data).is_err());
    }

    #[test]
    fn test_write_fail_not_enough_space() {
        let mut data = [0u8; 10];
        let mut message_builder = capnp::message::Builder::new_default();
        build_test_block(&mut message_builder);
        assert!(write_framed_message(&mut data, &message_builder).is_err());
    }

    // TODO: move to a bench
    #[test]
    fn test_bench_write_read_sequential() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let segment_path = dir.path().join("segment");
        let mut segment_file = SegmentFile::open(&segment_path, 1024 * 1024 * 1024).unwrap();

        info!("Writing into: {:?}", segment_file.path);
        let nb_blocks = 10_000;

        let begin = std::time::Instant::now();
        let mut next_offset = 0;
        for i in 0..nb_blocks {
            let mut message_builder = capnp::message::Builder::new_default();
            build_test_block(&mut message_builder);
            let (written_size, total_size) =
                write_framed_message(&mut segment_file.mmap[next_offset..], &message_builder)
                    .unwrap();
            next_offset += total_size;
        }
        info!("Elapsed writes: {:?}", begin.elapsed());

        let begin = std::time::Instant::now();
        let mut next_offset = 0;
        for i in 0..nb_blocks {
            let framed_message = read_framed_message(&segment_file.mmap[next_offset..]).unwrap();
            let block_reader = framed_message.reader.get_root::<block::Reader>().unwrap();
            let _ = block_reader.get_hash();

            next_offset += framed_message.total_size;
        }
        info!(
            "Elapsed read individual: {:?} Size={}",
            begin.elapsed(),
            next_offset
        );

        let begin = std::time::Instant::now();
        let iterator = FramedMessageIterator::new(&segment_file.mmap);
        assert_eq!(iterator.count(), nb_blocks);
        info!("Elapsed read iterator: {:?}", begin.elapsed());
    }

    #[test]
    fn test_framed_message_iterator() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let segment_path = dir.path().join("segment");
        let mut segment_file = SegmentFile::open(&segment_path, 1024 * 1024 * 1024).unwrap();

        let mut next_offset = 0;
        for i in 0..1000 {
            let mut message_builder = capnp::message::Builder::new_default();
            build_test_block(&mut message_builder);
            let (_msg_size, total_size) =
                write_framed_message(&mut segment_file.mmap[next_offset..], &message_builder)
                    .unwrap();
            next_offset += total_size;
        }

        // simple forward iteration
        let mut iterator = FramedMessageIterator::new(&segment_file.mmap);
        let mut count = 0;
        let mut last_offset = 0;
        for message in iterator.by_ref() {
            assert!(last_offset == 0 || message.offset > last_offset);
            last_offset = message.offset;
            count += 1;
        }
        assert_eq!(iterator.last_error, None);

        // make sure we can deserialize
        let message = FramedMessageIterator::new(&segment_file.mmap)
            .take(1)
            .last()
            .unwrap();
        assert_eq!(message.offset, 0);
        let block_reader = message
            .framed_message
            .reader
            .get_root::<block::Reader>()
            .unwrap();
        assert_eq!(block_reader.get_hash().unwrap(), "block_hash");

        // invalid data should have an error
        let mut data = [0u8; 1000];
        pack_u32(10, &mut data);
        let mut iterator = FramedMessageIterator::new(&data);
        assert_eq!(iterator.by_ref().count(), 0);
        assert!(iterator.last_error.is_some());
    }

    fn build_test_block<A: capnp::message::Allocator>(
        message_builder: &mut capnp::message::Builder<A>,
    ) {
        let mut block = message_builder.init_root::<block::Builder>();
        block.set_hash("block_hash");
        let mut entries = block.init_entries(1);
        {
            let mut entry = entries.reborrow().get(0);
            entry.set_hash("entry_hash");
        }
    }
}
