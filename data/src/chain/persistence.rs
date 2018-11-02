use super::*;

use exocore_common::simple_store::json_disk_store::JsonDiskStore;

use std;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use byteorder;
use byteorder::ByteOrder;
use memmap;

// TODO: Segments
// TODO: AVOID COPIES
// TODO: Since we pre-allocate mmap, we need to know if an incoming block will fit

pub trait Persistence {}

struct PersistedSegment {
    id: SegmentID,
    first_offset: BlockOffset,
    last_offset: BlockOffset,
    size: SegmentSize,
    frozen: bool, // if frozen, means that we have an last offset
}


// TODO: Move to flatbuffer
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

    fn freeze_segment(&mut self, segment_id: SegmentID) {}

    fn create_segment(&mut self, segment_id: SegmentID) {
        unimplemented!()
    }

    fn write_block(&mut self, segment_id: SegmentID, block_offset: BlockOffset, block: Block) -> (BlockOffset, BlockSize) {
        unimplemented!()
    }
}

impl Persistence for DirectoryPersistence {
    // TODO:
}

struct DirectorySegment {
    meta: PersistedSegment,
    file: File,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir;
    use exocore_common;
    use std;
    use flatbuffers;

    use self::chain_schema_generated::chain;

    #[test]
    fn pack_unpack_u32() {
        let mut buf = vec![0, 0, 0, 0, 0, 0];
        pack_u32(44231323213, &mut buf);
        assert_eq!(44231323213, unpack_u32(&buf));
    }

    #[test]
    fn test_flatbuffers() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let path = dir.path().join("test.data");
        let file = OpenOptions::new().read(true).write(true).create(true).open(&path).unwrap();
        file.set_len(1000).unwrap();
        // TODO: directory segment + mmap + write header
        let mut mfile = unsafe {
            memmap::MmapOptions::new().map_mut(&file).unwrap()
        };

        {
            let block = chain::get_size_prefixed_root_as_block_entry(&mfile);
            error!("BLABLA {}", block.offset());
        }

        let mut fb_builder = flatbuffers::FlatBufferBuilder::new();
        let mut block_entry_offset = {
            fb_builder.start_vector::<i8>(6);
            fb_builder.push(12i8);
            fb_builder.push(12i8);
            fb_builder.push(12i8);
            fb_builder.push(12i8);
            fb_builder.push(12i8);
            fb_builder.push(12i8);
            let vec_offset = fb_builder.end_vector(6);

            let mut block_entry_builder = chain::BlockEntryBuilder::new(&mut fb_builder);
            block_entry_builder.add_offset(10);
            block_entry_builder.add_data(vec_offset);
            block_entry_builder.finish()
        };
//        fb_builder.finish(block_entry_offset, None);
        fb_builder.finish_size_prefixed(block_entry_offset, None);
        let buf = fb_builder.finished_data();

        let len = buf.len();
        info!("Len is {}", len);
        mfile[0..len].copy_from_slice(&buf[0..len]);

        info!("{:?}", buf);

        let block = chain::get_size_prefixed_root_as_block_entry(&mfile);
//        let block = chain::get_root_as_block_entry(&mfile);
        info!("{:?}", block.offset());
        info!("{:?}", block.data());
    }


    #[test]
    fn test_capnp() {
        use utils;
        utils::setup_logging();
        use capnp;
        use test_capnp;
        use test_capnp::{address_book, person};
        use capnp::message::ReaderSegments;

        let dir = tempdir::TempDir::new("test").unwrap();
        let path = dir.path().join("test.data");
        let file = OpenOptions::new().read(true).write(true).create(true).open(&path).unwrap();
        file.set_len(1000).unwrap(); // TODO: We will have to pre-allocate size in big chunks (10MB ? 50MB ?)
        // TODO: directory segment + mmap + write header
        let mut mfile = unsafe {
            memmap::MmapOptions::new().map_mut(&file).unwrap()
        };


        let mut message = capnp::message::Builder::new_default();

        {
            let address_book = message.init_root::<address_book::Builder>();
            let mut people = address_book.init_people(2);
            {
                let mut alice = people.reborrow().get(0);
                alice.set_id(123);
                alice.set_name("Alice");
                alice.set_email("alice@example.com");
                {
                    let mut alice_phones = alice.reborrow().init_phones(1);
                    alice_phones.reborrow().get(0).set_number("555-1212");
                    alice_phones.reborrow().get(0).set_type(person::phone_number::Type::Mobile);
                }
                alice.get_employment().set_school("MIT");
            }
        }

        //        capnp::message::
        use std::io::prelude::*;
        use std::io::BufWriter;
        use std::fs::File;

        let (message_size, written_size) = {
            let mut mmap_io = FramedMessageWriter::new(&mut mfile);
            capnp::serialize::write_message(&mut mmap_io, &message);
            mmap_io.finish()
        };

        info!("message_size={} written_size={}", message_size, written_size);

        let (read_size, message_reader) = {
            let opts = capnp::message::ReaderOptions::new();
            let words = unsafe { capnp::Word::bytes_to_words(&mfile[4..196]) }; // Has to be exact size...
            let read = capnp::serialize::read_message_from_words(&words, opts).unwrap();
//            let mut mmap_io = FramedMessageReader::new(&mut mfile);
//            assert!(mmap_io.is_valid_size());
//            (mmap_io.message_size(), capnp::serialize::read_message(&mut mmap_io, opts).unwrap())
                (0, read)
        };

//        assert_eq!(read_size, message_size);

        let address_book = message_reader.get_root::<address_book::Reader>().unwrap();

        for person in address_book.get_people().unwrap().iter() {
            info!("{}: {}", person.get_name().unwrap(), person.get_email().unwrap());
        }
    }


    // TODO: We should have magic bytes
    const FRAME_SIZE_BYTES: usize = 4;

    struct FramedMessageWriter<'a> {
        mmap: &'a mut [u8],
        count: usize,
        finished: bool,
    }

    impl<'a> FramedMessageWriter<'a> {
        fn new(mmap: &'a mut [u8]) -> FramedMessageWriter<'a> {
            FramedMessageWriter {
                mmap,
                count: FRAME_SIZE_BYTES,
                finished: false,
            }
        }

        fn finish(&mut self) -> (u32, u32) {
            let message_size = (self.count - FRAME_SIZE_BYTES) as u32;
            pack_u32(message_size, self.mmap);
            pack_u32(message_size, &mut self.mmap[self.count..(self.count + FRAME_SIZE_BYTES)]);
            self.finished = true;
            (message_size, message_size + 2 * FRAME_SIZE_BYTES as u32)
        }

        fn message_size(&self) -> u32 {
            debug_assert!(self.finished, ".finish() needs to be called first");
            (self.count - FRAME_SIZE_BYTES) as u32
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
            let offset = self.count;
            let len = buf.len();
            self.mmap[offset..(offset + len)].copy_from_slice(buf);
            self.count += len;
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    struct FramedMessageReader<'a> {
        data: &'a mut [u8],
        count: usize,
    }

    impl<'a> FramedMessageReader<'a> {
        fn new(mmap: &'a mut [u8]) -> FramedMessageReader<'a> {
            FramedMessageReader {
                data: mmap,
                count: FRAME_SIZE_BYTES,
            }
        }

        fn message_size(&self) -> u32 {
            unpack_u32(&self.data[0..FRAME_SIZE_BYTES])
        }

        // Make sure that the size at beginning of message is same as at the end
        fn is_valid_size(&self) -> bool {
            let begin_size = unpack_u32(&self.data[0..FRAME_SIZE_BYTES]) as usize;
            if self.data.len() > begin_size + (2 * FRAME_SIZE_BYTES) {
                let offset = (FRAME_SIZE_BYTES + begin_size);
                let end_size = unpack_u32(&self.data[offset..(offset + 4)]) as usize;
                begin_size == end_size
            } else {
                false
            }
        }
    }

    impl<'a> std::io::Read for FramedMessageReader<'a> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
            let offset = self.count;
            let len = buf.len();
            buf.copy_from_slice(&self.data[offset..(offset + len)]);
            self.count += len;
            Ok(len)
        }
    }
}
