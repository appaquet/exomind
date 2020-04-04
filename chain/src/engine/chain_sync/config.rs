use crate::block::{BlockHeight, BlockOffset};
use crate::engine::request_tracker;

/// Chain synchronizer's configuration
#[derive(Copy, Clone, Debug)]
pub struct ChainSyncConfig {
    /// Config for requests timing tracker
    pub request_tracker: request_tracker::RequestTrackerConfig,

    /// Maximum number of synchronization failures before considering a node
    /// offsync
    pub meta_sync_max_failures: usize,

    /// Number of headers to always include at beginning of a headers sync
    /// request
    pub headers_sync_begin_count: BlockOffset,

    /// Number of headers to always include at end of a headers sync request
    pub headers_sync_end_count: BlockOffset,

    /// Number of sampled headers to include between begin and end headers of a
    /// headers sync request
    pub headers_sync_sampled_count: BlockOffset,

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
            headers_sync_begin_count: 5,
            headers_sync_end_count: 5,
            headers_sync_sampled_count: 10,
            blocks_max_send_size: 50 * 1024,
            max_leader_common_block_height_delta: 5,
        }
    }
}
