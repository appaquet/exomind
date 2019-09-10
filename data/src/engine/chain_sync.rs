use std::collections::HashMap;

use capnp::traits::ToU16;
use exocore_common::capnp;
use exocore_common::node::{Node, NodeId};
use exocore_common::protos::data_chain_capnp::{block_header, block_partial_header};
use exocore_common::protos::data_transport_capnp::{
    chain_sync_request, chain_sync_request::RequestedDetails, chain_sync_response,
};
use exocore_common::time::Clock;

use crate::block::{Block, BlockHeight, BlockOffset, BlockRef, BlockSignaturesSize};
use crate::chain;
use crate::chain::ChainStore;
use crate::engine::request_tracker::RequestTracker;
use crate::engine::{request_tracker, Event};
use crate::engine::{Error, SyncContext};
use exocore_common::cell::{Cell, CellNodes, CellNodesOwned};
use exocore_common::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};

///
/// Synchronizes the local chain against remote nodes' chain.
///
/// It achieves synchronization in 3 stages:
///
/// 1) Gather knowledge about remote nodes' chain metadata (last block, last common block).
///    During this stage, the status is 'Unknown'
///
/// 2) Once we have knowledge of the majority of node, we find the leader node with the longest chain.
///    Taking the longest chain is valid, since this node could not have made progress without a majority
///    of nodes signing the latest blocks.
///
/// 3) Download the missing blocks from the leader, starting from the latest common block, which
///    is our common ancestor.
///    During this stage, the status is 'Downloading'
///
/// 4) Once fully downloaded, we keep asking for other nodes' metadata to make sure we progress and
///    leadership hasn't changed.
///
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

    ///
    /// Called at interval to make progress on the synchronization. Depending on the current synchronization
    /// status, we could be asking for more details about remote nodes' chain, or could be asking for blocks
    /// data from the lead node.
    ///
    pub fn tick(&mut self, sync_context: &mut SyncContext, store: &CS) -> Result<(), Error> {
        let status_start = self.status;
        let nodes = self.cell.nodes().to_owned();

        let (nb_nodes_metadata_sync, nb_nodes) = self.check_nodes_status(&nodes);
        let majority_nodes_metadata_sync = nodes.is_quorum(usize::from(nb_nodes_metadata_sync));

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
                let leader_node_info = self.get_or_create_node_info_mut(&leader_node_id);
                let common_block_delta = leader_node_info.common_blocks_height_delta().unwrap_or(0);
                let sync_status = leader_node_info.status();

                let lost_leadership = if sync_status != NodeStatus::Synchronized {
                    info!(
                        "Node {} lost leadership status because it isn't sync anymore",
                        leader_node_id
                    );
                    true
                } else if common_block_delta > self.config.max_leader_common_block_height_delta {
                    info!("Node {} lost leadership status because of common block height delta is too high (height {} > height {})", leader_node_id, common_block_delta, self.config.max_leader_common_block_height_delta);
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

        // if we lost synchronization, we reset request trackers to allow quicker resynchronization by
        // not waiting for request interval
        if status_start == Status::Synchronized && self.status != Status::Synchronized {
            for node in self.nodes_info.values_mut() {
                node.request_tracker.reset();
            }
        }

        // check if we can elect a leader and download the chain from it
        if self.status != Status::Synchronized && majority_nodes_metadata_sync {
            let mut nb_non_divergent = 1;

            for node in nodes.iter().all_except_local() {
                let node_info = self.get_or_create_node_info_mut(node.id());
                if !node_info.is_divergent(store)? {
                    nb_non_divergent += 1;
                }
            }

            if nodes.is_quorum(nb_non_divergent) {
                if self.leader.is_none() {
                    self.find_leader_node(store)?;

                    if self.im_leader() {
                        info!("I'm the leader");
                    } else if let Some(leader_node_id) = self.leader.clone() {
                        info!("Our leader node is {}", leader_node_id);
                    } else {
                        warn!("Couldn't find any leader node");
                    }
                }

                self.start_leader_downloading(sync_context, store, &nodes)?;
            } else {
                return Err(ChainSyncError::Diverged(
                    format!(
                        "Our local chain is divergent with a majority of nodes (only {} non divergents out of {})",
                        nb_non_divergent,
                        nodes.len()
                    )
                ).into());
            }
        }

        // synchronize chain state with nodes
        self.synchronize_nodes_metadata(sync_context, &nodes)?;

        debug!(
            "Sync tick ended. start_start={:?} status_end={:?}",
            status_start, self.status
        );
        Ok(())
    }

    ///
    /// Handles an incoming sync request. This request can be for headers, or could be for blocks.
    ///
    pub fn handle_sync_request<F: FrameReader>(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        request: TypedCapnpFrame<F, chain_sync_request::Owned>,
    ) -> Result<(), Error> {
        let request_reader: chain_sync_request::Reader = request.get_reader()?;
        let (from_offset, to_offset) = (
            request_reader.get_from_offset(),
            request_reader.get_to_offset(),
        );
        let requested_details = request_reader.get_requested_details()?;
        debug!(
            "Got request from node {} for offset from {} to offset {} requested_details={}",
            from_node.id(),
            from_offset,
            to_offset,
            requested_details.to_u16()
        );

        let node_info = self.get_or_create_node_info_mut(&from_node.id());
        node_info.request_tracker.set_last_responded_now();

        if requested_details == chain_sync_request::RequestedDetails::Headers {
            let to_offset_opt = if to_offset != 0 {
                Some(to_offset)
            } else {
                None
            };

            let headers = chain_sample_block_partial_headers(
                store,
                from_offset,
                to_offset_opt,
                self.config.headers_sync_begin_count,
                self.config.headers_sync_end_count,
                self.config.headers_sync_sampled_count,
            )?;

            let response = Self::create_sync_response_for_headers(from_offset, to_offset, headers)?;
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

    ///
    /// Handles a sync response from a node, that could contain either headers or blocks data.
    /// If it contains headers, we gather the knowledge about the remote node's chain metadata.
    /// If it contains data, it means that it comes from the leader node and we need to append data
    /// to our local chain.
    ///
    pub fn handle_sync_response<R: FrameReader>(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        response: TypedCapnpFrame<R, chain_sync_response::Owned>,
    ) -> Result<(), Error> {
        let response_reader: chain_sync_response::Reader = response.get_reader()?;
        if response_reader.has_blocks() {
            debug!("Got blocks response from node {}", from_node.id());
            self.handle_sync_response_blocks(sync_context, from_node, store, response_reader)?;
        } else if response_reader.has_headers() {
            debug!("Got headers response from node {}", from_node.id());
            self.handle_sync_response_headers(sync_context, from_node, store, response_reader)?;
        } else {
            warn!(
                "Got a response without headers and blocks from node {}",
                from_node.id()
            );
        }

        // last responded is set at the end so that if we failed reading response, it's considered
        // as if we didn't receive anything (which will lead to timeout & retries)
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

    ///
    /// Sends a sync request to each node that has elapsed the periodic check duration to discover
    /// its chain (last common block, last known block, etc.)
    ///
    fn synchronize_nodes_metadata(
        &mut self,
        sync_context: &mut SyncContext,
        nodes: &CellNodesOwned,
    ) -> Result<(), Error> {
        for node in nodes.iter().all_except_local() {
            let node_info = self.get_or_create_node_info_mut(node.id());

            if node_info.request_tracker.can_send_request() {
                debug!("Sending metadata sync request to {}", node.id());
                let request =
                    Self::create_sync_request(node_info, RequestedDetails::Headers, None)?;
                sync_context.push_chain_sync_request(node.id().clone(), request);
                node_info.request_tracker.set_last_send_now();
            }
        }

        Ok(())
    }

    ///
    /// Starts chain downloading from current leader if needed. If leader is the local node, we don't
    /// need to download anything and mark as synchronized.
    ///
    fn start_leader_downloading(
        &mut self,
        sync_context: &mut SyncContext,
        store: &CS,
        nodes: &CellNodesOwned,
    ) -> Result<(), Error> {
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

        // leader is another node, we check if we're already synced with it, or initiate downloading with it
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
            let leader_node = nodes.get(&leader_node_id).ok_or_else(|| {
                ChainSyncError::Other(format!(
                    "Couldn't find leader node {} in nodes list",
                    node_id
                ))
            })?;

            debug!("Initiating chain download with leader: last_common_block={:?} last_known_block={:?}", leader_node_info.last_common_block, leader_node_info.last_known_block);
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

    ///
    /// Creates a new sync request to be sent to a node, asking for headers or blocks. Headers are
    /// used remote node's chain metadata, while blocks are requested if we determined that a node
    /// is our leader.
    ///
    fn create_sync_request(
        node_info: &NodeSyncInfo,
        requested_details: RequestedDetails,
        to_offset: Option<BlockOffset>,
    ) -> Result<CapnpFrameBuilder<chain_sync_request::Owned>, Error> {
        let mut frame_builder = CapnpFrameBuilder::new();
        let mut request_builder: chain_sync_request::Builder = frame_builder.get_builder();

        let from_offset = node_info.last_common_block.as_ref().map_or(0, |b| {
            // if we requesting blocks, we want data from next offset to prevent getting data
            // for a block we have already have
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

    ///
    /// Creates a response to a request for headers from a remote node.
    ///
    fn create_sync_response_for_headers(
        from_offset: BlockOffset,
        to_offset: BlockOffset,
        headers: Vec<BlockPartialHeader>,
    ) -> Result<CapnpFrameBuilder<chain_sync_response::Owned>, Error> {
        let mut frame_builder = CapnpFrameBuilder::new();
        let mut response_builder: chain_sync_response::Builder = frame_builder.get_builder();
        response_builder.set_from_offset(from_offset);
        response_builder.set_to_offset(to_offset);

        let mut headers_builder = response_builder.init_headers(headers.len() as u32);
        for (i, header) in headers.iter().enumerate() {
            header.copy_into_builder(&mut headers_builder.reborrow().get(i as u32));
        }

        debug!(
            "Sending {} header(s) from offset {:?} to offset {:?}",
            headers.len(),
            from_offset,
            to_offset,
        );

        Ok(frame_builder)
    }

    ///
    /// Creates a response to request for blocks data from a remote node.
    /// If we're asked for data, this means we're the lead.
    ///
    fn create_sync_response_for_blocks<'s, I: Iterator<Item = BlockRef<'s>>>(
        config: &ChainSyncConfig,
        from_offset: BlockOffset,
        to_offset: BlockOffset,
        blocks_iter: I,
    ) -> Result<CapnpFrameBuilder<chain_sync_response::Owned>, Error> {
        let mut frame_builder = CapnpFrameBuilder::new();
        let mut response_builder: chain_sync_response::Builder = frame_builder.get_builder();
        response_builder.set_from_offset(from_offset);
        response_builder.set_to_offset(to_offset);

        // accumulate blocks' data until we reach max packet size
        let mut data_size = 0;
        let blocks = blocks_iter
            .take_while(|block| {
                // check if we reached max at first so that we send at least 1 block even if it max out
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

    ///
    /// Manages headers response by comparing to local blocks and finding the common ancestor (if any)
    /// and the last block of the node against which we're syncing.
    ///
    /// If we didn't find the latest common ancestor, we reply with another request from the earliest
    /// common ancestor we could find so far.
    ///
    fn handle_sync_response_headers(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        response_reader: chain_sync_response::Reader,
    ) -> Result<(), Error> {
        let from_node_info = self.get_or_create_node_info_mut(&from_node.id());

        let headers_reader = response_reader.get_headers()?;
        let mut has_new_common_block = false;
        let mut first_non_common_block: Option<BlockOffset> = None;
        let mut last_header_height = None;
        let mut all_contiguous = true;

        for header in headers_reader.iter() {
            let header_reader: block_partial_header::Reader = header;
            let offset = header_reader.get_offset();
            let height = header_reader.get_height();

            // check if headers are contiguous blocks, which would mean we can take for granted that no block
            // are missing between the first and last given header
            if let Some(last_header_height) = last_header_height {
                if height != last_header_height + 1 {
                    all_contiguous = false;
                }
            }
            last_header_height = Some(height);

            // if we haven't encountered a block we didn't have in common, we keep checking if we have
            // the block locally, and update the last_common_block
            if first_non_common_block.is_none() {
                if let Ok(local_block) = store.get_block(offset) {
                    let local_block_signature =
                        local_block.header.inner().inner().multihash_bytes();
                    if header_reader.get_block_hash()? == local_block_signature {
                        let is_latest_common_offset = from_node_info
                            .last_common_block
                            .as_ref()
                            .map_or(true, |b| b.offset < offset);
                        if is_latest_common_offset {
                            from_node_info.last_common_block =
                                Some(BlockPartialHeader::from_block_partial_header_reader(
                                    header_reader,
                                )?);
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
                    BlockPartialHeader::from_block_partial_header_reader(header_reader)?,
                );
            }
        }

        // if we have new common block, and blocks weren't contiguous, it means we need to ask for headers
        // from our common ancestors again, since we may have a higher common block that wasn't in headers
        if has_new_common_block && !all_contiguous {
            let to_offset = first_non_common_block;
            debug!(
                "New common ancestor block: {:?} to {:?}. Asking for more headers.",
                from_node_info.last_common_block, first_non_common_block
            );

            let request =
                Self::create_sync_request(from_node_info, RequestedDetails::Headers, to_offset)?;
            sync_context.push_chain_sync_request(from_node_info.node_id.clone(), request);
        } else if !from_node_info.last_common_is_known {
            debug!(
                "Finished fetching metadata of node {}. last_known_block={:?}, last_common_ancestor={:?}",
                from_node_info.node_id,
                from_node_info.last_known_block,
                from_node_info.last_common_block
            );
            from_node_info.last_common_is_known = true;
            from_node_info.request_tracker.force_next_request();
        }

        Ok(())
    }

    ///
    /// Manages blocks (full data) response coming from the lead node, and appends them to our local chain.
    /// If there are still blocks after, we respond with a further request
    ///
    fn handle_sync_response_blocks(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        response_reader: chain_sync_response::Reader,
    ) -> Result<(), Error> {
        if !self.is_leader(from_node.id()) {
            warn!("Got data from a non-lead node {}", from_node.id());
            return Err(Error::Other(format!(
                "Got data from a non-lead node {}",
                from_node.id()
            )));
        }

        let from_node_info = self.get_or_create_node_info_mut(&from_node.id());

        // write incoming blocks
        let mut last_local_block: Option<BlockPartialHeader> = store
            .get_last_block()?
            .map(BlockPartialHeader::from_stored_block)
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
                .map_or(0, BlockPartialHeader::next_offset);
            if block.offset() == next_local_offset {
                sync_context.push_event(Event::NewChainBlock(block.offset()));
                store.write_block(&block)?;
                let new_block_partial_header = BlockPartialHeader::from_stored_block(block)?;
                last_local_block = Some(new_block_partial_header);
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

        let config = self.config;
        let clock = self.clock.clone();
        self.nodes_info
            .entry(node_id.clone())
            .or_insert_with(move || NodeSyncInfo::new(node_id.clone(), config, clock))
    }

    ///
    /// Iterates through all nodes we sync against and check if their status has changed
    ///
    fn check_nodes_status(&mut self, nodes: &CellNodesOwned) -> (u16, u16) {
        let mut nodes_total = 0;
        let mut nodes_metadata_sync = 0;
        for node in nodes.iter().all() {
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

    fn find_leader_node(&mut self, store: &CS) -> Result<(), Error> {
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
                Some(local_node_id.clone())
            }
            (None, Some(_last_local_block)) => {
                // i have at least the genesis block, i'm alone, so i'm the leader
                Some(local_node_id.clone())
            }
            (Some((node_info, _)), _) => Some(node_info.node_id.clone()),
            _ => None,
        };

        Ok(())
    }
}

///
/// Chain synchronizer's configuration
///
#[derive(Copy, Clone, Debug)]
pub struct ChainSyncConfig {
    /// Config for requests timing tracker
    pub request_tracker: request_tracker::RequestTrackerConfig,

    /// Maximum number of synchronization failures before considering a node offsync
    pub meta_sync_max_failures: usize,

    /// Number of headers to always include at beginning of a headers sync request
    pub headers_sync_begin_count: BlockOffset,

    /// Number of headers to always include at end of a headers sync request
    pub headers_sync_end_count: BlockOffset,

    /// Number of sampled headers to include between begin and end headers of a headers sync request
    pub headers_sync_sampled_count: BlockOffset,

    /// Maximum number of bytes worth of blocks to send in a response
    /// This should be lower than transport maximum packet size
    pub blocks_max_send_size: usize,

    /// Maximum height in blocks that we can tolerate between our common ancestor block
    /// and its latest block. If it gets higher than this value, this means that we may
    /// have diverged and we need to re-synchronize.
    pub max_leader_common_block_height_delta: BlockHeight,
}

impl Default for ChainSyncConfig {
    fn default() -> Self {
        ChainSyncConfig {
            request_tracker: request_tracker::RequestTrackerConfig::default(),
            meta_sync_max_failures: 2,
            headers_sync_begin_count: 5,
            headers_sync_end_count: 5,
            headers_sync_sampled_count: 10,
            blocks_max_send_size: 50 * 1024,
            max_leader_common_block_height_delta: 5,
        }
    }
}

///
/// Synchronization information about a remote node
///
struct NodeSyncInfo {
    config: ChainSyncConfig,
    node_id: NodeId,

    last_common_block: Option<BlockPartialHeader>,
    last_common_is_known: bool,
    last_known_block: Option<BlockPartialHeader>,

    request_tracker: RequestTracker,
}

impl NodeSyncInfo {
    fn new(node_id: NodeId, config: ChainSyncConfig, clock: Clock) -> NodeSyncInfo {
        NodeSyncInfo {
            config,
            node_id,

            last_common_block: None,
            last_common_is_known: false,
            last_known_block: None,

            request_tracker: RequestTracker::new_with_clock(clock, config.request_tracker),
        }
    }

    fn check_status(&mut self) -> NodeStatus {
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

    fn status(&self) -> NodeStatus {
        if self.last_common_is_known {
            NodeStatus::Synchronized
        } else {
            NodeStatus::Unknown
        }
    }

    fn chain_fully_downloaded(&self) -> bool {
        let last_known_offset = self.last_known_block.as_ref().map(|b| b.offset);
        let last_common_offset = self.last_common_block.as_ref().map(|b| b.offset);
        self.last_known_block.is_some() && last_known_offset == last_common_offset
    }

    ///
    /// Returns delta in block height between the last known block of the node and the last common block
    /// that we have.
    ///
    fn common_blocks_height_delta(&self) -> Option<BlockHeight> {
        match (&self.last_common_block, &self.last_known_block) {
            (Some(common), Some(known)) => Some(known.height - common.height),
            _ => None,
        }
    }

    ///
    /// Check if what we know of the remote node's chain is considered divergent. A divergent chain
    /// is a forked chain, in which we have a common ancestor, but different subsequent blocks.
    ///
    fn is_divergent<CS: ChainStore>(&self, local_store: &CS) -> Result<bool, Error> {
        if let Some(last_common_block) = &self.last_common_block {
            let last_known_block = if let Some(last_known_block) = self.last_known_block.as_ref() {
                last_known_block
            } else {
                return Ok(false);
            };

            let last_local_block = local_store.get_last_block()?.ok_or_else(|| {
                Error::Other(String::from(
                    "Expected a common block to be in stored since it had previously been",
                ))
            })?;
            let last_local_height = last_local_block.get_height()?;

            // if we have a block after common, and that the remote has one too, we are divergent
            Ok(last_local_height > last_common_block.height
                && last_known_block.height > last_common_block.height)
        } else {
            // if we don't have any common block and we have at least one block in local chain,
            // and that remote node is not empty, we have diverged from it
            let last_local_block = local_store.get_last_block()?;
            Ok(last_local_block.is_some() && self.last_known_block.is_some())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum NodeStatus {
    Unknown,
    Synchronized,
}

///
/// Partial header of a block coming from local store or remote node, used for comparison
/// between local and remote stores
///
#[derive(Debug)]
struct BlockPartialHeader {
    offset: BlockOffset,
    height: BlockHeight,
    hash: Vec<u8>,
    previous_offset: BlockOffset,
    previous_hash: Vec<u8>,

    block_size: u32,
    operations_size: u32,
    signatures_size: BlockSignaturesSize,
}

impl BlockPartialHeader {
    fn from_stored_block<B: Block>(stored_block: B) -> Result<BlockPartialHeader, Error> {
        let block_header_reader: block_header::Reader = stored_block.header().get_reader()?;
        let block_signature = stored_block.header().inner().inner().multihash_bytes();

        Ok(BlockPartialHeader {
            offset: stored_block.offset(),
            height: block_header_reader.get_height(),
            hash: block_signature.to_vec(),
            previous_offset: block_header_reader.get_previous_offset(),
            previous_hash: block_header_reader.get_previous_hash()?.to_vec(),

            block_size: stored_block.header().whole_data_size() as u32,
            operations_size: block_header_reader.get_operations_size(),
            signatures_size: block_header_reader.get_signatures_size(),
        })
    }

    fn from_block_partial_header_reader(
        block_partial_header_reader: block_partial_header::Reader,
    ) -> Result<BlockPartialHeader, Error> {
        Ok(BlockPartialHeader {
            offset: block_partial_header_reader.get_offset(),
            height: block_partial_header_reader.get_height(),
            hash: block_partial_header_reader.get_block_hash()?.to_vec(),
            previous_offset: block_partial_header_reader.get_previous_offset(),
            previous_hash: block_partial_header_reader.get_previous_hash()?.to_vec(),
            block_size: block_partial_header_reader.get_block_size(),
            operations_size: block_partial_header_reader.get_operations_size(),
            signatures_size: block_partial_header_reader.get_signatures_size(),
        })
    }

    #[inline]
    fn next_offset(&self) -> BlockOffset {
        self.offset
            + BlockOffset::from(self.block_size)
            + BlockOffset::from(self.operations_size)
            + BlockOffset::from(self.signatures_size)
    }

    fn copy_into_builder(&self, builder: &mut block_partial_header::Builder) {
        builder.set_offset(self.offset);
        builder.set_height(self.height);
        builder.set_block_hash(&self.hash);
        builder.set_previous_offset(self.previous_offset);
        builder.set_previous_hash(&self.previous_hash);
        builder.set_block_size(self.block_size);
        builder.set_operations_size(self.operations_size);
        builder.set_signatures_size(self.signatures_size);
    }
}

///
/// Chain synchronizer specific error
///
#[derive(Clone, Debug, Fail)]
pub enum ChainSyncError {
    #[fail(display = "Got an invalid sync request: {}", _0)]
    InvalidSyncRequest(String),
    #[fail(display = "Got an invalid sync response: {}", _0)]
    InvalidSyncResponse(String),
    #[fail(display = "Our local chain has diverged from leader node: {}", _0)]
    Diverged(String),
    #[fail(display = "Got an error: {}", _0)]
    Other(String),
}

impl ChainSyncError {
    pub fn is_fatal(&self) -> bool {
        match *self {
            ChainSyncError::Diverged(_) => true,
            _ => false,
        }
    }
}

///
/// Samples the local chain and returns a collection of `BlockPartialHeader` at different position in the asked range.
///
/// `from_offset` and `to_offset` are best efforts and fallback to begin/end of chain if they don't exist.
/// `begin_count` and `end_count` are number of headers to include without sampling from beginning and end of range.
/// `sampled_count` is the approximate number of headers to return, excluding the `begin_count` and `end_count`
///
fn chain_sample_block_partial_headers<CS: chain::ChainStore>(
    store: &CS,
    from_offset: BlockOffset,
    to_offset: Option<BlockOffset>,
    begin_count: BlockOffset,
    end_count: BlockOffset,
    sampled_count: BlockOffset,
) -> Result<Vec<BlockPartialHeader>, Error> {
    let mut headers = Vec::new();

    let segments_range = store.segments();
    if segments_range.is_empty() {
        return Ok(headers);
    }

    let last_block = match to_offset {
        Some(offset) => store.get_block(offset).map(Some).or_else(|_| {
            warn!(
                "Given to offset {} didn't exist. Falling back to last block of chain",
                offset
            );
            store.get_last_block()
        }),
        None => store.get_last_block(),
    }?
    .ok_or_else(|| {
        ChainSyncError::Other("Expected a last block since ranges were not empty".to_string())
    })?;

    let last_block_header_reader: block_header::Reader = last_block.header.get_reader()?;
    let last_block_height = last_block_header_reader.get_height();

    let mut blocks_iter = store
        .blocks_iter(from_offset)
        .or_else(|_| store.blocks_iter(0))?
        .peekable();

    let first_block = blocks_iter.peek().ok_or_else(|| {
        ChainSyncError::Other("Expected a first block since ranges were not empty".to_string())
    })?;
    let first_block_header_reader: block_header::Reader = first_block.header.get_reader()?;
    let first_block_height = first_block_header_reader.get_height();

    let range_blocks_count = last_block_height - first_block_height;
    let range_blocks_skip = (range_blocks_count / sampled_count).max(1);

    // from which block do we include all headers so that we always include last `end_count` blocks
    let range_blocks_lasts = range_blocks_count
        .checked_sub(end_count)
        .unwrap_or(range_blocks_count);

    for (blocks_count, current_block) in blocks_iter
        .enumerate()
        .take(range_blocks_count as usize + 1)
    {
        // we always include headers if the block is within the first `begin_count` or in the last `end_count`
        // otherwise, we include if it falls within sampling condition
        let blocks_count = blocks_count as BlockOffset;
        if blocks_count < begin_count
            || blocks_count > range_blocks_lasts
            || blocks_count % range_blocks_skip == 0
        {
            let block_partial_header = BlockPartialHeader::from_stored_block(current_block)?;
            headers.push(block_partial_header);
        }
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use chain::directory::DirectoryChainStore;

    use crate::engine::testing::*;
    use crate::engine::{SyncContextMessage, SyncState};

    use super::*;
    use crate::operation::OperationBuilder;
    use exocore_common::framing::FrameBuilder;
    use itertools::Itertools;

    #[test]
    fn handle_sync_response_blocks() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(0, 10, 1234);
        cluster.chain_generate_dummy(1, 100, 1234);

        let node0 = cluster.get_node(0);
        let node1 = cluster.get_node(1);

        run_sync_1_to_1(&mut cluster, 0, 1)?;
        cluster.tick_chain_synchronizer(0)?;
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Downloading);
        assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

        // response from non-leader should result in an error
        let blocks_iter = cluster.chains[1].blocks_iter(0)?;
        let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
            &cluster.chains_synchronizer[1].config,
            10,
            0,
            blocks_iter,
        )?;
        let response_frame = response.as_owned_frame();
        let mut sync_context = SyncContext::new(SyncState::default());
        let result = cluster.chains_synchronizer[0].handle_sync_response(
            &mut sync_context,
            &node0,
            &mut cluster.chains[0],
            response_frame,
        );
        assert!(result.is_err());
        assert!(sync_context.messages.is_empty());

        // response from leader with blocks that aren't next should fail
        let blocks_iter = cluster.chains[1].blocks_iter(0)?;
        let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
            &cluster.chains_synchronizer[1].config,
            10,
            0,
            blocks_iter,
        )?;
        let response_frame = response.as_owned_frame();
        let mut sync_context = SyncContext::new(SyncState::default());
        let result = cluster.chains_synchronizer[0].handle_sync_response(
            &mut sync_context,
            &node1,
            &mut cluster.chains[0],
            response_frame,
        );
        assert!(result.is_err());

        // response from leader with blocks at right position should succeed and append
        let blocks_iter = cluster.chains[1].blocks_iter(0).unwrap().skip(10); // skip 10 will go to 10th block
        let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
            &cluster.chains_synchronizer[0].config,
            10,
            0,
            blocks_iter,
        )?;
        let response_frame = response.as_owned_frame();
        let mut sync_context = SyncContext::new(SyncState::default());
        cluster.chains_synchronizer[0].handle_sync_response(
            &mut sync_context,
            &node1,
            &mut cluster.chains[0],
            response_frame,
        )?;

        Ok(())
    }

    #[test]
    fn test_chain_sample_block_partial_headers() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(1);
        cluster.chain_generate_dummy(0, 100, 3424);

        let offsets: Vec<BlockOffset> = cluster.chains[0]
            .blocks_iter(0)?
            .map(|b| b.offset)
            .collect();

        let headers = chain_sample_block_partial_headers(&cluster.chains[0], 0, None, 2, 2, 10)?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![0, 1, 9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 98, 99]
        );

        let headers = chain_sample_block_partial_headers(&cluster.chains[0], 0, None, 0, 0, 1)?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![0, 99]
        );

        let headers =
            chain_sample_block_partial_headers(&cluster.chains[0], offsets[10], None, 5, 5, 10)?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![10, 11, 12, 13, 14, 18, 26, 34, 42, 50, 58, 66, 74, 82, 90, 95, 96, 97, 98, 99]
        );

        let headers = chain_sample_block_partial_headers(
            &cluster.chains[0],
            offsets[10],
            Some(offsets[50]),
            2,
            2,
            5,
        )?;
        assert_eq!(
            headers.iter().map(|b| b.height).collect::<Vec<_>>(),
            vec![10, 11, 18, 26, 34, 42, 49, 50]
        );

        Ok(())
    }

    #[test]
    fn sync_empty_node1_to_full_node2() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(1, 100, 3434);

        let node1 = cluster.get_node(1);

        run_sync_1_to_1(&mut cluster, 0, 1)?;
        {
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
            assert_eq!(NodeStatus::Synchronized, node1_node2_info.status(),);
            assert_eq!(
                None,
                node1_node2_info
                    .last_common_block
                    .as_ref()
                    .map(|b| b.height),
            );
            assert_eq!(
                Some(99),
                node1_node2_info.last_known_block.as_ref().map(|b| b.height),
            );
        }

        // this will sync blocks & mark as synchronized
        run_sync_1_to_1(&mut cluster, 0, 1)?;
        assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);
        assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

        // force status back to downloading to check if tick will turn back to synchronized
        cluster.chains_synchronizer[0].status = Status::Downloading;
        run_sync_1_to_1(&mut cluster, 0, 1)?;
        assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);

        nodes_expect_chain_equals(&cluster.chains[0], &cluster.chains[1]);

        Ok(())
    }

    #[test]
    fn sync_full_node1_to_empty_node2() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(0, 100, 3434);

        let node1 = cluster.get_node(1);

        // running sync twice will yield to nothing as node2 is empty
        for _i in 0..2 {
            run_sync_1_to_1(&mut cluster, 0, 1)?;
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
            assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
            assert_eq!(
                node1_node2_info
                    .last_common_block
                    .as_ref()
                    .map(|b| b.height),
                None
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.height),
                None
            );
        }

        // node1 is full, it has quorum (1 out of 2 nodes >= 50%)
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        Ok(())
    }

    #[test]
    fn sync_full_node1_to_half_node2() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(0, 100, 3434);
        cluster.chain_generate_dummy(1, 50, 3434);

        let node0 = cluster.get_node(0);
        let node1 = cluster.get_node(1);

        // running sync twice will yield to nothing as node1 is leader
        for _i in 0..2 {
            run_sync_1_to_1(&mut cluster, 0, 1)?;
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
            assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
            assert_eq!(
                node1_node2_info
                    .last_common_block
                    .as_ref()
                    .map(|b| b.height),
                Some(49)
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.height),
                Some(49)
            );
        }

        // we're leader and synchronized because of it
        assert!(cluster.chains_synchronizer[0].is_leader(node0.id()));
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        Ok(())
    }

    #[test]
    fn sync_half_node1_to_full_node2() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(0, 50, 3434);
        cluster.chain_generate_dummy(1, 100, 3434);

        let node1 = cluster.get_node(1);

        run_sync_1_to_1(&mut cluster, 0, 1)?;
        {
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
            assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
            assert_eq!(
                node1_node2_info
                    .last_common_block
                    .as_ref()
                    .map(|b| b.height),
                Some(49)
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.height),
                Some(99)
            );
        }

        // this will sync blocks & mark as synchronized
        run_sync_1_to_1(&mut cluster, 0, 1)?;

        // node2 is leader
        assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        nodes_expect_chain_equals(&cluster.chains[0], &cluster.chains[1]);

        Ok(())
    }

    #[test]
    fn sync_fully_divergent_node1_to_full_node2() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(0, 100, 1234);
        cluster.chain_generate_dummy(1, 100, 9876);

        let node1 = cluster.get_node(1);

        run_sync_1_to_1(&mut cluster, 0, 1)?;
        {
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
            assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
            assert_eq!(
                node1_node2_info
                    .last_common_block
                    .as_ref()
                    .map(|b| b.height),
                None,
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.height),
                Some(99),
            );
        }

        match run_sync_1_to_1(&mut cluster, 0, 1).err() {
            Some(Error::ChainSync(ChainSyncError::Diverged(_))) => {}
            other => panic!("Expected a diverged error, got {:?}", other),
        }

        // still unknown since we don't have a clear leader, as we've diverged from it
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Unknown);

        Ok(())
    }

    #[test]
    fn sync_single_block_even_if_max_out_size() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);

        let node_0 = cluster.get_local_node(0).clone();
        cluster.chain_add_genesis_block(0);

        // generate a block that exceeds maximum send size
        let operation_size = cluster.chains_synchronizer[0].config.blocks_max_send_size / 9;
        let operations = (0..10)
            .map(|_i| {
                let op_id = cluster.consistent_timestamp(0).into();
                let data = vec![0u8; operation_size + 1];
                OperationBuilder::new_entry(op_id, node_0.id(), &data)
                    .sign_and_build(&node_0)
                    .unwrap()
                    .frame
            })
            .collect_vec();
        cluster.chain_add_block_with_operations(0, operations.into_iter())?;

        let node0_last_block = cluster.chains[0].get_last_block()?.unwrap();
        let node0_last_block_size = node0_last_block.operations_data().len();
        assert!(node0_last_block_size > cluster.chains_synchronizer[0].config.blocks_max_send_size);

        // node 1 is empty
        cluster.chain_generate_dummy(1, 0, 1234);

        // make node 1 fetch data from node 0
        run_sync_1_to_1(&mut cluster, 1, 0)?;
        run_sync_1_to_1(&mut cluster, 1, 0)?;

        // node 1 should have the block even if it was bigger than maximum size, but it should
        // have sent blocks 1 by 1 instead
        let node1_last_block = cluster.chains[1].get_last_block()?.unwrap();
        assert_eq!(
            node0_last_block_size,
            node1_last_block.operations_data().len()
        );

        Ok(())
    }

    #[test]
    fn cannot_sync_all_divergent() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(4);
        cluster.chain_generate_dummy(0, 100, 1234);
        cluster.chain_generate_dummy(1, 100, 9876);
        cluster.chain_generate_dummy(2, 100, 9876);
        cluster.chain_generate_dummy(3, 100, 9876);

        run_sync_1_to_n(&mut cluster, 0)?;
        match run_sync_1_to_n(&mut cluster, 0).err() {
            Some(Error::ChainSync(ChainSyncError::Diverged(_))) => {}
            other => panic!("Expected a diverged error, got {:?}", other),
        }

        // still unknown since we don't have a clear leader, as we've diverged from it
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Unknown);

        Ok(())
    }

    #[test]
    fn sync_half_divergent_node1_to_full_node2() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(0, 100, 1234);
        cluster.chain_generate_dummy(1, 50, 1234);
        cluster.chain_append_dummy(1, 50, 1234);

        let node1 = cluster.get_node(1);

        run_sync_1_to_1(&mut cluster, 0, 1)?;
        {
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info[node1.id()];
            assert_eq!(node1_node2_info.status(), NodeStatus::Synchronized);
            assert_eq!(
                node1_node2_info
                    .last_common_block
                    .as_ref()
                    .map(|b| b.height),
                Some(49),
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.height),
                Some(99),
            );
        }

        match run_sync_1_to_1(&mut cluster, 0, 1).err() {
            Some(Error::ChainSync(ChainSyncError::Diverged(_))) => {}
            other => panic!("Expected a diverged error, got {:?}", other),
        }

        // still unknown since we don't have a clear leader, as we've diverged from it
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Unknown);

        Ok(())
    }

    #[test]
    fn sync_empty_node1_to_big_chain_node2() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);

        // this will force multiple back and forth for data
        cluster.chains_synchronizer[0].config.blocks_max_send_size = 1024;

        cluster.chain_generate_dummy(1, 1024, 3434);

        // first sync for metadata
        run_sync_1_to_1(&mut cluster, 0, 1)?;

        // second sync for data
        run_sync_1_to_1(&mut cluster, 0, 1)?;

        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        Ok(())
    }

    #[test]
    fn leader_lost_metadata_out_of_date() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(4);
        cluster.chain_generate_dummy(0, 50, 3434);
        cluster.chain_generate_dummy(1, 100, 3434);
        cluster.chain_generate_dummy(2, 90, 3434);
        cluster.chain_generate_dummy(3, 90, 3434);

        let node1 = cluster.get_node(1);

        run_sync_1_to_n(&mut cluster, 0)?;
        run_sync_1_to_n(&mut cluster, 0)?;

        // node 1 is now leader
        assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

        {
            // we remove sync metadata from leader
            let node_info = cluster.chains_synchronizer[0].get_or_create_node_info_mut(node1.id());
            assert_eq!(node_info.status(), NodeStatus::Synchronized);
            node_info.last_common_is_known = false;
            node_info.last_known_block = None;
            assert_eq!(node_info.status(), NodeStatus::Unknown);
        }

        // node 1 is not leader anymore
        cluster.tick_chain_synchronizer(0)?;
        assert!(!cluster.chains_synchronizer[0].is_leader(node1.id()));

        Ok(())
    }

    #[test]
    fn leader_lost_chain_too_far() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(2);
        cluster.chain_generate_dummy(0, 50, 3434);
        cluster.chain_generate_dummy(1, 100, 3434);
        cluster.clocks[0].set_fixed_instant(Instant::now());

        let node1 = cluster.get_node(1);

        run_sync_1_to_1(&mut cluster, 0, 1)?;
        run_sync_1_to_1(&mut cluster, 0, 1)?;

        assert!(cluster.chains_synchronizer[0].is_leader(node1.id()));

        // make leader add 2 blocks, which shouldn't be considered as too far ahead
        cluster.chain_append_dummy(1, 2, 3434);
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(10));
        run_sync_1_to_1(&mut cluster, 0, 1)?;
        assert_eq!(
            Status::Synchronized,
            cluster.chains_synchronizer[0].status(),
        );

        // make leader add 10 blocks, which should now be considered as too far ahead
        cluster.chain_append_dummy(1, 10, 3434);
        cluster.clocks[0].add_fixed_instant_duration(Duration::from_secs(10));
        run_sync_1_to_1(&mut cluster, 0, 1)?;

        // now, a simple tick should reset status to downloading since we need to catch up with master
        cluster.tick_chain_synchronizer(0)?;
        assert_eq!(Status::Downloading, cluster.chains_synchronizer[0].status(),);

        Ok(())
    }

    #[test]
    fn quorum_lost_and_regain() -> Result<(), failure::Error> {
        let mut cluster = EngineTestCluster::new(3);
        cluster.chain_generate_dummy(0, 50, 3434);
        cluster.chain_generate_dummy(1, 100, 3434);
        cluster.chain_generate_dummy(2, 100, 3434);

        run_sync_1_to_n(&mut cluster, 0)?;
        run_sync_1_to_n(&mut cluster, 0)?;

        assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);

        // wipe metadata for node 1 and 2
        for node_idx in 1..=2 {
            let node = cluster.get_node(node_idx);
            let node_info = cluster.chains_synchronizer[0].get_or_create_node_info_mut(node.id());
            assert_eq!(NodeStatus::Synchronized, node_info.check_status());
            node_info.request_tracker.set_response_failure_count(100);
            assert_eq!(NodeStatus::Unknown, node_info.check_status());
        }

        // we lost quorum, we should now be synchronized anymore, no matter how many ticks we do
        cluster.tick_chain_synchronizer(0)?;
        cluster.tick_chain_synchronizer(0)?;
        cluster.tick_chain_synchronizer(0)?;
        assert_eq!(Status::Unknown, cluster.chains_synchronizer[0].status);

        // reset request tracker to prevent waiting for last request timeout
        for node_idx in 1..=2 {
            let node = cluster.get_node(node_idx);
            let node_info = cluster.chains_synchronizer[0].get_or_create_node_info_mut(node.id());
            node_info.request_tracker.reset();
        }

        // now we do full sync between nodes, it will put back status
        run_sync_1_to_n(&mut cluster, 0)?;
        run_sync_1_to_n(&mut cluster, 0)?;
        assert_eq!(Status::Synchronized, cluster.chains_synchronizer[0].status);

        Ok(())
    }

    fn extract_request_frame_sync_context(
        sync_context: &SyncContext,
        to_node: &NodeId,
    ) -> TypedCapnpFrame<Vec<u8>, chain_sync_request::Owned> {
        for sync_message in &sync_context.messages {
            match sync_message {
                SyncContextMessage::ChainSyncRequest(msg_to_node, req)
                    if msg_to_node == to_node =>
                {
                    return req.as_owned_frame();
                }
                _ => {}
            }
        }

        panic!("Couldn't find message for node {}", to_node);
    }

    fn extract_response_frame_sync_context(
        sync_context: &SyncContext,
    ) -> (NodeId, TypedCapnpFrame<Vec<u8>, chain_sync_response::Owned>) {
        match sync_context.messages.last().unwrap() {
            SyncContextMessage::ChainSyncResponse(to_node, req) => {
                (to_node.clone(), req.as_owned_frame())
            }
            _other => panic!("Expected a chain sync response, got another type of message"),
        }
    }

    fn run_sync_1_to_1(
        cluster: &mut EngineTestCluster,
        node_id_a: usize,
        node_id_b: usize,
    ) -> Result<(usize, usize), Error> {
        let sync_context = cluster.tick_chain_synchronizer(node_id_a)?;
        if sync_context.messages.is_empty() {
            return Ok((0, 0));
        }

        let node2 = cluster.get_node(node_id_b).clone();
        let message = extract_request_frame_sync_context(&sync_context, node2.id());

        run_sync_1_to_1_with_request(cluster, node_id_a, node_id_b, message)
    }

    fn run_sync_1_to_n(cluster: &mut EngineTestCluster, node_id_from: usize) -> Result<(), Error> {
        let sync_context = cluster.tick_chain_synchronizer(node_id_from)?;
        for sync_message in sync_context.messages {
            if let SyncContextMessage::ChainSyncRequest(to_node, req) = sync_message {
                let request_frame = req.as_owned_frame();
                let node_id_to = cluster.get_node_index(&to_node);
                run_sync_1_to_1_with_request(cluster, node_id_from, node_id_to, request_frame)?;
            }
        }

        Ok(())
    }

    fn run_sync_1_to_1_with_request(
        cluster: &mut EngineTestCluster,
        node_id_a: usize,
        node_id_b: usize,
        first_request: TypedCapnpFrame<Vec<u8>, chain_sync_request::Owned>,
    ) -> Result<(usize, usize), Error> {
        let node1 = cluster.get_node(node_id_a).clone();
        let node2 = cluster.get_node(node_id_b).clone();

        let mut count_1_to_2 = 0;
        let mut count_2_to_1 = 0;

        let mut request = Some(first_request);
        loop {
            count_1_to_2 += 1;
            let mut sync_context = SyncContext::new(SyncState::default());
            cluster.chains_synchronizer[node_id_b].handle_sync_request(
                &mut sync_context,
                &node1,
                &mut cluster.chains[node_id_b],
                request.take().unwrap(),
            )?;
            if sync_context.messages.is_empty() {
                break;
            }

            count_2_to_1 += 1;
            let (to_node, response) = extract_response_frame_sync_context(&sync_context);
            assert_eq!(&to_node, node1.id());
            let mut sync_context = SyncContext::new(SyncState::default());
            cluster.chains_synchronizer[node_id_a].handle_sync_response(
                &mut sync_context,
                &node2,
                &mut cluster.chains[node_id_a],
                response,
            )?;
            if sync_context.messages.is_empty() {
                break;
            }
            let message = extract_request_frame_sync_context(&sync_context, node2.id());
            request = Some(message);
        }

        Ok((count_1_to_2, count_2_to_1))
    }

    fn nodes_expect_chain_equals(
        chain1: &chain::directory::DirectoryChainStore,
        chain2: &chain::directory::DirectoryChainStore,
    ) {
        let node1_last_block = chain1
            .get_last_block()
            .unwrap()
            .expect("Node 1 didn't have any data");
        let node2_last_block = chain2
            .get_last_block()
            .unwrap()
            .expect("Node 2 didn't have any data");
        assert_eq!(node1_last_block.offset, node2_last_block.offset);
        assert_eq!(
            node1_last_block.header.whole_data(),
            node2_last_block.header.whole_data()
        );
        assert_eq!(
            node1_last_block.signatures.whole_data(),
            node2_last_block.signatures.whole_data()
        );
    }
}
