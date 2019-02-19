use std::ops::Range;

use exocore_common::data_chain_capnp::{block, block_signatures};
use exocore_common::serialization::framed;
use exocore_common::serialization::framed::TypedFrame;

type BlockOffset = u64;

pub mod directory;

pub trait Store {
    fn write_block<B, S>(&mut self, block: &B, block_signatures: &S) -> Result<BlockOffset, Error>
    where
        B: framed::TypedFrame<block::Owned>,
        S: framed::TypedFrame<block_signatures::Owned>;

    fn available_segments(&self) -> Vec<Range<BlockOffset>>;

    fn block_iter(&self, from_offset: BlockOffset) -> Result<StoredBlockIterator, Error>;

    fn block_iter_reverse(
        &self,
        from_next_offset: BlockOffset,
    ) -> Result<StoredBlockIterator, Error>;

    fn get_block(&self, offset: BlockOffset) -> Result<StoredBlock, Error>;

    fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<StoredBlock, Error>;

    fn truncate_from_offset(&mut self, block_offset: BlockOffset) -> Result<(), Error>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    UnexpectedState,
    Serialization(framed::Error),
    Integrity,
    SegmentFull,
    OutOfBound,
    IO,
}

pub struct StoredBlock<'a> {
    block: framed::TypedSliceFrame<'a, block::Owned>,
    signatures: framed::TypedSliceFrame<'a, block_signatures::Owned>,
}

impl<'a> StoredBlock<'a> {
    #[inline]
    pub fn total_size(&self) -> usize {
        self.block.frame_size() + self.signatures.frame_size()
    }

    #[inline]
    pub fn get_offset(&self) -> Result<BlockOffset, framed::Error> {
        let block_reader = self.block.get_typed_reader()?;
        Ok(block_reader.get_offset())
    }

    #[inline]
    pub fn next_offset(&self) -> Result<BlockOffset, framed::Error> {
        let block_reader = self.block.get_typed_reader()?;
        let offset = block_reader.get_offset();
        Ok(offset + (self.block.frame_size() + self.signatures.frame_size()) as BlockOffset)
    }
}

type StoredBlockIterator<'pers> = Box<dyn Iterator<Item = StoredBlock<'pers>> + 'pers>;

pub enum EntryType {
    Data,
    Truncate,
    Duplicate,
}

pub struct EntryData {
    data: Vec<u8>,
}
