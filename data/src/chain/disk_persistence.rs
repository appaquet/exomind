use super::*;

use std::path::PathBuf;

// TODO: mmaped files ?
// TODO: Segments

struct DiskPersistence {}

impl DiskPersistence {
    fn create(directory: PathBuf) -> DiskPersistence {
        unimplemented!()
    }

    fn open(directory: PathBuf) -> DiskPersistence {
        // TODO: Check if exists
        unimplemented!()
    }
}

impl ChainPersistence for DiskPersistence {}
