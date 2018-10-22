use super::*;

use exocore_common::simple_store::json_disk_store::JsonDiskStore;

use std::fs::File;
use std::path::PathBuf;

// TODO: mmaped files ?
// TODO: Segments

pub trait Persistence {
    // TODO: Should use Async IO
}

struct DirectoryPersistence {
    directory: PathBuf,
    opened_file: Vec<SegmentFile>,
}

impl DirectoryPersistence {
    fn create(directory: PathBuf) -> DirectoryPersistence {
        unimplemented!()
    }

    fn open(directory: PathBuf) -> DirectoryPersistence {
        // TODO: Check if exists
        unimplemented!()
    }

    fn write_block(&self, block: NewBlock) -> usize {
        unimplemented!()
    }
}

impl Persistence for DirectoryPersistence {
    // TODO:
}

struct SegmentFile {
    block_offset: BlockOffset,
    file: File,
}

#[cfg(test)]
mod tests {
    use super::*;

}
