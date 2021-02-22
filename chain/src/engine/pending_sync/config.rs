use crate::{block::BlockHeight, engine::request_tracker};

/// Pending synchronizer configuration
#[derive(Copy, Clone, Debug)]
pub struct PendingSyncConfig {
    pub max_operations_per_range: u32,

    pub request_tracker_config: request_tracker::RequestTrackerConfig,

    /// Related to `CommitManagerConfig`.`operations_cleanup_after_block_depth`.
    /// This indicates how many blocks after the last cleaned up block we should
    /// include by default when doing sync requests, so that we don't
    /// request for operations that may have been cleaned up on other nodes.

    /// The `CommitManager` does cleanup at interval, and sets the last block
    /// that got cleaned in the `SyncState` up from the `PendingStore`
    /// because it was committed for more than `CommitManagerConfig`.
    /// `operations_cleanup_after_block_depth` of depth.

    /// This value is added to the `SyncState` last cleanup block depth to make
    /// sure we don't ask or include operations that got cleaned up.
    pub operations_depth_after_cleanup: BlockHeight,
}

impl Default for PendingSyncConfig {
    fn default() -> Self {
        PendingSyncConfig {
            max_operations_per_range: 30,
            request_tracker_config: request_tracker::RequestTrackerConfig::default(),
            operations_depth_after_cleanup: 2,
        }
    }
}
