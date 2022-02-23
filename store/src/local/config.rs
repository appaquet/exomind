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

    /// Specifies the interval at which new blocks in the chain get indexed.
    /// New blocks may not necessarily get immediately indexed if they don't
    /// fall in the interval of `chain_index_min_depth` and
    /// `chain_index_depth_leeway`.
    ///
    /// Indexation can also be prevented if user queries were recently executed
    /// (see `chain_index_deferred_query_secs`)
    ///
    /// If '0' is specified, deferred indexation is disabled and blocks are
    /// indexed when the chain layer emits events.
    pub chain_index_deferred_interval: Option<Duration>,

    /// Specifies the minimum interval to wait before indexing chain blocks
    /// after receiving a user query. It prevents potential slow downs caused
    /// by chain indexation if a user query get executed frequently.
    pub chain_index_deferred_query_interval: Duration,

    /// Specifies the maximum interval for which indexation may be blocked by
    /// incoming user queries.
    pub chain_index_deferred_max_interval: Duration,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            query_channel_size: 1000,
            query_parallelism: 4,
            handle_watch_query_channel_size: 100,
            chain_events_batch_size: 50,
            mutation_tracker_timeout: Duration::from_secs(5),
            garbage_collect_interval: Duration::from_secs(13),
            chain_index_deferred_interval: Some(Duration::from_secs(5)),
            chain_index_deferred_query_interval: Duration::from_secs(15),
            chain_index_deferred_max_interval: Duration::from_secs(5 * 60),
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

            if let Some(secs) = index.chain_index_deferred_interval_secs {
                config.chain_index_deferred_interval = Some(Duration::from_secs(secs));
            }

            if let Some(secs) = index.chain_index_deferred_query_secs {
                config.chain_index_deferred_query_interval = Duration::from_secs(secs);
            }

            if let Some(secs) = index.chain_index_deferred_max_secs {
                config.chain_index_deferred_max_interval = Duration::from_secs(secs);
            }
        }

        config
    }
}
