use std::ops::Range;

use crate::block;
use crate::block::{Block, BlockOffset, BlockRef};
use crate::operation::OperationId;
use exocore_common::serialization::capnp;

pub mod directory;

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

///
/// Chain related errors
///
#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Block related error: {}", _0)]
    Block(#[fail(cause)] block::Error),
    #[fail(display = "The store is in an unexpected state: {}", _0)]
    UnexpectedState(String),
    #[fail(display = "The store has an integrity problem: {}", _0)]
    Integrity(String),
    #[fail(display = "A segment has reached its full capacity")]
    SegmentFull,
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "An offset is out of the chain data: {}", _0)]
    OutOfBound(String),
    #[fail(display = "IO error of kind {:?}: {}", _0, _1)]
    IO(std::io::ErrorKind, String),
    #[fail(display = "Error in directory chain store: {:?}", _0)]
    DirectoryError(#[fail(cause)] directory::DirectoryError),
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Error::UnexpectedState(_) | Error::Integrity(_) | Error::IO(_, _) => true,
            _ => false,
        }
    }
}

impl From<block::Error> for Error {
    fn from(err: block::Error) -> Self {
        Error::Block(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

impl From<directory::DirectoryError> for Error {
    fn from(err: directory::DirectoryError) -> Self {
        Error::DirectoryError(err)
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{BlockOperations, BlockOwned};
    use crate::operation::OperationBuilder;
    use exocore_common::cell::FullCell;
    use exocore_common::framing::FrameReader;
    use exocore_common::node::LocalNode;

    #[test]
    fn test_block_create_and_read() -> Result<(), failure::Error> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node.clone());
        let genesis = BlockOwned::new_genesis(&cell)?;

        let operations = vec![
            OperationBuilder::new_entry(123, local_node.id(), b"some_data")
                .sign_and_build(&local_node)?
                .frame,
        ];
        let operations = BlockOperations::from_operations(operations.into_iter())?;

        let second_block = BlockOwned::new_with_prev_block(&cell, &genesis, 0, operations)?;

        let mut data = [0u8; 5000];
        second_block.copy_data_into(&mut data);

        let read_second_block = BlockRef::new(&data[0..second_block.total_size()])?;
        assert_eq!(
            second_block.block.whole_data(),
            read_second_block.block.whole_data()
        );
        assert_eq!(
            second_block.operations_data,
            read_second_block.operations_data
        );
        assert_eq!(
            second_block.signatures.whole_data(),
            read_second_block.signatures.whole_data()
        );

        let block_reader = second_block.block.get_reader()?;
        assert_eq!(block_reader.get_offset(), genesis.next_offset());
        assert_eq!(
            block_reader.get_signatures_size(),
            second_block.signatures.whole_data_size() as u16
        );
        assert_eq!(
            block_reader.get_operations_size(),
            second_block.operations_data.len() as u32
        );

        let signatures_reader = second_block.signatures.get_reader()?;
        assert_eq!(
            signatures_reader.get_operations_size(),
            second_block.operations_data.len() as u32
        );

        let signatures = signatures_reader.get_signatures()?;
        assert_eq!(signatures.len(), 1);

        Ok(())
    }

    #[test]
    fn test_block_operations() -> Result<(), failure::Error> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node.clone());
        let genesis = BlockOwned::new_genesis(&cell)?;

        // 0 operations
        let block = BlockOwned::new_with_prev_block(&cell, &genesis, 0, BlockOperations::empty())?;
        assert_eq!(block.operations_iter()?.count(), 0);

        // 5 operations
        let operations = (0..5)
            .map(|i| {
                OperationBuilder::new_entry(i, local_node.id(), b"op1")
                    .sign_and_build(&local_node)
                    .unwrap()
                    .frame
            })
            .collect::<Vec<_>>();

        let block_operations = BlockOperations::from_operations(operations.into_iter())?;
        let block = BlockOwned::new_with_prev_block(&cell, &genesis, 0, block_operations)?;
        assert_eq!(block.operations_iter()?.count(), 5);

        Ok(())
    }
}
