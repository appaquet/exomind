use std::time::Duration;

use crate::block::BlockHeight;

/// CommitManager's configuration
#[derive(Copy, Clone, Debug)]
pub struct CommitManagerConfig {
    /// How deep a block need to be before we cleanup its operations from
    /// pending store
    pub operations_cleanup_after_block_depth: BlockHeight,

    /// After how many new operations in pending store do we force a commit,
    /// even if we aren't past the commit interval
    pub commit_maximum_pending_store_count: usize,

    /// Interval at which commits are made, unless we hit
    /// `commit_maximum_pending_count`
    pub commit_maximum_interval: Duration,

    /// For how long a block proposal is considered valid after its creation
    /// This is used to prevent
    pub block_proposal_timeout: Duration,
}

impl Default for CommitManagerConfig {
    fn default() -> Self {
        CommitManagerConfig {
            operations_cleanup_after_block_depth: 6,
            commit_maximum_pending_store_count: 10,
            commit_maximum_interval: Duration::from_secs(3),
            block_proposal_timeout: Duration::from_secs(7),
        }
    }
}
