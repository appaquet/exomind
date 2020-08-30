use exocore_core::cell::NodeId;
use exocore_core::time::Clock;

use crate::block::{Block, BlockHeight};
use crate::chain::ChainStore;
use crate::engine::error::EngineError;
use crate::engine::request_tracker::RequestTracker;

use super::BlockMetadata;
use super::ChainSyncConfig;

/// Synchronization information about a remote node
pub struct NodeSyncInfo {
    pub config: ChainSyncConfig,
    pub node_id: NodeId,

    pub(super) last_common_block: Option<BlockMetadata>,
    pub last_common_is_known: bool,
    pub(super) last_known_block: Option<BlockMetadata>,

    pub request_tracker: RequestTracker,
}

impl NodeSyncInfo {
    pub fn new(node_id: NodeId, config: ChainSyncConfig, clock: Clock) -> NodeSyncInfo {
        let request_tracker_config = config.request_tracker;

        NodeSyncInfo {
            config,
            node_id,

            last_common_block: None,
            last_common_is_known: false,
            last_known_block: None,

            request_tracker: RequestTracker::new_with_clock(clock, request_tracker_config),
        }
    }

    pub fn check_status(&mut self) -> NodeStatus {
        let response_failures_count = self.request_tracker.response_failure_count();
        let is_failed = response_failures_count >= self.config.meta_sync_max_failures;

        if self.last_common_is_known && !is_failed {
            NodeStatus::Synchronized
        } else {
            if self.last_common_is_known {
                debug!("Lost node {} synchronization status", self.node_id);
                self.last_common_is_known = false;
            }

            NodeStatus::Unknown
        }
    }

    pub fn status(&self) -> NodeStatus {
        if self.last_common_is_known {
            NodeStatus::Synchronized
        } else {
            NodeStatus::Unknown
        }
    }

    pub fn chain_fully_downloaded(&self) -> bool {
        let last_known_offset = self.last_known_block.as_ref().map(|b| b.offset);
        let last_common_offset = self.last_common_block.as_ref().map(|b| b.offset);
        self.last_known_block.is_some() && last_known_offset == last_common_offset
    }

    /// Returns delta in block height between the last known block of the node
    /// and the last common block that we have.
    pub fn common_blocks_height_delta(&self) -> Option<BlockHeight> {
        match (&self.last_common_block, &self.last_known_block) {
            (Some(common), Some(known)) => Some(known.height - common.height),
            _ => None,
        }
    }

    /// Check if what we know of the remote node's chain is considered
    /// divergent. A divergent chain is a forked chain, in which we have a
    /// common ancestor, but different subsequent blocks.
    pub fn is_divergent<CS: ChainStore>(&self, local_store: &CS) -> Result<bool, EngineError> {
        if let Some(last_common_block) = &self.last_common_block {
            let last_known_block = if let Some(last_known_block) = self.last_known_block.as_ref() {
                last_known_block
            } else {
                return Ok(false);
            };

            let last_local_block = local_store.get_last_block()?.ok_or_else(|| {
                EngineError::Other(String::from(
                    "Expected a common block to be in stored since it had previously been",
                ))
            })?;
            let last_local_height = last_local_block.get_height()?;

            // if we have a block after common, and that the remote has one too, we are
            // divergent
            Ok(last_local_height > last_common_block.height
                && last_known_block.height > last_common_block.height)
        } else {
            // if we don't have any common block and we have at least one block in local
            // chain, and that remote node is not empty, we have diverged from
            // it
            let last_local_block = local_store.get_last_block()?;
            Ok(last_local_block.is_some() && self.last_known_block.is_some())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeStatus {
    Unknown,
    Synchronized,
}
