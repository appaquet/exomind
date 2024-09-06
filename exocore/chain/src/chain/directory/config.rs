/// Configuration for directory based chain persistence.
#[derive(Copy, Clone, Debug)]
pub struct DirectoryChainStoreConfig {
    /// Segment over allocation, in bytes. When resizing a segment, this
    /// amount of bytes will be allocated over the required len to prevent
    /// remapping the segment too often.
    pub segment_over_allocate_size: u64,

    /// Maximum size in bytes per segment. This is a soft limit since the last
    /// block could overflow that maximum. This should be small enough so
    /// that a few segments can fit in allocable virtual space on 32b
    /// systems. See `segment_max_open` for maximum concurrently opened
    /// segments.
    pub segment_max_size: u64,

    /// Maximum number of segments concurrently mmap. On 64b systems, where
    /// virtual memory isn't a problem, this can be high. But on 32b
    /// systems, one should aim to have maximum ~1-2gb of concurrently mmap
    /// segments. See `segment_max_size` for maximum size per segment.
    pub segment_max_open_mmap: usize,

    /// Maximum number of operations to keep in memory in the operation index
    /// before flushing into a sorted file.
    pub operation_index_max_memory_items: usize,
}

impl Default for DirectoryChainStoreConfig {
    fn default() -> Self {
        DirectoryChainStoreConfig {
            segment_over_allocate_size: 20 * 1024 * 1024, // 20mb
            segment_max_size: 200 * 1024 * 1024,          // 200mb
            segment_max_open_mmap: 20,
            operation_index_max_memory_items: 10000,
        }
    }
}

impl From<exocore_protos::core::ChainConfig> for DirectoryChainStoreConfig {
    fn from(proto: exocore_protos::core::ChainConfig) -> Self {
        let mut config = DirectoryChainStoreConfig::default();

        if let Some(val) = proto.segment_max_size {
            config.segment_max_size = val;
        }

        if let Some(val) = proto.segment_max_open_mmap {
            config.segment_max_open_mmap = val as usize;
        }

        config
    }
}
