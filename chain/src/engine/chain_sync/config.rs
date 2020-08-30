use crate::block::BlockHeight;
use crate::engine::request_tracker;

/// Chain synchronizer's configuration
#[derive(Clone, Debug)]
pub struct ChainSyncConfig {
    /// Config for requests timing tracker
    pub request_tracker: request_tracker::RequestTrackerConfig,

    /// Maximum number of synchronization failures before considering a node
    /// offsync
    pub meta_sync_max_failures: usize,

    /// Number of blocks metadata to always include at beginning of a metadata
    /// sync request
    pub metadata_sync_begin_count: usize,

    /// Number of blocks metadata to always include at end of a metadata sync
    /// request
    pub metadata_sync_end_count: usize,

    /// Number of sampled blocks metadata to include between begin and end
    /// blocks of a metadata sync request
    pub metadata_sync_sampled_count: usize,

    /// When doing blocks metadata synchronization, if the requested range spans
    /// multiple segments, this is the threshold from which we fall into a
    /// fast synchronization mode. Instead of sampling blocks, only the
    /// first block of each segments (segments boundary) is sent preventing
    /// scanning blocks.
    pub metadata_sync_segments_boundaries_threshold: usize,

    /// Maximum number of bytes worth of blocks to send in a response
    /// This should be lower than transport maximum packet size
    pub blocks_max_send_size: usize,

    /// Maximum height in blocks that we can tolerate between our common
    /// ancestor block and its latest block. If it gets higher than this
    /// value, this means that we may have diverged and we need to
    /// re-synchronize.
    pub max_leader_common_block_height_delta: BlockHeight,
}

impl Default for ChainSyncConfig {
    fn default() -> Self {
        ChainSyncConfig {
            request_tracker: request_tracker::RequestTrackerConfig::default(),
            meta_sync_max_failures: 2,
            metadata_sync_begin_count: 5,
            metadata_sync_end_count: 5,
            metadata_sync_sampled_count: 10,
            metadata_sync_segments_boundaries_threshold: 5,
            blocks_max_send_size: 50 * 1024,
            max_leader_common_block_height_delta: 5,
        }
    }
}
