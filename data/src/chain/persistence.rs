use super::*;

use exocore_common::simple_store::json_disk_store::JsonDiskStore;

use std::fs::File;
use std::path::PathBuf;

use memmap;

// TODO: Segments
// TODO: AVOID COPIES

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


#[cfg(test)]
mod tests {
    use super::*;
    use tempdir;
    use exocore_common;
    use std;
    use flatbuffers;

    use self::chain_schema_generated::chain;

    #[test]
    fn test_segment_create() {
        exocore_common::logging::setup();

        let dir = tempdir::TempDir::new("test").unwrap();
        let path = dir.path().join("test.data");
        info!("{}", dir.path().is_dir());
        info!("Path: {:?}", path);

        let file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&path).unwrap();
        file.set_len(1000).unwrap();
        // TODO: directory segment + mmap + write header

        let mut mfile = unsafe {
            memmap::MmapOptions::new().map_mut(&file).unwrap()
        };

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
}
