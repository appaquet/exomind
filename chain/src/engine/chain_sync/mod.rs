use super::{EngineError, Event, SyncContext};
use crate::block::{Block, BlockOffset, BlockRef};
use crate::chain::ChainStore;
use capnp::traits::ToU16;
use exocore_core::capnp;
use exocore_core::cell::{Cell, CellNodeRole, CellNodes, CellNodesOwned};
use exocore_core::cell::{Node, NodeId};
use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_core::protos::generated::data_chain_capnp::block_partial_header;
use exocore_core::protos::generated::data_transport_capnp::{
    chain_sync_request, chain_sync_request::RequestedDetails, chain_sync_response,
};
use exocore_core::time::Clock;
use std::collections::HashMap;

pub use config::ChainSyncConfig;
pub use error::ChainSyncError;
use node_info::{NodeStatus, NodeSyncInfo};

mod meta;
mod node_info;
use self::meta::BlockMetadata;
mod config;
mod error;
#[cfg(test)]
mod tests;

/// Synchronizes the local chain against remote nodes' chain.
///
/// It achieves synchronization in 3 stages:
///
/// 1) Gather knowledge about remote nodes' chain metadata (last block, last
/// common block).    During this stage, the status is 'Unknown'
///
/// 2) Once we have knowledge of the majority of node, we find the leader node
/// with the longest chain.    Taking the longest chain is valid, since this
/// node could not have made progress without a majority    of nodes signing the
/// latest blocks.
///
/// 3) Download the missing blocks from the leader, starting from the latest
/// common block, which    is our common ancestor.
///    During this stage, the status is 'Downloading'
///
/// 4) Once fully downloaded, we keep asking for other nodes' metadata to make
/// sure we progress and    leadership hasn't changed.
pub(super) struct ChainSynchronizer<CS: ChainStore> {
    config: ChainSyncConfig,
    cell: Cell,
    nodes_info: HashMap<NodeId, NodeSyncInfo>,
    status: Status,
    leader: Option<NodeId>,
    clock: Clock,
    phantom: std::marker::PhantomData<CS>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Unknown,
    Downloading,
    Synchronized,
}

impl<CS: ChainStore> ChainSynchronizer<CS> {
    pub fn new(config: ChainSyncConfig, cell: Cell, clock: Clock) -> ChainSynchronizer<CS> {
        ChainSynchronizer {
            config,
            cell,
            status: Status::Unknown,
            nodes_info: HashMap::new(),
            leader: None,
            clock,
            phantom: std::marker::PhantomData,
        }
    }

    /// Called at interval to make progress on the synchronization. Depending on
    /// the current synchronization status, we could be asking for more
    /// details about remote nodes' chain, or could be asking for blocks
    /// data from the lead node.
    pub fn tick(&mut self, sync_context: &mut SyncContext, store: &CS) -> Result<(), EngineError> {
        let status_start = self.status;
        let nodes = self.cell.nodes().to_owned();

        let (nb_nodes_metadata_sync, nb_nodes) = self.check_nodes_status(&nodes);
        let majority_nodes_metadata_sync = nodes.is_quorum(
            usize::from(nb_nodes_metadata_sync),
            Some(CellNodeRole::Chain),
        );

        let last_block_offset = store.get_last_block()?.map(|b| b.offset);
        debug!(
            "Sync tick begins. current_status={:?} last_block_offset={:?} nb_nodes={} nb_nodes_metadata_sync={}",
            self.status, last_block_offset, nb_nodes, nb_nodes_metadata_sync
        );

        // make sure we still have majority of nodes metadata
        if self.status == Status::Synchronized && !majority_nodes_metadata_sync {
            info!("Lost majority of nodes being synchronized. Changing status to unknown");
            self.status = Status::Unknown;
            self.leader = None;
        }

        // make sure we are still in sync with the leader
        if self.status == Status::Synchronized && !self.im_leader() {
            if let Some(leader_node_id) = self.leader.clone() {
                let leader_node_disp = nodes
                    .get(&leader_node_id)
                    .map(|cn| cn.node().to_string())
                    .unwrap_or_else(|| String::from("NOT_FOUND"));

                let leader_node_info = self.get_or_create_node_info_mut(&leader_node_id);
                let common_block_delta = leader_node_info.common_blocks_height_delta().unwrap_or(0);
                let sync_status = leader_node_info.status();

                let lost_leadership = if sync_status != NodeStatus::Synchronized {
                    info!(
                        "Node {} lost leadership status because it isn't sync anymore",
                        leader_node_disp,
                    );
                    true
                } else if common_block_delta > self.config.max_leader_common_block_height_delta {
                    info!("Node {} lost leadership status because of common block height delta is too high (height {} > height {})", leader_node_disp, common_block_delta, self.config.max_leader_common_block_height_delta);
                    true
                } else {
                    false
                };

                if lost_leadership {
                    self.leader = None;
                    self.status = Status::Unknown;
                }
            }
        }

        // if we lost synchronization, we reset request trackers to allow quicker
        // resynchronization by not waiting for request interval
        if status_start == Status::Synchronized && self.status != Status::Synchronized {
            for sync_info in self.nodes_info.values_mut() {
                sync_info.request_tracker.reset();
            }
        }

        // check if we can elect a leader and download the chain from it
        if self.status != Status::Synchronized && majority_nodes_metadata_sync {
            let mut nb_non_divergent = 1;
            let mut nb_total = 1;

            for cell_node in nodes
                .iter()
                .all_except_local()
                .filter(|cn| cn.has_role(CellNodeRole::Chain))
            {
                nb_total += 1;
                let node_info = self.get_or_create_node_info_mut(cell_node.node().id());
                if !node_info.is_divergent(store)? {
                    nb_non_divergent += 1;
                }
            }

            if nodes.is_quorum(nb_non_divergent, Some(CellNodeRole::Chain)) {
                if self.leader.is_none() {
                    self.find_leader_node(store)?;

                    if self.im_leader() {
                        info!("I'm the leader");
                    } else if let Some(leader_node_id) = self.leader.clone() {
                        let leader_node_disp = nodes
                            .get(&leader_node_id)
                            .map(|cn| cn.node().to_string())
                            .unwrap_or_else(|| String::from("NOT_FOUND"));
                        info!("Our leader node is {}", leader_node_disp);
                    } else {
                        warn!("Couldn't find any leader node");
                    }
                }

                self.start_leader_downloading(sync_context, store, &nodes)?;
            } else {
                return Err(ChainSyncError::Diverged(format!(
                    "Our local chain is divergent with a majority of nodes (only {} non divergents out of {})",
                    nb_non_divergent,
                    nb_total,
                ))
                    .into());
            }
        }

        // synchronize chain state with nodes
        self.synchronize_nodes_metadata(sync_context, &nodes)?;

        if status_start != self.status {
            info!(
                "Sync tick ended with new status. start_start={:?} status_end={:?}",
                status_start, self.status
            );
        } else {
            debug!(
                "Sync tick ended. start_start={:?} status_end={:?}",
                status_start, self.status
            );
        }

        Ok(())
    }

    /// Handles an incoming sync request. This request can be for metadata, or
    /// could be for blocks.
    pub fn handle_sync_request<F: FrameReader>(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        request: TypedCapnpFrame<F, chain_sync_request::Owned>,
    ) -> Result<(), EngineError> {
        let request_reader: chain_sync_request::Reader = request.get_reader()?;
        let (from_offset, to_offset) = (
            request_reader.get_from_offset(),
            request_reader.get_to_offset(),
        );
        let requested_details = request_reader.get_requested_details()?;
        debug!(
            "Got request from node {} for offset from {} to offset {} requested_details={}",
            from_node,
            from_offset,
            to_offset,
            requested_details.to_u16()
        );

        let node_info = self.get_or_create_node_info_mut(&from_node.id());
        node_info.request_tracker.set_last_responded_now();

        if requested_details == chain_sync_request::RequestedDetails::Headers {
            let from_offset_opt = if from_offset != 0 {
                Some(from_offset)
            } else {
                None
            };

            let to_offset_opt = if to_offset != 0 {
                Some(to_offset)
            } else {
                None
            };

            let blocks_metadata =
                BlockMetadata::from_store(store, from_offset_opt, to_offset_opt, &self.config)?;
            let response =
                Self::create_sync_response_for_metadata(from_offset, to_offset, blocks_metadata)?;
            sync_context.push_chain_sync_response(from_node.id().clone(), response);
        } else if requested_details == chain_sync_request::RequestedDetails::Blocks {
            let blocks_iter = store
                .blocks_iter(from_offset)?
                .filter(|b| to_offset == 0 || b.offset <= to_offset);
            let response = Self::create_sync_response_for_blocks(
                &self.config,
                from_offset,
                to_offset,
                blocks_iter,
            )?;
            sync_context.push_chain_sync_response(from_node.id().clone(), response);
        } else {
            return Err(ChainSyncError::InvalidSyncRequest(format!(
                "Unsupported requested details: {:?}",
                requested_details.to_u16()
            ))
            .into());
        }

        Ok(())
    }

    /// Handles a sync response from a node, that could contain either blocks
    /// metadata or blocks data. If it contains metadata, we gather the
    /// knowledge about the remote node's chain metadata. If it contains
    /// data, it means that it comes from the leader node and we need to
    /// append data to our local chain.
    pub fn handle_sync_response<R: FrameReader>(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        response: TypedCapnpFrame<R, chain_sync_response::Owned>,
    ) -> Result<(), EngineError> {
        let response_reader: chain_sync_response::Reader = response.get_reader()?;
        if response_reader.has_blocks() {
            debug!("Got blocks response from node {}", from_node);
            self.handle_sync_response_blocks(sync_context, from_node, store, response_reader)?;
        } else if response_reader.has_headers() {
            debug!("Got metadata response from node {}", from_node);
            self.handle_sync_response_metadata(sync_context, from_node, store, response_reader)?;
        } else {
            warn!(
                "Got a response without metadata and blocks from node {}",
                from_node
            );
        }

        // last responded is set at the end so that if we failed reading response, it's
        // considered as if we didn't receive anything (which will lead to
        // timeout & retries)
        let node_info = self.get_or_create_node_info_mut(&from_node.id());
        node_info.request_tracker.set_last_responded_now();

        Ok(())
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn im_leader(&self) -> bool {
        self.is_leader(self.cell.local_node().id())
    }

    pub fn is_leader(&self, node_id: &NodeId) -> bool {
        self.leader
            .as_ref()
            .map_or(false, |leader| leader == node_id)
    }

    /// Sends a sync request to each node that has elapsed the periodic check
    /// duration to discover its chain (last common block, last known block,
    /// etc.)
    fn synchronize_nodes_metadata(
        &mut self,
        sync_context: &mut SyncContext,
        nodes: &CellNodesOwned,
    ) -> Result<(), EngineError> {
        for cell_node in nodes
            .iter()
            .all_except_local()
            .filter(|cn| cn.has_role(CellNodeRole::Chain))
        {
            let node = cell_node.node();

            let node_info = self.get_or_create_node_info_mut(node.id());

            if node_info.request_tracker.can_send_request() {
                debug!("Sending metadata sync request to {}", node);
                let request =
                    Self::create_sync_request(node_info, RequestedDetails::Headers, None)?;
                sync_context.push_chain_sync_request(node.id().clone(), request);
                node_info.request_tracker.set_last_send_now();
            }
        }

        Ok(())
    }

    /// Starts chain downloading from current leader if needed. If leader is the
    /// local node, we don't need to download anything and mark as
    /// synchronized.
    fn start_leader_downloading(
        &mut self,
        sync_context: &mut SyncContext,
        store: &CS,
        nodes: &CellNodesOwned,
    ) -> Result<(), EngineError> {
        let node_id = self.cell.local_node().id().clone();

        // check if we're leader, and return right away if we are
        if self.im_leader() {
            info!("I'm the leader. Switching status to synchronized");
            self.status = Status::Synchronized;
            return Ok(());
        }

        let leader_node_id = if let Some(leader_node_id) = self.leader.clone() {
            leader_node_id
        } else {
            return Ok(());
        };

        // leader is another node, we check if we're already synced with it, or initiate
        // downloading with it
        let leader_node_info = self.get_or_create_node_info_mut(&leader_node_id);

        if leader_node_info.chain_fully_downloaded() {
            info!("Changing status to synchronized, as chain is fully synchronized with leader");
            self.status = Status::Synchronized;
            return Ok(());
        }

        if leader_node_info.is_divergent(store)? {
            if let Some(last_block) = store.get_last_block()? {
                error!(
                    "Leader node has no common block with us. Our last block is at offset {}",
                    last_block.offset
                );
                return Err(ChainSyncError::Diverged(format!(
                    "Diverged from leader {}",
                    leader_node_id
                ))
                .into());
            }
        }

        if leader_node_info.request_tracker.can_send_request() {
            let leader_node = nodes
                .get(&leader_node_id)
                .ok_or_else(|| {
                    ChainSyncError::Other(format!(
                        "Couldn't find leader node {} in nodes list",
                        node_id
                    ))
                })?
                .node();

            debug!(
                "Initiating chain download with leader: last_common_block={:?} last_known_block={:?}",
                leader_node_info.last_common_block, leader_node_info.last_known_block
            );
            let to_offset = leader_node_info
                .last_known_block
                .as_ref()
                .map_or(0, |block| block.offset);
            let request = Self::create_sync_request(
                leader_node_info,
                RequestedDetails::Blocks,
                Some(to_offset),
            )?;
            leader_node_info.request_tracker.set_last_send_now();
            sync_context.push_chain_sync_request(leader_node.id().clone(), request);
            self.status = Status::Downloading;
        }

        Ok(())
    }

    /// Creates a new sync request to be sent to a node, asking for blocks
    /// metadata or blocks. Blocks metadata are used remote node's chain
    /// metadata, while blocks are requested if we determined that a node is
    /// our leader.
    fn create_sync_request(
        node_info: &NodeSyncInfo,
        requested_details: RequestedDetails,
        to_offset: Option<BlockOffset>,
    ) -> Result<CapnpFrameBuilder<chain_sync_request::Owned>, EngineError> {
        let mut frame_builder = CapnpFrameBuilder::new();
        let mut request_builder: chain_sync_request::Builder = frame_builder.get_builder();

        let from_offset = node_info.last_common_block.as_ref().map_or(0, |b| {
            // if we requesting blocks, we want data from next offset to prevent getting
            // data for a block we have already have
            if requested_details == RequestedDetails::Headers {
                b.offset
            } else {
                b.next_offset()
            }
        });

        let to_offset = to_offset.unwrap_or(0);

        request_builder.set_from_offset(from_offset);
        request_builder.set_to_offset(to_offset);
        request_builder.set_requested_details(requested_details);

        debug!(
            "Sending sync_request to node={} from_offset={} to_offset={} requested_details={:?}",
            node_info.node_id,
            from_offset,
            to_offset,
            requested_details.to_u16(),
        );

        Ok(frame_builder)
    }

    /// Creates a response to a request for blocks metadata from a remote node.
    fn create_sync_response_for_metadata(
        from_offset: BlockOffset,
        to_offset: BlockOffset,
        blocks_metadata: Vec<BlockMetadata>,
    ) -> Result<CapnpFrameBuilder<chain_sync_response::Owned>, EngineError> {
        let mut frame_builder = CapnpFrameBuilder::new();
        let mut response_builder: chain_sync_response::Builder = frame_builder.get_builder();
        response_builder.set_from_offset(from_offset);
        response_builder.set_to_offset(to_offset);

        let mut headers_builder = response_builder.init_headers(blocks_metadata.len() as u32);
        for (i, header) in blocks_metadata.iter().enumerate() {
            header.copy_into_builder(&mut headers_builder.reborrow().get(i as u32));
        }

        debug!(
            "Sending {} block(s) metadata from offset {:?} to offset {:?}",
            blocks_metadata.len(),
            from_offset,
            to_offset,
        );

        Ok(frame_builder)
    }

    /// Creates a response to request for blocks data from a remote node.
    /// If we're asked for data, this means we're the lead.
    fn create_sync_response_for_blocks<'s, I: Iterator<Item = BlockRef<'s>>>(
        config: &ChainSyncConfig,
        from_offset: BlockOffset,
        to_offset: BlockOffset,
        blocks_iter: I,
    ) -> Result<CapnpFrameBuilder<chain_sync_response::Owned>, EngineError> {
        let mut frame_builder = CapnpFrameBuilder::new();
        let mut response_builder: chain_sync_response::Builder = frame_builder.get_builder();
        response_builder.set_from_offset(from_offset);
        response_builder.set_to_offset(to_offset);

        // accumulate blocks' data until we reach max packet size
        let mut data_size = 0;
        let blocks = blocks_iter
            .take_while(|block| {
                // check if we reached max at first so that we send at least 1 block even if it
                // max out
                let is_full = data_size < config.blocks_max_send_size;
                data_size += block.total_size();
                is_full
            })
            .collect::<Vec<_>>();
        let blocks_len = blocks.len() as u32;

        if blocks_len > 0 {
            let mut blocks_builder = response_builder.init_blocks(blocks_len);
            for i in 0..blocks_len {
                let block_and_signatures = blocks[i as usize].as_data_vec();
                blocks_builder.reborrow().set(i, &block_and_signatures);
            }

            debug!(
                "Sending {} block(s) data with total size {} bytes from offset {:?} to offset {:?}",
                blocks.len(),
                data_size,
                blocks.first().map(|b| b.offset),
                blocks.last().map(|b| b.offset)
            );
        }

        Ok(frame_builder)
    }

    /// Manages blocks metadata response by comparing to local blocks and
    /// finding the common ancestor (if any) and the last block of the node
    /// against which we're syncing.
    ///
    /// If we didn't find the latest common ancestor, we reply with another
    /// request from the earliest common ancestor we could find so far.
    fn handle_sync_response_metadata(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        response_reader: chain_sync_response::Reader,
    ) -> Result<(), EngineError> {
        let from_node_info = self.get_or_create_node_info_mut(&from_node.id());

        let metadata_reader = response_reader.get_headers()?;
        let mut has_new_common_block = false;
        let mut first_non_common_block: Option<BlockOffset> = None;
        let mut last_block_height = None;
        let mut all_contiguous = true;

        for metadata in metadata_reader.iter() {
            let metadata_reader: block_partial_header::Reader = metadata;
            let offset = metadata_reader.get_offset();
            let height = metadata_reader.get_height();

            // check if metedata are contiguous blocks, which would mean we can take for
            // granted that no block are missing between the first and last
            // given metadata
            if let Some(last_block_height) = last_block_height {
                if height != last_block_height + 1 {
                    all_contiguous = false;
                }
            }
            last_block_height = Some(height);

            // if we haven't encountered a block we didn't have in common, we keep checking
            // if we have the block locally, and update the last_common_block
            if first_non_common_block.is_none() {
                if let Ok(local_block) = store.get_block(offset) {
                    let local_block_signature =
                        local_block.header.inner().inner().multihash_bytes();
                    if metadata_reader.get_block_hash()? == local_block_signature {
                        let is_latest_common_offset = from_node_info
                            .last_common_block
                            .as_ref()
                            .map_or(true, |b| b.offset < offset);
                        if is_latest_common_offset {
                            from_node_info.last_common_block = Some(
                                BlockMetadata::from_block_partial_metadata_reader(metadata_reader)?,
                            );
                            has_new_common_block = true;
                        }
                    } else {
                        first_non_common_block = Some(offset);
                    }
                } else {
                    first_non_common_block = Some(offset);
                }
            }

            // update last known block if it's higher than previously known one
            let is_latest_offset = from_node_info
                .last_known_block
                .as_ref()
                .map_or(true, |b| b.offset < offset);
            if is_latest_offset {
                from_node_info.last_known_block = Some(
                    BlockMetadata::from_block_partial_metadata_reader(metadata_reader)?,
                );
            }
        }

        // if we have new common block, and blocks weren't contiguous, it means we need
        // to ask for blocks metadata from our common ancestors again, since we may have
        // a higher common block that wasn't in metadata
        if has_new_common_block && !all_contiguous {
            let to_offset = first_non_common_block;
            debug!(
                "New common ancestor block: {:?} to {:?}. Asking for more metadata.",
                from_node_info.last_common_block, first_non_common_block
            );

            let request =
                Self::create_sync_request(from_node_info, RequestedDetails::Headers, to_offset)?;
            sync_context.push_chain_sync_request(from_node_info.node_id.clone(), request);
        } else if !from_node_info.last_common_is_known {
            debug!(
                "Finished fetching metadata of node {}. last_known_block={:?}, last_common_ancestor={:?}",
                from_node_info.node_id, from_node_info.last_known_block, from_node_info.last_common_block
            );
            from_node_info.last_common_is_known = true;
            from_node_info.request_tracker.force_next_request();
        }

        Ok(())
    }

    /// Manages blocks (full data) response coming from the lead node, and
    /// appends them to our local chain. If there are still blocks after, we
    /// respond with a further request
    fn handle_sync_response_blocks(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        response_reader: chain_sync_response::Reader,
    ) -> Result<(), EngineError> {
        if !self.is_leader(from_node.id()) {
            warn!("Got data from a non-lead node {}", from_node.id());
            return Err(EngineError::Other(format!(
                "Got data from a non-lead node {}",
                from_node.id()
            )));
        }

        let from_node_info = self.get_or_create_node_info_mut(&from_node.id());

        // write incoming blocks
        let mut last_local_block: Option<BlockMetadata> = store
            .get_last_block()?
            .map(BlockMetadata::from_stored_block)
            .transpose()?;
        let blocks_reader = response_reader.get_blocks()?;
        for data_res in blocks_reader.iter() {
            // data contains both block + block_signatures
            let data = data_res?;

            // read block from data
            let block = BlockRef::new(data)?;

            // make sure the block was expected in our chain, then add it
            let next_local_offset = last_local_block
                .as_ref()
                .map_or(0, BlockMetadata::next_offset);
            if block.offset() == next_local_offset {
                sync_context.push_event(Event::NewChainBlock(block.offset()));
                store.write_block(&block)?;
                let new_block_partial_metadata = BlockMetadata::from_stored_block(block)?;
                last_local_block = Some(new_block_partial_metadata);
            } else {
                return Err(ChainSyncError::InvalidSyncResponse(format!(
                    "Got a block with data at an invalid offset. \
                     expected_offset={} block_offset={}",
                    next_local_offset,
                    block.offset()
                ))
                .into());
            }
        }
        from_node_info.last_common_block = last_local_block;

        // check if we're done
        if from_node_info.chain_fully_downloaded() {
            info!("Finished downloading chain from leader node !");
            self.status = Status::Synchronized;
        } else {
            let to_offset = from_node_info
                .last_known_block
                .as_ref()
                .map_or(0, |block| block.offset);
            let request = Self::create_sync_request(
                from_node_info,
                RequestedDetails::Blocks,
                Some(to_offset),
            )?;
            sync_context.push_chain_sync_request(from_node_info.node_id.clone(), request);
            from_node_info.request_tracker.set_last_send_now();
        }

        Ok(())
    }

    fn get_or_create_node_info_mut(&mut self, node_id: &NodeId) -> &mut NodeSyncInfo {
        if self.nodes_info.contains_key(node_id) {
            return self.nodes_info.get_mut(node_id).unwrap();
        }

        let config = self.config.clone();
        let clock = self.clock.clone();
        self.nodes_info
            .entry(node_id.clone())
            .or_insert_with(move || NodeSyncInfo::new(node_id.clone(), config, clock))
    }

    /// Iterates through all nodes we sync against and check if their status has
    /// changed
    fn check_nodes_status(&mut self, nodes: &CellNodesOwned) -> (u16, u16) {
        let mut nodes_total = 0;
        let mut nodes_metadata_sync = 0;
        for cell_node in nodes.iter().with_role(CellNodeRole::Chain) {
            let node = cell_node.node();

            nodes_total += 1;

            if node.id() == self.cell.local_node().id() {
                nodes_metadata_sync += 1;
                continue;
            }

            let node_info = self.get_or_create_node_info_mut(node.id());
            if node_info.check_status() == NodeStatus::Synchronized {
                nodes_metadata_sync += 1;
            }
        }

        (nodes_metadata_sync, nodes_total)
    }

    fn find_leader_node(&mut self, store: &CS) -> Result<(), EngineError> {
        let local_node_id = self.cell.local_node().id().clone();
        let maybe_leader = self
            .nodes_info
            .values()
            .filter_map(|info| {
                if let Some(last_known_block) = &info.last_known_block {
                    let sync_status = info.status();
                    if sync_status == NodeStatus::Synchronized {
                        Some((info, last_known_block.height))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .max_by(|(_node_a, height_a), (_node_b, height_b)| height_a.cmp(height_b));

        let last_local_block = store.get_last_block()?;
        self.leader = match (maybe_leader, &last_local_block) {
            (Some((_node_info, node_height)), Some(last_local_block))
                if last_local_block.get_height()? > node_height =>
            {
                // there are other nodes, but i have the longest chain
                Some(local_node_id)
            }
            (None, Some(_last_local_block)) => {
                // i have at least the genesis block, i'm alone, so i'm the leader
                Some(local_node_id)
            }
            (Some((node_info, _)), _) => Some(node_info.node_id.clone()),
            _ => None,
        };

        Ok(())
    }
}
