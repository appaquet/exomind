/// Trait index configuration
#[derive(Clone, Copy, Debug)]
pub struct MutationIndexConfig {
    pub indexer_num_threads: Option<usize>,
    pub indexer_heap_size_bytes: usize,
    pub iterator_page_size: u32,
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
            indexer_num_threads: Some(1),
            indexer_heap_size_bytes: 5_000_000,
            iterator_page_size: 50,
            entity_mutations_cache_size: 2000,

            dynamic_reference_fields: 5,
            dynamic_string_fields: 5,
            dynamic_text_fields: 5,
            dynamic_i64_fields: 2,
            dynamic_i64_sortable_fields: 2,
            dynamic_u64_fields: 2,
            dynamic_u64_sortable_fields: 2,
        }
    }
}
