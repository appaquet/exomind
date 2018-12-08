// TODO: WAL stored on disk, rotated
// TODO: Or in memory, with spilling to disk?

use std::path::PathBuf;

use exocore_common::security::signature::Signature;

mod persistence;

pub struct PendingsStore {
    directory: PathBuf,
    // transactions:
}

impl PendingsStore {
    pub fn new(_bla: usize) -> PendingsStore {
        // TODO: Path of wal
        unimplemented!()
    }
}

struct PendingEntry {
    signatures: Vec<Signature>,
}

#[cfg(test)]
mod test {
    #[test]
    fn test_store() {}
}
