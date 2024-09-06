use exocore_protos::generated::exocore_core::MutationIndexConfig as ProtoMutationIndexConfig;
/// Trait index configuration
#[derive(Clone, Copy, Debug)]
pub struct MutationIndexConfig {
    /// Number of indexing threads.
    pub indexer_num_threads: Option<usize>,

    /// Maximum heap size of each indexing thread.
    pub indexer_heap_size_bytes: usize,

    /// Page size of results iterator.
    pub iterator_page_size: u32,

    /// Maximum number of pages returned by results iterator.
    pub iterator_max_pages: usize,

    /// Size of the entity mutations cache in bytes.
    pub entity_mutations_cache_size: usize,

    pub dynamic_reference_fields: u32,
    pub dynamic_string_fields: u32,
    pub dynamic_text_fields: u32,
    pub dynamic_i64_fields: u32,
    pub dynamic_i64_sortable_fields: u32,
    pub dynamic_u64_fields: u32,
    pub dynamic_u64_sortable_fields: u32,
}

impl Default for MutationIndexConfig {
    fn default() -> Self {
        MutationIndexConfig {
            indexer_num_threads: Some(2),
            indexer_heap_size_bytes: 30_000_000,
            iterator_page_size: 1000,
            iterator_max_pages: 5,
            entity_mutations_cache_size: 5000,

            dynamic_reference_fields: 10,
            dynamic_string_fields: 10,
            dynamic_text_fields: 10,
            dynamic_i64_fields: 10,
            dynamic_i64_sortable_fields: 10,
            dynamic_u64_fields: 10,
            dynamic_u64_sortable_fields: 10,
        }
    }
}

impl From<ProtoMutationIndexConfig> for MutationIndexConfig {
    fn from(proto: ProtoMutationIndexConfig) -> Self {
        let mut config = MutationIndexConfig::default();

        if let Some(v) = proto.indexer_num_threads {
            config.indexer_num_threads = Some(v as usize);
        }

        if let Some(v) = proto.indexer_heap_size_bytes {
            config.indexer_heap_size_bytes = v as usize;
        }

        if let Some(v) = proto.entity_mutations_cache_size {
            config.entity_mutations_cache_size = v as usize;
        }

        config
    }
}
