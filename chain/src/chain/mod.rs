use std::ops::Range;

use crate::block::{Block, BlockOffset, BlockRef};
use crate::operation::OperationId;

#[cfg(feature = "directory-chain")]
pub mod directory;

pub mod error;
pub use error::Error;

/// Persistence for the chain
pub trait ChainStore: Send + Sync + 'static {
    fn segments(&self) -> Segments;

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

/// Segment of the chain with a specified offsets range, in bytes.
///
/// The upper range is exclusive. You can use `get_block_from_next_offset` to
/// get the last block of the segment.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub range: Range<BlockOffset>,
}

/// Collection of segments of the chain.
#[derive(Clone, Debug, PartialEq)]
pub struct Segments(pub Vec<Segment>);

impl From<Vec<Segment>> for Segments {
    fn from(segments: Vec<Segment>) -> Self {
        Segments(segments)
    }
}

impl Into<Vec<Segment>> for Segments {
    fn into(self) -> Vec<Segment> {
        self.0
    }
}

impl Segments {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Segment> {
        self.0.iter()
    }

    pub fn filter_in_range(
        self,
        from_offset: Option<BlockOffset>,
        to_offset: Option<BlockOffset>,
    ) -> Segments {
        let from_offset = from_offset.unwrap_or(std::u64::MIN);
        let to_offset = to_offset.unwrap_or(std::u64::MAX);

        Segments(
            self.0
                .into_iter()
                .filter(|segment| {
                    from_offset <= segment.range.end && segment.range.start <= to_offset
                })
                .collect(),
        )
    }
}

impl std::iter::IntoIterator for Segments {
    type Item = Segment;

    type IntoIter = std::vec::IntoIter<Segment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Iterator over stored blocks.
type StoredBlockIterator<'pers> = Box<dyn Iterator<Item = BlockRef<'pers>> + 'pers>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_filter_in_range() {
        let segments = Segments(vec![
            Segment { range: 0..100 },
            Segment { range: 100..200 },
            Segment { range: 200..300 },
        ]);

        let assert_segments = |from, to, segs: Vec<BlockOffset>| {
            let matching = segments.clone().filter_in_range(from, to);
            let ids: Vec<BlockOffset> = matching.into_iter().map(|r| r.range.start / 100).collect();
            assert_eq!(segs, ids);
        };

        assert_segments(None, None, vec![0, 1, 2]);
        assert_segments(Some(0), None, vec![0, 1, 2]);
        assert_segments(None, Some(300), vec![0, 1, 2]);
        assert_segments(None, Some(299), vec![0, 1, 2]);
        assert_segments(None, Some(199), vec![0, 1]);
        assert_segments(Some(100), Some(199), vec![0, 1]);
        assert_segments(Some(101), Some(199), vec![1]);
    }
}
