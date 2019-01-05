use exocore_common::range;

use exocore_common::chain_block_capnp::{block, block_signatures};
use exocore_common::serialization::msg;
use exocore_common::serialization::msg::FramedTypedMessage;

pub use self::persistence::Persistence;

pub mod persistence;

type BlockOffset = u64;
type BlockSize = u32;

pub struct Chain<P>
where
    P: Persistence,
{
    persistence: P,
    last_block_offset: BlockOffset,
    next_block_offset: BlockOffset,
}

impl<P> Chain<P>
where
    P: Persistence,
{
    pub fn new(persistence: P) -> Chain<P> {
        // TODO: Load segments
        // TODO: Get next block offset

        let available_segments = persistence.available_segments();
        let segments_gaps = range::get_gaps(available_segments.iter());
        if !segments_gaps.is_empty() {
            warn!("The chain contains gaps at: {:?}", segments_gaps);
        }

        Chain {
            persistence,
            last_block_offset: 0,
            next_block_offset: 0, // TODO: !
        }
    }

    pub fn write_block<B, S>(&mut self, _block: &B, _signatures: &S) -> Result<(), Error>
    where
        B: msg::FramedTypedMessage<block::Owned>,
        S: msg::FramedTypedMessage<block_signatures::Owned>,
    {
        unimplemented!()
    }

    pub fn blocks_iter(&self, from_offset: Option<BlockOffset>) -> StoredBlockIterator {
        //        self.persistence.block_iter(from_offset.unwrap_or(0))
        unimplemented!()
    }

    pub fn blocks_iter_reverse(&self, _from_offset: Option<BlockOffset>) -> StoredBlockIterator {
        unimplemented!()
    }

    pub fn get_block(&self, offset: BlockOffset) -> Result<StoredBlock, Error> {
        // TODO: Find segment in which it is
        // TODO: Find block
        let _block = self.persistence.get_block(offset)?;

        unimplemented!()
    }

    pub fn get_last_block(&self) -> Result<(), Error> {
        // TODO: Get next offset
        // TODO: Get last block's hash

        unimplemented!()
    }

    pub fn verify(&mut self) -> Result<(), Error> {
        unimplemented!()
    }
}

pub struct StoredBlock<'a> {
    block: msg::FramedSliceTypedMessage<'a, block::Owned>,
    signatures: msg::FramedSliceTypedMessage<'a, block_signatures::Owned>,
}

impl<'a> StoredBlock<'a> {
    #[inline]
    pub fn total_size(&self) -> usize {
        self.block.data_size() + self.signatures.data_size()
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
        Ok(offset + (self.block.data_size() + self.signatures.data_size()) as BlockOffset)
    }
}

type StoredBlockIterator<'pers> = Box<dyn Iterator<Item = StoredBlock<'pers>> + 'pers>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Persistence(persistence::Error),
}

impl From<persistence::Error> for Error {
    fn from(err: persistence::Error) -> Self {
        Error::Persistence(err)
    }
}

pub enum EntryType {
    Data,
    Truncate,
    Duplicate,
}

pub struct EntryData {
    data: Vec<u8>,
}
