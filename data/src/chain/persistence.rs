use super::*;

use std;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use byteorder;
use byteorder::ByteOrder;
use capnp;
use memmap;
use memmap::{MmapMut, MmapOptions};

// TODO: Segments
// TODO: AVOID COPIES
// TODO: Since we pre-allocate mmap, we need to know if an incoming block will fit
// TODO: We will have to pre-allocate size in big chunks (10MB ? 50MB ?)
// TODO: directory segment + mmap + write header
// TODO: Use ScratchSpace to reuse memory buffer for writes

const FRAMING_DATA_SIZE_BYTES: usize = 4;

pub trait Persistence {}

struct PersistedSegment {
    id: SegmentID,
    first_offset: BlockOffset,
    last_offset: BlockOffset,
    size: SegmentSize,
    frozen: bool, // if frozen, means that we have an last offset
}

// TODO: Move to capnp
struct PersistedBlock {
    offset: BlockOffset,
    size: BlockSize,
    hash: Hash,
    entries: Vec<Entry>,
    size_end: BlockSize, // Allow backward iteration
}

struct DirectoryPersistence {
    directory: PathBuf,
    opened_file: Vec<DirectorySegment>,
}

impl DirectoryPersistence {
    fn create(directory: PathBuf) -> DirectoryPersistence {
        unimplemented!()
    }

    fn open(directory: PathBuf) -> DirectoryPersistence {
        // TODO: Check if exists
        unimplemented!()
    }

    fn list_segments(&self) -> &[PersistedSegment] {
        unimplemented!()
    }

    fn freeze_segment(&mut self, segment_id: SegmentID) {

    }

    fn create_segment(&mut self, segment_id: SegmentID) {
        unimplemented!()
    }

    fn write_block(
        &mut self,
        segment_id: SegmentID,
        block_offset: BlockOffset,
        block: Block,
    ) -> (BlockOffset, BlockSize) {
        unimplemented!()
    }
}

impl Persistence for DirectoryPersistence {
    // TODO:
}

struct DirectorySegment {
    meta: PersistedSegment,
    path: PathBuf,
    file: Option<File>,
    mmap: Option<memmap::MmapMut>,
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
    Ok(mmap_io.finish())
}

fn get_frame_size(buffer: &[u8]) -> Result<usize, Error> {
    let begin_size = unpack_u32(&buffer[0..FRAMING_DATA_SIZE_BYTES]) as usize;
    if buffer.len() > begin_size + (2 * FRAMING_DATA_SIZE_BYTES) {
        let offset = FRAMING_DATA_SIZE_BYTES + begin_size;
        let end_size = unpack_u32(&buffer[offset..(offset + 4)]) as usize;
        if begin_size == end_size {
            Ok(begin_size)
        } else {
            warn!(
                "Invalid frame size information: begin {} != end {}",
                begin_size, end_size
            );
            Err(Error::Persistence)
        }
    } else {
        warn!(
            "Invalid frame size: would exceed buffer size: size={} buffer_size={}",
            begin_size,
            buffer.len()
        );
        Err(Error::Persistence)
    }
}

fn read_framed_message(
    buffer: &[u8],
) -> Result<
    (
        usize,
        capnp::message::Reader<capnp::serialize::SliceSegments>,
    ),
    Error,
> {
    let frame_size = get_frame_size(buffer)?;

    let offset_from = FRAMING_DATA_SIZE_BYTES;
    let offset_to = offset_from + frame_size;
    let words = unsafe { capnp::Word::bytes_to_words(&buffer[offset_from..offset_to]) };

    let opts = capnp::message::ReaderOptions::new();
    let message = capnp::serialize::read_message_from_words(&words, opts).map_err(|err| {
        warn!("Couldn't deserialize message: {:?}", err);
        Error::Persistence
    })?;

    Ok((frame_size, message))
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
                offset_to, self.buffer.len()
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
    use chain_block_capnp;
    use chain_block_capnp::{address_book, block, block_entry, person};

    #[test]
    fn test_pack_unpack_u32() {
        let mut buf = vec![0, 0, 0, 0, 0, 0];
        pack_u32(44323213, &mut buf);
        assert_eq!(44323213, unpack_u32(&buf));
    }

    #[test]
    fn test_write_read_block() {
        let mut data = [0u8; 1000];

        let mut message_builder = capnp::message::Builder::new_default();
        build_test_block(&mut message_builder);

        let (write_msg_size, total_size) =
            write_framed_message(&mut data, &message_builder).unwrap();
        let (read_msg_size, message_reader) = read_framed_message(&data).unwrap();

        assert_eq!(read_msg_size, write_msg_size);

        let block_reader = message_reader.get_root::<block::Reader>().unwrap();
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
        assert!(read_framed_message(&data).is_err());

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

    fn build_test_block<A: capnp::message::Allocator>(message_builder: &mut capnp::message::Builder<A>) {
        let mut block = message_builder.init_root::<block::Builder>();
        block.set_hash("block_hash");
        let mut entries = block.init_entries(1);
        {
            let mut entry = entries.reborrow().get(0);
            entry.set_hash("entry_hash");
        }
    }

    struct TestFile {
        dir: tempdir::TempDir,
        file: File,
        mmap: MmapMut,
    }

    fn get_test_file() -> TestFile {
        let dir = tempdir::TempDir::new("test").unwrap();
        let path = dir.path().join("test.data");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .unwrap();
        file.set_len(1000).unwrap();
        let mmap = unsafe { memmap::MmapOptions::new().map_mut(&file).unwrap() };
        TestFile { dir, file, mmap }
    }

}
