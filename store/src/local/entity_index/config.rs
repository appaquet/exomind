use crate::local::mutation_index::MutationIndexConfig;
use exocore_chain::block::BlockHeight;

/// Configuration of the entities index
#[derive(Clone, Copy, Debug)]
pub struct EntityIndexConfig {
    /// What is the minimum depth that a block needs to be the chain to be
    /// indexed. This is required to lower the odds that we are going to
    /// revert the block if our local chain forked.
    ///
    /// `CommitManagerConfig`.`operations_cleanup_after_block_depth`
    pub chain_index_min_depth: BlockHeight,

    /// If specified, prevent indexing every new block on each commit.
    /// Operations will be kept in pending index for a bit longer and
    /// preventing the costly chain index modification.
    pub chain_index_depth_leeway: BlockHeight,

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
            chain_index_depth_leeway: 10,
            pending_index_config: MutationIndexConfig::default(),
            chain_index_config: MutationIndexConfig::default(),
            chain_index_in_memory: false,
        }
    }
}
