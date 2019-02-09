use exocore_common::range;

use exocore_common::data_chain_capnp::{block, block_signatures};
use exocore_common::serialization::msg;
use exocore_common::serialization::msg::FramedTypedMessage;

// TODO: Move to common
type BlockOffset = u64;
type BlockSize = u32;

pub mod directory;

pub trait Store {
    fn write_block<B, S>(&mut self, block: &B, block_signatures: &S) -> Result<BlockOffset, Error>
    where
        B: msg::FramedTypedMessage<block::Owned>,
        S: msg::FramedTypedMessage<block_signatures::Owned>;

    fn available_segments(&self) -> Vec<range::Range<BlockOffset>>;

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
    Serialization(msg::Error),
    Integrity,
    SegmentFull,
    OutOfBound,
    IO,
}

pub struct StoredBlock<'a> {
    block: msg::FramedSliceTypedMessage<'a, block::Owned>,
    signatures: msg::FramedSliceTypedMessage<'a, block_signatures::Owned>,
}

impl<'a> StoredBlock<'a> {
    #[inline]
    pub fn total_size(&self) -> usize {
        self.block.frame_size() + self.signatures.frame_size()
    }

    #[inline]
    pub fn get_offset(&self) -> Result<BlockOffset, msg::Error> {
        let block_reader = self.block.get_typed_reader()?;
        Ok(block_reader.get_offset())
    }

    #[inline]
    pub fn next_offset(&self) -> Result<BlockOffset, msg::Error> {
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
