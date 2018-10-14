use exocore_common::security::signature::Signature;
use std::sync::Arc;

use flatbuffers;
mod chain_schema_generated;
mod disk_persistence;

// TODO: Serialization using flatbuffers
pub struct Chain<P: ChainPersistence> {
    persistence: P,
    // TODO: Link to segments
}

impl<P: ChainPersistence> Chain<P> {
    pub fn new(persistence: P) -> Chain<P> {
        Chain { persistence }
    }

    pub fn segments(&self) {}

    pub fn get_block(&self, offset: u64) {
        // TODO: Find segment in which it is
        // TODO: Find block
    }

    pub fn get_block_entries(&self, block: Block, with_data: bool) -> &dyn Iterator<Item = Entry> {
        unimplemented!()
    }
}

pub trait ChainPersistence {
    // TODO: Should use Async IO
}

pub struct Segment {
    from_offset: u64,
    to_offset: u64,
    frozen: bool,
}

pub struct Block {
    offset: u64, // abs segment
    size: u64,
    previous_block_offset: u64,
    previous_block_hash: String,
    hash: String,
    signatures: Vec<Signature>,
    // TODO: Link to entries (with_data=bool)
}

pub struct Entry {
    offset: u64,
    entry_type: EntryType,
    data: Option<Arc<EntryData>>,
}

pub enum EntryType {
    OpNewSegment,
    OpCopy,
    OpTruncate, // All nodes agrees to start chain from new section
    Data,
}

pub struct EntryData {
    // TODO: Should aim for zero-copy
// TODO: Not yet loaded from disk ???
// TODO: mmaped ?
}

#[cfg(test)]
mod test {
    #[test]
    fn test_chain() {}
}
