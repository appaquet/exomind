use exocore_common::security::hash::Hash;
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

    pub fn write_entry_try(&mut self, entry: NewEntry) -> Result<TriedNewEntry, Error> {
        unimplemented!()
    }

    pub fn write_entry_commit(&mut self, entry: TriedNewEntry) -> Result<(), Error> {
        // TODO: Make sure we didn't have any other entry meanwhile...
        unimplemented!()
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

pub enum Error {
    ConcurrentWrite,
}

pub struct Segment {
    offset: BlockOffset,
    previous_block: Option<BlockOffset>,
    to_offset: BlockOffset,
    frozen: bool,
}

pub struct Block {
    offset: BlockOffset, // abs position segment
    size: u32,
    previous_block_offset: BlockOffset,
    previous_block_hash: Hash,
    hash: Hash,
    signatures: Vec<Signature>,
    // TODO: Link to entries (with_data=bool)
}

pub struct NewBlock {
    entries: Vec<NewEntry>,
}

pub struct TriedNewEntry {
    // TODO: Entry that hasn't been written yet
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

// TODO: Should aim for zero-copy
// TODO: Not yet loaded from disk ???
// TODO: mmaped ?
pub struct EntryData {
    data: Vec<u8>,
}

#[cfg(test)]
mod test {
    #[test]
    fn test_chain() {}
}
