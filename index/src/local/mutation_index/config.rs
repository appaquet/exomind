/// Trait index configuration
#[derive(Clone, Copy, Debug)]
pub struct MutationIndexConfig {
    pub indexer_num_threads: Option<usize>,
    pub indexer_heap_size_bytes: usize,
    pub iterator_page_size: u32,
}

impl Default for MutationIndexConfig {
    fn default() -> Self {
        MutationIndexConfig {
            indexer_num_threads: None,
            indexer_heap_size_bytes: 50_000_000,
            iterator_page_size: 50,
        }
    }
}
