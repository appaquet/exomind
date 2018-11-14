// TODO: WAL stored on disk, rotated
// TODO: Or in memory, with spilling to disk?

use exocore_common::security::signature::Signature;

use std::path::PathBuf;

pub struct PendingsStore {
    directory: PathBuf,
    // transactions:
}

impl PendingsStore {
    pub fn new() -> PendingsStore {
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
