use super::ChainSyncError;
use crate::block::{Block, BlockHeight, BlockOffset, BlockSignaturesSize};
use crate::chain;
use crate::engine::EngineError;
use exocore_core::framing::FrameReader;
use exocore_core::protos::generated::data_chain_capnp::{block_header, block_partial_header};

/// Block metadata used for comparison between local and remote stores.
#[derive(Debug)]
pub struct BlockMeta {
    pub offset: BlockOffset,
    pub height: BlockHeight,
    pub hash: Vec<u8>,
    pub previous_offset: BlockOffset,
    pub previous_hash: Vec<u8>,

    pub block_size: u32,
    pub operations_size: u32,
    pub signatures_size: BlockSignaturesSize,
}

impl BlockMeta {
    /// Samples the local chain and returns a collection of `BlockPartialHeader`
    /// at different position in the asked range.
    ///
    /// `from_offset` and `to_offset` are best efforts and fallback to begin/end
    /// of chain if they don't exist. `begin_count` and `end_count` are
    /// number of headers to include without sampling from beginning and end
    /// of range. `sampled_count` is the approximate number of headers to
    /// return, excluding the `begin_count` and `end_count`
    pub fn from_sampled_chain_slice<CS: chain::ChainStore>(
        store: &CS,
        from_offset: BlockOffset,
        to_offset: Option<BlockOffset>,
        begin_count: BlockOffset,
        end_count: BlockOffset,
        sampled_count: BlockOffset,
    ) -> Result<Vec<BlockMeta>, EngineError> {
        let mut headers = Vec::new();

        let segments_range = store.segments();
        if segments_range.is_empty() {
            return Ok(headers);
        }

        let last_block = match to_offset {
            Some(offset) => store.get_block(offset).map(Some).or_else(|_| {
                warn!(
                    "Given to offset {} didn't exist. Falling back to last block of chain",
                    offset
                );
                store.get_last_block()
            }),
            None => store.get_last_block(),
        }?
        .ok_or_else(|| {
            ChainSyncError::Other("Expected a last block since ranges were not empty".to_string())
        })?;

        let last_block_header_reader: block_header::Reader = last_block.header.get_reader()?;
        let last_block_height = last_block_header_reader.get_height();

        let mut blocks_iter = store
            .blocks_iter(from_offset)
            .or_else(|_| store.blocks_iter(0))?
            .peekable();

        let first_block = blocks_iter.peek().ok_or_else(|| {
            ChainSyncError::Other("Expected a first block since ranges were not empty".to_string())
        })?;
        let first_block_header_reader: block_header::Reader = first_block.header.get_reader()?;
        let first_block_height = first_block_header_reader.get_height();

        let range_blocks_count = last_block_height - first_block_height;
        let range_blocks_skip = (range_blocks_count / sampled_count).max(1);

        // from which block do we include all headers so that we always include last
        // `end_count` blocks
        let range_blocks_lasts = range_blocks_count
            .checked_sub(end_count)
            .unwrap_or(range_blocks_count);

        for (blocks_count, current_block) in blocks_iter
            .enumerate()
            .take(range_blocks_count as usize + 1)
        {
            // we always include headers if the block is within the first `begin_count` or
            // in the last `end_count` otherwise, we include if it falls within
            // sampling condition
            let blocks_count = blocks_count as BlockOffset;
            if blocks_count < begin_count
                || blocks_count > range_blocks_lasts
                || blocks_count % range_blocks_skip == 0
            {
                let block_partial_header = BlockMeta::from_stored_block(current_block)?;
                headers.push(block_partial_header);
            }
        }

        Ok(headers)
    }

    pub fn from_stored_block<B: Block>(stored_block: B) -> Result<BlockMeta, EngineError> {
        let block_header_reader: block_header::Reader = stored_block.header().get_reader()?;
        let block_signature = stored_block.header().inner().inner().multihash_bytes();

        Ok(BlockMeta {
            offset: stored_block.offset(),
            height: block_header_reader.get_height(),
            hash: block_signature.to_vec(),
            previous_offset: block_header_reader.get_previous_offset(),
            previous_hash: block_header_reader.get_previous_hash()?.to_vec(),

            block_size: stored_block.header().whole_data_size() as u32,
            operations_size: block_header_reader.get_operations_size(),
            signatures_size: block_header_reader.get_signatures_size(),
        })
    }

    pub fn from_block_partial_header_reader(
        block_partial_header_reader: block_partial_header::Reader,
    ) -> Result<BlockMeta, EngineError> {
        Ok(BlockMeta {
            offset: block_partial_header_reader.get_offset(),
            height: block_partial_header_reader.get_height(),
            hash: block_partial_header_reader.get_block_hash()?.to_vec(),
            previous_offset: block_partial_header_reader.get_previous_offset(),
            previous_hash: block_partial_header_reader.get_previous_hash()?.to_vec(),
            block_size: block_partial_header_reader.get_block_size(),
            operations_size: block_partial_header_reader.get_operations_size(),
            signatures_size: block_partial_header_reader.get_signatures_size(),
        })
    }

    #[inline]
    pub fn next_offset(&self) -> BlockOffset {
        self.offset
            + BlockOffset::from(self.block_size)
            + BlockOffset::from(self.operations_size)
            + BlockOffset::from(self.signatures_size)
    }

    pub fn copy_into_builder(&self, builder: &mut block_partial_header::Builder) {
        builder.set_offset(self.offset);
        builder.set_height(self.height);
        builder.set_block_hash(&self.hash);
        builder.set_previous_offset(self.previous_offset);
        builder.set_previous_hash(&self.previous_hash);
        builder.set_block_size(self.block_size);
        builder.set_operations_size(self.operations_size);
        builder.set_signatures_size(self.signatures_size);
    }
}

#[cfg(test)]
mod tests {
    use crate::block::BlockOffset;
    use crate::chain::ChainStore;
    use crate::engine::chain_sync::block::BlockMeta;
    use crate::engine::testing::EngineTestCluster;

    #[test]
    fn test_chain_sample_block_partial_headers() -> anyhow::Result<()> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_generate_dummy(0, 100, 3424);

        let offsets: Vec<BlockOffset> = cluster.chains[0]
            .blocks_iter(0)?
            .map(|b| b.offset)
            .collect();

        let headers = BlockMeta::from_sampled_chain_slice(&cluster.chains[0], 0, None, 2, 2, 10)?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![0, 1, 9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 98, 99]
        );

        let headers = BlockMeta::from_sampled_chain_slice(&cluster.chains[0], 0, None, 0, 0, 1)?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![0, 99]
        );

        let headers =
            BlockMeta::from_sampled_chain_slice(&cluster.chains[0], offsets[10], None, 5, 5, 10)?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![10, 11, 12, 13, 14, 18, 26, 34, 42, 50, 58, 66, 74, 82, 90, 95, 96, 97, 98, 99]
        );

        let headers = BlockMeta::from_sampled_chain_slice(
            &cluster.chains[0],
            offsets[10],
            Some(offsets[50]),
            2,
            2,
            5,
        )?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![10, 11, 18, 26, 34, 42, 49, 50]
        );

        Ok(())
    }
}
