use super::ChainSyncError;
use crate::block::{Block, BlockHeight, BlockOffset, BlockSignaturesSize};
use crate::chain;
use crate::{engine::EngineError, ChainSyncConfig};
use chain::Segments;
use exocore_core::framing::FrameReader;
use exocore_core::protos::generated::data_chain_capnp::{block_header, block_partial_header};

/// Block metadata used for comparison between local and remote stores.
#[derive(Debug)]
pub struct BlockMetadata {
    pub offset: BlockOffset,
    pub height: BlockHeight,
    pub hash: Vec<u8>,
    pub previous_offset: BlockOffset,
    pub previous_hash: Vec<u8>,

    pub block_size: u32,
    pub operations_size: u32,
    pub signatures_size: BlockSignaturesSize,
}

impl BlockMetadata {
    pub fn from_store<CS: chain::ChainStore>(
        store: &CS,
        from_offset: Option<BlockOffset>,
        to_offset: Option<BlockOffset>,
        config: &ChainSyncConfig,
    ) -> Result<Vec<BlockMetadata>, EngineError> {
        let range_segments = store.segments().filter_in_range(from_offset, to_offset);

        if range_segments.is_empty() {
            Ok(Vec::new())
        } else if range_segments.len() > config.metadata_sync_segments_boundaries_threshold {
            debug!(
                "Using segments boundaries for faster metadata sync request from {:?} to {:?} with {} segments",
                from_offset,
                to_offset,
                range_segments.len(),
            );
            BlockMetadata::from_segment_boundaries(
                store,
                range_segments,
                from_offset,
                to_offset,
                config,
            )
        } else {
            BlockMetadata::from_sampled_chain_slice(store, from_offset, to_offset, config)
        }
    }

    /// Returns a collection of `BlockPartialHeader` by using segments' first
    /// block instead of sampling the chain like in
    /// `from_sampled_chain_slice`, which is much faster.
    pub fn from_segment_boundaries<CS: chain::ChainStore>(
        store: &CS,
        range_segments: Segments,
        from_offset: Option<BlockOffset>,
        to_offset: Option<BlockOffset>,
        config: &ChainSyncConfig,
    ) -> Result<Vec<BlockMetadata>, EngineError> {
        let mut blocks_meta = Vec::new();

        // include N blocks from the requested from offset
        let from_offset = from_offset.unwrap_or(0);
        let blocks_iter = store
            .blocks_iter(from_offset)?
            .take(config.metadata_sync_begin_count);
        for block in blocks_iter {
            let block_meta = BlockMetadata::from_stored_block(block)?;
            blocks_meta.push(block_meta);
        }

        // include first block of each segment
        for segment in range_segments.into_iter() {
            let block_offset = segment.range.start;

            // make sure that we don't add a block that is before last added block
            if let Some(last_added_offset) = blocks_meta.last().map(|h| h.offset) {
                if block_offset <= last_added_offset {
                    continue;
                }
            }

            let block = store.get_block(block_offset)?;
            let block_meta = BlockMetadata::from_stored_block(block)?;
            blocks_meta.push(block_meta);
        }

        // include N blocks before the requested to offset
        let last_chain_block = store.get_last_block()?;
        let to_offset = to_offset
            .or_else(|| last_chain_block.map(|b| b.next_offset()))
            .ok_or_else(|| {
                ChainSyncError::InvalidSyncRequest(format!(
                    "Couldn't find last to offset block {:?}",
                    to_offset
                ))
            })?;
        let blocks_iter = store.blocks_iter_reverse(to_offset);
        if let Ok(blocks_iter) = blocks_iter {
            let mut blocks = blocks_iter
                .take(config.metadata_sync_end_count)
                .collect::<Vec<_>>();
            blocks.reverse();
            for block in blocks {
                let block_meta = BlockMetadata::from_stored_block(block)?;

                // make sure that we don't add a block that is before last added block
                if let Some(last_added_offset) = blocks_meta.last().map(|h| h.offset) {
                    if block_meta.offset <= last_added_offset {
                        continue;
                    }
                }

                blocks_meta.push(block_meta);
            }
        }

        Ok(blocks_meta)
    }

    /// Samples the local chain and returns a collection of `BlockPartialHeader`
    /// at different positions in the asked range.
    ///
    /// `from_offset` and `to_offset` are best efforts and fallback to begin/end
    /// of chain if they don't exist. `begin_count` and `end_count` are
    /// number of meta to include without sampling from beginning and end
    /// of range. `sampled_count` is the approximate number of blocks meta to
    /// return, excluding the `begin_count` and `end_count`
    pub fn from_sampled_chain_slice<CS: chain::ChainStore>(
        store: &CS,
        from_offset: Option<BlockOffset>,
        to_offset: Option<BlockOffset>,
        config: &ChainSyncConfig,
    ) -> Result<Vec<BlockMetadata>, EngineError> {
        let mut blocks_meta = Vec::new();

        let begin_count = config.metadata_sync_begin_count;
        let end_count = config.metadata_sync_end_count;
        let sampled_count = config.metadata_sync_sampled_count;

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

        let last_block_reader: block_header::Reader = last_block.header.get_reader()?;
        let last_block_height = last_block_reader.get_height();

        let from_offset = from_offset.unwrap_or(0);
        let mut blocks_iter = store
            .blocks_iter(from_offset)
            .or_else(|_| store.blocks_iter(0))?
            .peekable();

        let first_block = blocks_iter.peek().ok_or_else(|| {
            ChainSyncError::Other("Expected a first block since ranges were not empty".to_string())
        })?;
        let first_block_reader: block_header::Reader = first_block.header.get_reader()?;
        let first_block_height = first_block_reader.get_height();

        let range_blocks_count = (last_block_height - first_block_height) as usize;
        let range_blocks_skip = (range_blocks_count / sampled_count).max(1);

        // from which block do we include all blocks metadata so that we always include
        // last `end_count` blocks
        let range_blocks_lasts = range_blocks_count
            .checked_sub(end_count)
            .unwrap_or(range_blocks_count);

        for (blocks_count, current_block) in blocks_iter
            .enumerate()
            .take(range_blocks_count as usize + 1)
        {
            // we always include metadata if the block is within the first `begin_count` or
            // in the last `end_count` otherwise, we include if it falls within
            // sampling condition
            if blocks_count < begin_count
                || blocks_count > range_blocks_lasts
                || blocks_count % range_blocks_skip == 0
            {
                let block_partial_header = BlockMetadata::from_stored_block(current_block)?;
                blocks_meta.push(block_partial_header);
            }
        }

        Ok(blocks_meta)
    }

    pub fn from_stored_block<B: Block>(stored_block: B) -> Result<BlockMetadata, EngineError> {
        let block_header_reader: block_header::Reader = stored_block.header().get_reader()?;
        let block_signature = stored_block.header().inner().inner().multihash_bytes();

        Ok(BlockMetadata {
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

    pub fn from_block_partial_metadata_reader(
        block_partial_header_reader: block_partial_header::Reader,
    ) -> Result<BlockMetadata, EngineError> {
        Ok(BlockMetadata {
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
    use super::*;
    use crate::block::BlockOffset;
    use crate::chain::ChainStore;
    use crate::{
        engine::testing::{EngineTestCluster, EngineTestClusterConfig},
        DirectoryChainStoreConfig,
    };

    #[test]
    fn from_segment_boundaries() -> anyhow::Result<()> {
        let config = EngineTestClusterConfig {
            nodes_count: 1,
            chain_config: DirectoryChainStoreConfig {
                segment_max_size: 5_000,
                segment_over_allocate_size: 5_100,
                ..Default::default()
            },
        };
        let mut cluster = EngineTestCluster::new_from_config(config);
        cluster.chain_generate_dummy(0, 100, 3424);

        let store = &cluster.chains[0];
        let segments = store.segments();

        let assert_metadata = |from, to, config, expected_heights| {
            let segments = segments.clone().filter_in_range(from, to);
            let blocks =
                BlockMetadata::from_segment_boundaries(store, segments, from, to, &config).unwrap();
            assert_eq!(
                blocks.iter().map(|b| { b.height }).collect::<Vec<_>>(),
                expected_heights
            );
        };

        {
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 2,
                metadata_sync_end_count: 2,
                ..Default::default()
            };
            assert_metadata(
                None,
                None,
                config,
                vec![0, 1, 13, 26, 39, 52, 65, 78, 91, 98, 99],
            );
        }

        {
            // invalid to offset should still work
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 5,
                metadata_sync_end_count: 5,
                ..Default::default()
            };
            assert_metadata(None, Some(1337), config, vec![0, 1, 2, 3, 4]);
        }

        {
            // segments block shouldn't be included if they were already included in
            // begin/end
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 5,
                metadata_sync_end_count: 5,
                ..Default::default()
            };
            assert_metadata(None, Some(321), config, vec![0, 1, 2, 3, 4]);
        }

        {
            // segments block shouldn't be included if they were already included in
            // begin/end
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 5,
                metadata_sync_end_count: 5,
                ..Default::default()
            };
            assert_metadata(
                None,
                Some(23425),
                config,
                vec![0, 1, 2, 3, 4, 13, 26, 39, 52, 65],
            );
        }

        Ok(())
    }

    #[test]
    fn chain_sample_block_partial_headers() -> anyhow::Result<()> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_generate_dummy(0, 100, 3424);

        let offsets: Vec<BlockOffset> = cluster.chains[0]
            .blocks_iter(0)?
            .map(|b| b.offset)
            .collect();

        {
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 2,
                metadata_sync_end_count: 2,
                metadata_sync_sampled_count: 10,
                ..Default::default()
            };
            let headers =
                BlockMetadata::from_sampled_chain_slice(&cluster.chains[0], None, None, &config)?;
            assert_eq!(
                headers.iter().map(|b| b.height).collect::<Vec<_>>(),
                vec![0, 1, 9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 98, 99]
            );
        }

        {
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 0,
                metadata_sync_end_count: 0,
                metadata_sync_sampled_count: 1,
                ..Default::default()
            };
            let headers =
                BlockMetadata::from_sampled_chain_slice(&cluster.chains[0], None, None, &config)?;
            assert_eq!(
                headers.iter().map(|b| b.height).collect::<Vec<_>>(),
                vec![0, 99]
            );
        }

        {
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 5,
                metadata_sync_end_count: 5,
                metadata_sync_sampled_count: 10,
                ..Default::default()
            };
            let headers = BlockMetadata::from_sampled_chain_slice(
                &cluster.chains[0],
                Some(offsets[10]),
                None,
                &config,
            )?;
            assert_eq!(
                headers.iter().map(|b| b.height).collect::<Vec<_>>(),
                vec![
                    10, 11, 12, 13, 14, 18, 26, 34, 42, 50, 58, 66, 74, 82, 90, 95, 96, 97, 98, 99
                ]
            );
        }

        {
            let config = ChainSyncConfig {
                metadata_sync_begin_count: 2,
                metadata_sync_end_count: 2,
                metadata_sync_sampled_count: 5,
                ..Default::default()
            };
            let headers = BlockMetadata::from_sampled_chain_slice(
                &cluster.chains[0],
                Some(offsets[10]),
                Some(offsets[50]),
                &config,
            )?;
            assert_eq!(
                headers.iter().map(|b| b.height).collect::<Vec<_>>(),
                vec![10, 11, 18, 26, 34, 42, 49, 50]
            );
        }

        Ok(())
    }
}
