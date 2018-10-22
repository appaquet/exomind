use super::*;

use exocore_common::simple_store::json_disk_store::JsonDiskStore;

use std::fs::File;
use std::path::PathBuf;

// TODO: mmaped files ?
// TODO: Segments

pub trait Persistence {
    // TODO: Should use Async IO
}

struct DiskPersistence {
    directory: PathBuf,
    opened_file: Vec<SegmentFile>,
}

impl DiskPersistence {
    fn create(directory: PathBuf) -> DiskPersistence {
        unimplemented!()
    }

    fn open(directory: PathBuf) -> DiskPersistence {
        // TODO: Check if exists
        unimplemented!()
    }
}

impl Persistence for DiskPersistence {}

struct SegmentFile {
    block_offset: BlockOffset,
    file: File,
}

#[cfg(test)]
mod tests {}
