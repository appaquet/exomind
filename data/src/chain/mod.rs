use std::ops::Range;

use crate::block::{Block, BlockOffset, BlockRef};
use crate::operation::OperationId;

#[cfg(feature = "directory_chain")]
pub mod directory;

pub mod error;
pub use error::Error;

///
/// Persistence for the chain
///
pub trait ChainStore: Send + Sync + 'static {
    fn segments(&self) -> Vec<Segment>;

    fn write_block<B: Block>(&mut self, block: &B) -> Result<BlockOffset, Error>;

    fn blocks_iter(&self, from_offset: BlockOffset) -> Result<StoredBlockIterator, Error>;

    fn blocks_iter_reverse(
        &self,
        from_next_offset: BlockOffset,
    ) -> Result<StoredBlockIterator, Error>;

    fn get_block(&self, offset: BlockOffset) -> Result<BlockRef, Error>;

    fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<BlockRef, Error>;

    fn get_last_block(&self) -> Result<Option<BlockRef>, Error>;

    fn get_block_by_operation_id(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<BlockRef>, Error>;

    fn truncate_from_offset(&mut self, offset: BlockOffset) -> Result<(), Error>;
}

///
/// Segment of the chain with a specified offsets range, in bytes.
///
/// The upper range is exclusive. You can use `get_block_from_next_offset` to get the last block
/// of the segment.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub range: Range<BlockOffset>,
}

///
/// Iterator over stored blocks.
///
type StoredBlockIterator<'pers> = Box<dyn Iterator<Item = BlockRef<'pers>> + 'pers>;
