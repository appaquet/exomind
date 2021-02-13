use std::time::Duration;

use exocore_protos::core::NodeStoreConfig;

/// Configuration for `Store`.
#[derive(Clone, Copy)]
pub struct StoreConfig {
    /// Size of the channel of queries to be executed.
    pub query_channel_size: usize,

    /// Maximum number fo queries to execute in parallel.
    pub query_parallelism: usize,

    /// Size of the result channel of each watched query.
    pub handle_watch_query_channel_size: usize,

    /// Maximum number of events from chain engine to batch together if more are
    /// available.
    pub chain_events_batch_size: usize,

    /// Timeout for mutations that were awaiting for entities to be returned.
    pub mutation_tracker_timeout: Duration,

    /// How often the garbage collection process will run.
    ///
    /// Since garbage collection doesn't happen on the whole index, but only on
    /// entities that got flagged during search, it is better to run more
    /// often than less. `GarbageCollectorConfig::queue_size` can be tweaked
    /// to control rate of collection.
    pub garbage_collect_interval: Duration,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            query_channel_size: 1000,
            query_parallelism: 4,
            handle_watch_query_channel_size: 100,
            chain_events_batch_size: 50,
            mutation_tracker_timeout: Duration::from_secs(5),
            garbage_collect_interval: Duration::from_secs(33),
        }
    }
}

impl From<NodeStoreConfig> for StoreConfig {
    fn from(proto: NodeStoreConfig) -> Self {
        let mut config = StoreConfig::default();

        if let Some(v) = proto.query_parallelism {
            config.query_parallelism = v as usize;
        }

        if let Some(index) = &proto.index {
            if let Some(gc) = &index.garbage_collector {
                if let Some(v) = gc.run_interval_secs {
                    config.garbage_collect_interval = Duration::from_secs(v as u64);
                }
            }
        }

        config
    }
}
