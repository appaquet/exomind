use exocore_common::security::signature::Signature;
use std::sync::Arc;

use flatbuffers;
mod chain_schema_generated;

pub mod persistence;
pub use self::persistence::Persistence;

type BlockOffset = u64;
type EntryOffset = u64;

// TODO: Serialization using flatbuffers
pub struct Chain<P: Persistence> {
    persistence: P,
    next_block_offset: BlockOffset,
    // TODO: Link to segments
}

impl<P: Persistence> Chain<P> {
    pub fn new(persistence: P) -> Chain<P> {
        // TODO: Load segments
        // TODO: Get next block offset

        Chain {
            persistence,
            next_block_offset: 0,
        }
    }

    pub fn segments(&self) {}

    pub fn blocks_iter(
        &self,
        from_offset: Option<BlockOffset>,
        to_offset: Option<BlockOffset>,
    ) -> &dyn Iterator<Item = Block> {
        unimplemented!()
    }

    pub fn get_block(&self, offset: BlockOffset) -> &Block {
        // TODO: Find segment in which it is
        // TODO: Find block
        unimplemented!()
    }

    pub fn block_entries_iter(
        &self,
        block: &Block,
        with_data: bool,
    ) -> &dyn Iterator<Item = Entry> {
        unimplemented!()
    }

    pub fn entry(&self, entry: EntryOffset, with_data: bool) -> Entry {
        unimplemented!()
    }
}

pub struct Segment {
    offset: BlockOffset,
    previous_block: Option<BlockOffset>,
    to_offset: BlockOffset,
    frozen: bool,
}

pub struct Block {
    offset: BlockOffset, // abs segment
    size: u32,
    previous_block_offset: BlockOffset,
    previous_block_hash: String,
    hash: String,
    signatures: Vec<Signature>,
    // TODO: Link to entries (with_data=bool)
}

pub struct Entry {
    offset: EntryOffset,
    entry_type: EntryType,
    data: Option<Arc<EntryData>>,
}

pub struct NewEntry {
    entry_type: EntryType,
    data: EntryData,
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
