use super::{ChainSyncConfig, CommitManagerConfig, PendingSyncConfig};
use std::time::Duration;

/// Chain engine's configuration
#[derive(Clone)]
pub struct EngineConfig {
    pub chain_sync_config: ChainSyncConfig,
    pub pending_sync_config: PendingSyncConfig,
    pub commit_manager_config: CommitManagerConfig,
    pub manager_timer_interval: Duration,
    pub events_stream_buffer_size: usize,
    pub to_transport_channel_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            chain_sync_config: ChainSyncConfig::default(),
            pending_sync_config: PendingSyncConfig::default(),
            commit_manager_config: CommitManagerConfig::default(),
            manager_timer_interval: Duration::from_secs(1),
            events_stream_buffer_size: 1000,
            to_transport_channel_size: 3000,
        }
    }
}
