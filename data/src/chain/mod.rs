use crate::chain_block_capnp::{block, block_signatures};
use crate::serialize;
use crate::serialize::FramedTypedMessage;
use exocore_common::range;

pub use self::persistence::Persistence;

pub mod persistence;

const BLOCK_MSG_TYPE: u16 = 0;
const BLOCK_ENTRY_MSG_TYPE: u16 = 1;
const BLOCK_SIGNATURES_MSG_TYPE: u16 = 2;
const BLOCK_SIGNATURE_MSG_TYPE: u16 = 3;

type BlockOffset = u64;
type BlockSize = u32;

pub struct Chain<P>
where
    P: Persistence,
{
    persistence: P,
    last_block_offset: BlockOffset,
    next_block_offset: BlockOffset,
    // TODO: Link to segments
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
            next_block_offset: 0,
        }
    }

    pub fn write_block<B, S>(&mut self, _block: &B, _signatures: &S) -> Result<(), Error>
    where
        B: serialize::FramedTypedMessage<block::Owned>,
        S: serialize::FramedTypedMessage<block_signatures::Owned>,
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
    block: serialize::FramedSliceTypedMessage<'a, block::Owned>,
    signatures: serialize::FramedSliceTypedMessage<'a, block_signatures::Owned>,
}

impl<'a> StoredBlock<'a> {
    #[inline]
    pub fn total_size(&self) -> usize {
        self.block.data_size() + self.signatures.data_size()
    }

    #[inline]
    pub fn get_offset(&self) -> Result<BlockOffset, serialize::Error> {
        let block_reader = self.block.get()?;
        Ok(block_reader.get_offset())
    }

    #[inline]
    pub fn next_offset(&self) -> Result<BlockOffset, serialize::Error> {
        let block_reader = self.block.get()?;
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

impl<'a> crate::serialize::MessageType<'a> for crate::chain_block_capnp::block::Owned {
    fn message_type() -> u16 {
        1
    }
}

impl<'a> crate::serialize::MessageType<'a> for crate::chain_block_capnp::block_entry::Owned {
    fn message_type() -> u16 {
        2
    }
}

impl<'a> crate::serialize::MessageType<'a> for crate::chain_block_capnp::block_signatures::Owned {
    fn message_type() -> u16 {
        3
    }
}

impl<'a> crate::serialize::MessageType<'a> for crate::chain_block_capnp::block_signature::Owned {
    fn message_type() -> u16 {
        4
    }
}
