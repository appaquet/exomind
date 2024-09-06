use exocore_chain::block::BlockHeight;
use exocore_protos::generated::exocore_core::EntityIndexConfig as ProtoEntityIndexConfig;

use super::gc::GarbageCollectorConfig;
use crate::local::mutation_index::MutationIndexConfig;

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

    /// Boosts results of a full-text search when results originate from the
    /// pending store index (i.e. recently modified). This is useful as scores
    /// from the pending index may be less relevant because of the lack of other
    /// results (i.e. BM25 bias). A value above 1.0 boosts.
    pub pending_index_boost: f32,

    /// Configuration for the persisted traits index that are in the chain
    pub chain_index_config: MutationIndexConfig,

    /// For tests, allow not hitting the disk
    pub chain_index_in_memory: bool,

    /// Configuration entity / mutation index garbage collector process.
    pub garbage_collector: GarbageCollectorConfig,
}

impl Default for EntityIndexConfig {
    fn default() -> Self {
        EntityIndexConfig {
            chain_index_min_depth: 3,
            chain_index_depth_leeway: 10,
            pending_index_config: MutationIndexConfig::default(),
            pending_index_boost: 5.0,
            chain_index_config: MutationIndexConfig::default(),
            chain_index_in_memory: false,
            garbage_collector: GarbageCollectorConfig::default(),
        }
    }
}

impl From<ProtoEntityIndexConfig> for EntityIndexConfig {
    fn from(proto: ProtoEntityIndexConfig) -> Self {
        let mut config = EntityIndexConfig {
            pending_index_config: proto.pending_index.map(|m| m.into()).unwrap_or_default(),
            chain_index_config: proto.chain_index.map(|m| m.into()).unwrap_or_default(),
            ..EntityIndexConfig::default()
        };

        if let Some(v) = proto.chain_index_min_depth {
            config.chain_index_min_depth = v;
        }

        if let Some(v) = proto.chain_index_depth_leeway {
            config.chain_index_depth_leeway = v;
        }

        if let Some(gc) = proto.garbage_collector {
            config.garbage_collector = gc.into();
        }

        config
    }
}
