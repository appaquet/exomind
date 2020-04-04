use crate::local::mutation_index::MutationIndexConfig;
use exocore_chain::block::BlockHeight;

/// Configuration of the entities index
#[derive(Clone, Copy, Debug)]
pub struct EntityIndexConfig {
    /// When should we index a block in the chain so that odds that we aren't
    /// going to revert it are high enough. Related to
    /// `CommitManagerConfig`.`operations_cleanup_after_block_depth`
    pub chain_index_min_depth: BlockHeight,

    /// Configuration for the in-memory traits index that are in the pending
    /// store
    pub pending_index_config: MutationIndexConfig,

    /// Configuration for the persisted traits index that are in the chain
    pub chain_index_config: MutationIndexConfig,

    /// For tests, allow not hitting the disk
    pub chain_index_in_memory: bool,
}

impl Default for EntityIndexConfig {
    fn default() -> Self {
        EntityIndexConfig {
            chain_index_min_depth: 3,
            pending_index_config: MutationIndexConfig::default(),
            chain_index_config: MutationIndexConfig::default(),
            chain_index_in_memory: false,
        }
    }
}
