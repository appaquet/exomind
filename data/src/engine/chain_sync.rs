use std::collections::HashMap;
use std::time::Instant;

use capnp::traits::ToU16;
use exocore_common::node::{Node, NodeID, Nodes};
use exocore_common::serialization::capnp;
use exocore_common::serialization::framed::{FrameBuilder, SignedFrame, TypedFrame};
use exocore_common::serialization::protos::data_chain_capnp::{block, block_header};
use exocore_common::serialization::protos::data_transport_capnp::{
    chain_sync_request, chain_sync_request::RequestedDetails, chain_sync_response,
};

use crate::chain;
use crate::chain::{Block, BlockOffset, BlockRef, ChainStore};
use crate::engine::{request_tracker, Event};
use crate::engine::{Error, SyncContext};

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
    node_id: NodeID,
    config: ChainSyncConfig,
    nodes_info: HashMap<NodeID, NodeSyncInfo>,
    status: Status,
    leader: Option<NodeID>,
    phantom: std::marker::PhantomData<CS>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Unknown,
    Downloading,
    Synchronized,
}

impl<CS: ChainStore> ChainSynchronizer<CS> {
    pub fn new(node_id: NodeID, config: ChainSyncConfig) -> ChainSynchronizer<CS> {
        ChainSynchronizer {
            node_id,
            config,
            status: Status::Unknown,
            nodes_info: HashMap::new(),
            leader: None,
            phantom: std::marker::PhantomData,
        }
    }

    ///
    /// Called at interval to make progress on the synchronization. Depending on the current synchronization
    /// status, we could be asking for more details about remote nodes' chain, or could be asking for blocks
    /// data from the lead node.
    ///
    pub fn tick<'n>(
        &mut self,
        sync_context: &mut SyncContext,
        store: &CS,
        nodes: &'n Nodes,
    ) -> Result<(), Error> {
        let node_id = self.node_id.clone();

        // TODO: Should check if sync status changed. Ticket: https://github.com/appaquet/exocore/issues/44

        let (nb_nodes_metadata_sync, nb_nodes) = self.count_nodes_status(nodes);
        let majority_nodes_metadata_sync = nodes.is_quorum(usize::from(nb_nodes_metadata_sync));
        debug!(
            "Sync tick begins. current_status={:?} nb_nodes={} nb_nodes_metadata_sync={}",
            self.status, nb_nodes, nb_nodes_metadata_sync
        );

        // check if we need to gather data from a leader node
        if self.status != Status::Synchronized && majority_nodes_metadata_sync {
            if self.leader.is_none() {
                self.find_leader_node(store)?;
            }

            let im_leader = self
                .leader
                .as_ref()
                .map_or(false, |leader| leader == &node_id);
            if im_leader {
                debug!("I'm the leader. Switching status to synchronized");
                self.status = Status::Synchronized;
            } else if let Some(leader) = self.leader.clone() {
                debug!("Leader node is {}", leader);
                let leader_node_info = self.get_or_create_node_info_mut(&leader);

                if leader_node_info.chain_fully_downloaded() {
                    info!("Changing status to synchronized, as chain is full synchronized with leader");
                    self.status = Status::Synchronized
                } else {
                    if leader_node_info.last_common_block.is_none() {
                        if let Some(last_block) = store.get_last_block()? {
                            error!("Leader node has no common block with us. Our last block is at offset {}", last_block.offset);
                            return Err(ChainSyncError::Diverged(leader).into());
                        }
                    }

                    if leader_node_info.request_tracker.can_send_request() {
                        let leader_node = nodes.get(&leader).ok_or_else(|| {
                            ChainSyncError::Other(format!(
                                "Couldn't find leader node {} in nodes list",
                                node_id
                            ))
                        })?;

                        debug!("Initiating chain download with leader");
                        let request = Self::create_sync_request(
                            leader_node_info,
                            RequestedDetails::Blocks,
                            None,
                        )?;
                        leader_node_info
                            .request_tracker
                            .set_last_send(Instant::now());
                        sync_context.push_chain_sync_request(leader_node.id().clone(), request);
                        self.status = Status::Downloading;
                    }
                }
            } else {
                error!("Couldn't find any leader node !");
            }
        }

        // synchronize chain state with nodes
        for node in nodes.nodes_except(&node_id) {
            let node_info = self.get_or_create_node_info_mut(node.id());

            if node_info.request_tracker.can_send_request() {
                let request =
                    Self::create_sync_request(node_info, RequestedDetails::Headers, None)?;
                sync_context.push_chain_sync_request(node.id().clone(), request);
                node_info.request_tracker.set_last_send(Instant::now());
            }
        }

        debug!("Sync tick ended. current_status={:?}", self.status);
        Ok(())
    }

    ///
    /// Handles an incoming sync request. This request can be for headers, or could be for blocks.
    ///
    pub fn handle_sync_request<R>(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        request: R,
    ) -> Result<(), Error>
    where
        R: TypedFrame<chain_sync_request::Owned>,
    {
        let request_reader: chain_sync_request::Reader = request.get_typed_reader()?;
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
        node_info.request_tracker.set_last_responded(Instant::now());

        if requested_details == chain_sync_request::RequestedDetails::Headers {
            let to_offset_opt = if to_offset != 0 {
                Some(to_offset)
            } else {
                None
            };

            let headers = chain_sample_block_headers(
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
    pub fn handle_sync_response<R>(
        &mut self,
        sync_context: &mut SyncContext,
        from_node: &Node,
        store: &mut CS,
        response: R,
    ) -> Result<(), Error>
    where
        R: TypedFrame<chain_sync_response::Owned>,
    {
        let response_reader: chain_sync_response::Reader = response.get_typed_reader()?;
        if response_reader.has_blocks() {
            debug!("Got blocks response from node {}", from_node.id());
            self.handle_sync_response_blocks(sync_context, from_node, store, response_reader)?;
        } else if response_reader.has_headers() {
            debug!("Got headers response from node {}", from_node.id());
            self.handle_sync_response_headers(sync_context, from_node, store, response_reader)?;
        } else {
            warn!("Got a response without headers and blocks");
        }

        // last responded is set at the end so that if we failed reading response, it's considered
        // as if we didn't receive anything (which will lead to timeout & retries)
        let node_info = self.get_or_create_node_info_mut(&from_node.id());
        node_info.request_tracker.set_last_responded(Instant::now());

        Ok(())
    }

    pub fn status(&self) -> Status {
        self.status
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
    ) -> Result<FrameBuilder<chain_sync_request::Owned>, Error> {
        let mut frame_builder = FrameBuilder::new();
        let mut request_builder: chain_sync_request::Builder = frame_builder.get_builder_typed();

        let from_offset = node_info.last_common_block.as_ref().map_or(0, |b| {
            // if we requesting blocks, we want data from next offset to prevent getting data
            // for a block we have already have
            if requested_details == RequestedDetails::Headers {
                b.offset
            } else {
                b.next_offset()
            }
        });

        let to_offset = to_offset
            .unwrap_or_else(|| node_info.last_known_block.as_ref().map_or(0, |b| b.offset));

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
        from_offset: chain::BlockOffset,
        to_offset: chain::BlockOffset,
        headers: Vec<BlockHeader>,
    ) -> Result<FrameBuilder<chain_sync_response::Owned>, Error> {
        let mut frame_builder = FrameBuilder::new();
        let mut response_builder: chain_sync_response::Builder = frame_builder.get_builder_typed();
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
        from_offset: chain::BlockOffset,
        to_offset: chain::BlockOffset,
        blocks_iter: I,
    ) -> Result<FrameBuilder<chain_sync_response::Owned>, Error> {
        let mut frame_builder = FrameBuilder::new();
        let mut response_builder: chain_sync_response::Builder = frame_builder.get_builder_typed();
        response_builder.set_from_offset(from_offset);
        response_builder.set_to_offset(to_offset);

        // accumulate blocks' data until we reach max packet size
        let mut data_size = 0;
        let blocks = blocks_iter
            .take_while(|block| {
                data_size += block.total_size();
                data_size < config.blocks_max_send_size
            })
            .collect::<Vec<_>>();
        let blocks_len = blocks.len() as u32;

        if blocks_len > 0 {
            let mut blocks_builder = response_builder.init_blocks(blocks_len);
            for i in 0..blocks_len {
                let block_and_signatures = vec![
                    blocks[i as usize].block().frame_data(),
                    blocks[i as usize].operations_data(),
                    blocks[i as usize].signatures().frame_data(),
                ]
                .concat();

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
        let mut first_non_common_block: Option<chain::BlockOffset> = None;
        for header in headers_reader.iter() {
            let header_reader: block_header::Reader = header;
            let offset = header_reader.get_offset();

            // if we haven't encountered a block we didn't have in common, we keep checking if we have
            // the block locally, and update the last_common_block
            if first_non_common_block.is_none() {
                if let Ok(local_block) = store.get_block(offset) {
                    let local_block_signature = local_block
                        .block
                        .signature_data()
                        .expect("A stored block didn't have a signature");
                    if header_reader.get_block_hash()? == local_block_signature {
                        let is_latest_common_offset = from_node_info
                            .last_common_block
                            .as_ref()
                            .map_or(true, |b| b.offset < offset);
                        if is_latest_common_offset {
                            from_node_info.last_common_block =
                                Some(BlockHeader::from_block_header_reader(header_reader)?);
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
                from_node_info.last_known_block =
                    Some(BlockHeader::from_block_header_reader(header_reader)?);
            }
        }

        if has_new_common_block {
            let to_offset = first_non_common_block;
            debug!(
                "New common ancestor block: {:?} to {:?}. Asking for more headers.",
                from_node_info.last_common_block, first_non_common_block
            );

            let request =
                Self::create_sync_request(from_node_info, RequestedDetails::Headers, to_offset)?;
            sync_context.push_chain_sync_request(from_node_info.node_id.clone(), request);
        } else {
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
        let lead_node_id = self.leader.as_ref().cloned();
        let from_node_info = self.get_or_create_node_info_mut(&from_node.id());

        let is_from_leader =
            lead_node_id.map_or(false, |lead_node_id| lead_node_id == from_node_info.node_id);
        if is_from_leader {
            // write incoming blocks
            let mut last_local_block: Option<BlockHeader> = store
                .get_last_block()?
                .map(BlockHeader::from_stored_block)
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
                    .map_or(0, BlockHeader::next_offset);
                if block.offset() == next_local_offset {
                    sync_context.push_event(Event::ChainBlockNew(block.offset()));
                    store.write_block(&block)?;
                    let new_block_header = BlockHeader::from_stored_block(block)?;
                    last_local_block = Some(new_block_header);
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
                let request =
                    Self::create_sync_request(from_node_info, RequestedDetails::Blocks, None)?;
                sync_context.push_chain_sync_request(from_node_info.node_id.clone(), request);
                from_node_info.request_tracker.set_last_send(Instant::now());
            }
        } else {
            warn!("Got data from a non-lead node {:?}", from_node_info.node_id);
        }

        Ok(())
    }

    fn get_or_create_node_info_mut(&mut self, node_id: &str) -> &mut NodeSyncInfo {
        if self.nodes_info.contains_key(node_id) {
            return self.nodes_info.get_mut(node_id).unwrap();
        }

        let config = self.config;
        self.nodes_info
            .entry(node_id.to_string())
            .or_insert_with(move || NodeSyncInfo::new(node_id.to_string(), &config))
    }

    fn count_nodes_status(&self, nodes: &Nodes) -> (u16, u16) {
        let mut nodes_total = 0;
        let mut nodes_metadata_sync = 0;
        for node in nodes.nodes() {
            nodes_total += 1;

            if node.id() == &self.node_id {
                nodes_metadata_sync += 1;
                continue;
            }

            if let Some(node_info) = self.nodes_info.get(node.id()) {
                if node_info.chain_metadata_status() == NodeMetadataStatus::Synchronized {
                    nodes_metadata_sync += 1;
                }
            }
        }

        (nodes_metadata_sync, nodes_total)
    }

    fn find_leader_node(&mut self, store: &CS) -> Result<(), Error> {
        let maybe_leader = self
            .nodes_info
            .values()
            .filter(|info| info.last_known_block.is_some())
            .map(|info| {
                let last_offset = info
                    .last_known_block
                    .as_ref()
                    .map(|b| b.offset)
                    .expect("Node should have had a last_known_block");
                (info, last_offset)
            })
            .max_by(|(_node_a, offset_a), (_node_b, offset_b)| offset_a.cmp(offset_b));

        let last_local_block = store.get_last_block()?;
        self.leader = match (maybe_leader, &last_local_block) {
            (Some((_node_info, node_offset)), Some(last_local_block))
                if last_local_block.offset > node_offset =>
            {
                // there are other nodes, but i have the longest chain
                Some(self.node_id.clone())
            }
            (None, Some(_last_local_block)) => {
                // i have at least the genesis block, i'm alone, so i'm the leader
                Some(self.node_id.clone())
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
    pub request_tracker: request_tracker::RequestTrackerConfig,
    pub headers_sync_begin_count: chain::BlockOffset,
    pub headers_sync_end_count: chain::BlockOffset,
    pub headers_sync_sampled_count: chain::BlockOffset,
    pub blocks_max_send_size: usize,
}

impl Default for ChainSyncConfig {
    fn default() -> Self {
        ChainSyncConfig {
            request_tracker: request_tracker::RequestTrackerConfig::default(),
            headers_sync_begin_count: 5,
            headers_sync_end_count: 5,
            headers_sync_sampled_count: 10,
            blocks_max_send_size: 50 * 1024,
        }
    }
}

///
/// Synchronization information about a remote node
///
struct NodeSyncInfo {
    node_id: NodeID,

    last_common_block: Option<BlockHeader>,
    last_common_is_known: bool,
    last_known_block: Option<BlockHeader>,

    request_tracker: request_tracker::RequestTracker,
}

impl NodeSyncInfo {
    fn new(node_id: NodeID, config: &ChainSyncConfig) -> NodeSyncInfo {
        NodeSyncInfo {
            node_id,

            last_common_block: None,
            last_common_is_known: false,
            last_known_block: None,

            request_tracker: request_tracker::RequestTracker::new(config.request_tracker),
        }
    }

    fn chain_metadata_status(&self) -> NodeMetadataStatus {
        if self.last_common_is_known {
            NodeMetadataStatus::Synchronized
        } else {
            NodeMetadataStatus::Unknown
        }
    }

    fn chain_fully_downloaded(&self) -> bool {
        let last_known_offset = self.last_known_block.as_ref().map(|b| b.offset);
        let last_common_offset = self.last_common_block.as_ref().map(|b| b.offset);
        self.last_known_block.is_some() && last_known_offset == last_common_offset
    }
}

#[derive(Debug, PartialEq, Clone)]
enum NodeMetadataStatus {
    Unknown,
    Synchronized,
}

///
/// Abstracts block's header coming from local store or remote node
///
#[derive(Debug)]
struct BlockHeader {
    offset: chain::BlockOffset,
    depth: chain::BlockDepth,
    hash: Vec<u8>,
    previous_offset: chain::BlockOffset,
    previous_hash: Vec<u8>,

    block_size: u32,
    operations_size: u32,
    signatures_size: chain::BlockSignaturesSize,
}

impl BlockHeader {
    fn from_stored_block<B: Block>(stored_block: B) -> Result<BlockHeader, Error> {
        let block_reader: block::Reader = stored_block.block().get_typed_reader()?;
        let block_signature = stored_block
            .block()
            .signature_data()
            .expect("A stored block didn't signature data");

        Ok(BlockHeader {
            offset: stored_block.offset(),
            depth: block_reader.get_depth(),
            hash: block_signature.to_vec(),
            previous_offset: block_reader.get_previous_offset(),
            previous_hash: block_reader.get_previous_hash()?.to_vec(),

            block_size: stored_block.block().frame_size() as u32,
            operations_size: stored_block.operations_data().len() as u32,
            signatures_size: stored_block.signatures().frame_size() as chain::BlockSignaturesSize,
        })
    }

    fn from_block_header_reader(
        block_header_reader: block_header::Reader,
    ) -> Result<BlockHeader, Error> {
        Ok(BlockHeader {
            offset: block_header_reader.get_offset(),
            depth: block_header_reader.get_depth(),
            hash: block_header_reader.get_block_hash()?.to_vec(),
            previous_offset: block_header_reader.get_previous_offset(),
            previous_hash: block_header_reader.get_previous_hash()?.to_vec(),
            block_size: block_header_reader.get_block_size(),
            operations_size: block_header_reader.get_operations_size(),
            signatures_size: block_header_reader.get_signatures_size(),
        })
    }

    #[inline]
    fn next_offset(&self) -> chain::BlockOffset {
        self.offset
            + chain::BlockOffset::from(self.block_size)
            + chain::BlockOffset::from(self.operations_size)
            + chain::BlockOffset::from(self.signatures_size)
    }

    fn copy_into_builder(&self, builder: &mut block_header::Builder) {
        builder.set_offset(self.offset);
        builder.set_depth(self.depth);
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
#[derive(Debug, Fail)]
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
/// Samples the local chain and returns a collection of `BlockHeader` at different position in the asked range.
///
/// `from_offset` and `to_offset` are best efforts and fallback to begin/end of chain if they don't exist.
/// `begin_count` and `end_count` are number of headers to include without sampling from beginning and end of range.
/// `sampled_count` is the approximate number of headers to return, excluding the `begin_count` and `end_count`
///
fn chain_sample_block_headers<CS: chain::ChainStore>(
    store: &CS,
    from_offset: chain::BlockOffset,
    to_offset: Option<chain::BlockOffset>,
    begin_count: chain::BlockOffset,
    end_count: chain::BlockOffset,
    sampled_count: chain::BlockOffset,
) -> Result<Vec<BlockHeader>, Error> {
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

    let last_block_reader: block::Reader = last_block.block.get_typed_reader()?;
    let last_block_depth = last_block_reader.get_depth();

    let mut blocks_iter = store
        .blocks_iter(from_offset)
        .or_else(|_| store.blocks_iter(0))?
        .peekable();

    let first_block = blocks_iter.peek().ok_or_else(|| {
        ChainSyncError::Other("Expected a first block since ranges were not empty".to_string())
    })?;
    let first_block_reader: block::Reader = first_block.block.get_typed_reader()?;
    let first_block_depth = first_block_reader.get_depth();

    let range_blocks_count = last_block_depth - first_block_depth;
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
        let blocks_count = blocks_count as chain::BlockOffset;
        if blocks_count < begin_count
            || blocks_count > range_blocks_lasts
            || blocks_count % range_blocks_skip == 0
        {
            let block_header = BlockHeader::from_stored_block(current_block)?;
            headers.push(block_header);
        }
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chain::directory::DirectoryChainStore;
    use exocore_common::serialization::framed::OwnedTypedFrame;

    use crate::engine::testing::*;
    use crate::engine::SyncContextMessage;

    #[test]
    fn test_handle_sync_response_blocks() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(2);
        cluster.chain_generate_dummy(0, 10, 1234);
        cluster.chain_generate_dummy(1, 100, 1234);
        let node0 = cluster.get_node(0);
        let node1 = cluster.get_node(1);

        test_nodes_run_sync_new(&mut cluster, 0, 1)?;
        cluster.tick_chain_synchronizer(0)?;
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Downloading);
        assert_eq!(
            cluster.chains_synchronizer[0].leader,
            Some("node1".to_string())
        );

        // response from non-leader should just not reply
        let response = FrameBuilder::new();
        let response_frame = response.as_owned_unsigned_framed()?;
        let mut sync_context = SyncContext::new();
        cluster.chains_synchronizer[0].handle_sync_response(
            &mut sync_context,
            &node0,
            &mut cluster.chains[0],
            response_frame,
        )?;
        assert!(sync_context.messages.is_empty());

        // response from leader with blocks that aren't next should fail
        let blocks_iter = cluster.chains[1].blocks_iter(0)?;
        let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
            &cluster.chains_synchronizer[1].config,
            10,
            0,
            blocks_iter,
        )?;
        let response_frame = response.as_owned_unsigned_framed()?;
        let mut sync_context = SyncContext::new();
        let result = cluster.chains_synchronizer[0].handle_sync_response(
            &mut sync_context,
            &node1,
            &mut cluster.chains[0],
            response_frame,
        );
        assert!(result.is_err());

        // response from leader with blocks at right position should suceed and append
        let blocks_iter = cluster.chains[1].blocks_iter(0).unwrap().skip(10); // skip 10 will go to 10th block
        let response = ChainSynchronizer::<DirectoryChainStore>::create_sync_response_for_blocks(
            &cluster.chains_synchronizer[0].config,
            10,
            0,
            blocks_iter,
        )?;
        let response_frame = response.as_owned_unsigned_framed().unwrap();
        let mut sync_context = SyncContext::new();
        cluster.chains_synchronizer[0].handle_sync_response(
            &mut sync_context,
            &node1,
            &mut cluster.chains[0],
            response_frame,
        )?;

        Ok(())
    }

    #[test]
    fn test_chain_sample_block_headers() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(1);
        cluster.chain_generate_dummy(0, 100, 3424);

        let offsets: Vec<chain::BlockOffset> = cluster.chains[0]
            .blocks_iter(0)?
            .map(|b| b.offset)
            .collect();

        let headers = chain_sample_block_headers(&cluster.chains[0], 0, None, 2, 2, 10)?;
        assert_eq!(
            headers.iter().map(|b| b.depth).collect::<Vec<_>>(),
            vec![0, 1, 9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 98, 99]
        );

        let headers = chain_sample_block_headers(&cluster.chains[0], 0, None, 0, 0, 1)?;
        assert_eq!(
            headers.iter().map(|b| b.depth).collect::<Vec<_>>(),
            vec![0, 99]
        );

        let headers = chain_sample_block_headers(&cluster.chains[0], offsets[10], None, 5, 5, 10)?;
        assert_eq!(
            headers.iter().map(|b| b.depth).collect::<Vec<_>>(),
            vec![10, 11, 12, 13, 14, 18, 26, 34, 42, 50, 58, 66, 74, 82, 90, 95, 96, 97, 98, 99]
        );

        let headers = chain_sample_block_headers(
            &cluster.chains[0],
            offsets[10],
            Some(offsets[50]),
            2,
            2,
            5,
        )?;
        assert_eq!(
            headers.iter().map(|b| b.depth).collect::<Vec<_>>(),
            vec![10, 11, 18, 26, 34, 42, 49, 50]
        );

        Ok(())
    }

    #[test]
    fn sync_empty_node1_to_full_node2() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(2);
        cluster.chain_generate_dummy(1, 100, 3434);

        test_nodes_run_sync_new(&mut cluster, 0, 1)?;
        {
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info["node1"];
            assert_eq!(
                node1_node2_info.chain_metadata_status(),
                NodeMetadataStatus::Synchronized
            );
            assert_eq!(
                node1_node2_info.last_common_block.as_ref().map(|b| b.depth),
                None
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.depth),
                Some(99)
            );
        }

        // this will sync blocks & mark as synchronized
        test_nodes_run_sync_new(&mut cluster, 0, 1)?;
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);
        assert_eq!(
            cluster.chains_synchronizer[0].leader,
            Some("node1".to_string())
        );

        // force status back to downloading to check if tick will turn back to synchronized
        cluster.chains_synchronizer[0].status = Status::Downloading;
        test_nodes_run_sync_new(&mut cluster, 0, 1)?;
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        test_nodes_expect_chain_equals_new(&cluster.chains[0], &cluster.chains[1]);

        Ok(())
    }

    #[test]
    fn sync_full_node1_to_empty_node2() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(2);
        cluster.chain_generate_dummy(0, 100, 3434);

        // running sync twice will yield to nothing as node2 is empty
        for _i in 0..2 {
            test_nodes_run_sync_new(&mut cluster, 0, 1)?;
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info["node1"];
            assert_eq!(
                node1_node2_info.chain_metadata_status(),
                NodeMetadataStatus::Synchronized
            );
            assert_eq!(
                node1_node2_info.last_common_block.as_ref().map(|b| b.depth),
                None
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.depth),
                None
            );
        }

        // node1 is full, it has quorum (1 out of 2 nodes >= 50%)
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        Ok(())
    }

    #[test]
    fn sync_full_node1_to_half_node2() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(2);
        cluster.chain_generate_dummy(0, 100, 3434);
        cluster.chain_generate_dummy(1, 50, 3434);

        // running sync twice will yield to nothing as node1 is leader
        for _i in 0..2 {
            test_nodes_run_sync_new(&mut cluster, 0, 1)?;
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info["node1"];
            assert_eq!(
                node1_node2_info.chain_metadata_status(),
                NodeMetadataStatus::Synchronized
            );
            assert_eq!(
                node1_node2_info.last_common_block.as_ref().map(|b| b.depth),
                Some(49)
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.depth),
                Some(49)
            );
        }

        // we're leader and synchronized because of it
        assert_eq!(
            cluster.chains_synchronizer[0].leader,
            Some("node0".to_string())
        );
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        Ok(())
    }

    #[test]
    fn sync_half_node1_to_full_node2() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(2);
        cluster.chain_generate_dummy(0, 50, 3434);
        cluster.chain_generate_dummy(1, 100, 3434);

        test_nodes_run_sync_new(&mut cluster, 0, 1)?;
        {
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info["node1"];
            assert_eq!(
                node1_node2_info.chain_metadata_status(),
                NodeMetadataStatus::Synchronized
            );
            assert_eq!(
                node1_node2_info.last_common_block.as_ref().map(|b| b.depth),
                Some(49)
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.depth),
                Some(99)
            );
        }

        // this will sync blocks & mark as synchronized
        test_nodes_run_sync_new(&mut cluster, 0, 1)?;

        // node2 is leader
        assert_eq!(
            cluster.chains_synchronizer[0].leader,
            Some("node1".to_string())
        );
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Synchronized);

        test_nodes_expect_chain_equals_new(&cluster.chains[0], &cluster.chains[1]);

        Ok(())
    }

    #[test]
    fn sync_divergent_node1_to_full_node2() -> Result<(), failure::Error> {
        let mut cluster = TestCluster::new(2);
        cluster.chain_generate_dummy(0, 100, 1234);
        cluster.chain_generate_dummy(1, 100, 9876);

        test_nodes_run_sync_new(&mut cluster, 0, 1)?;
        {
            let node1_node2_info = &cluster.chains_synchronizer[0].nodes_info["node1"];
            assert_eq!(
                node1_node2_info.chain_metadata_status(),
                NodeMetadataStatus::Synchronized
            );
            assert_eq!(
                node1_node2_info.last_common_block.as_ref().map(|b| b.depth),
                None,
            );
            assert_eq!(
                node1_node2_info.last_known_block.as_ref().map(|b| b.depth),
                Some(99),
            );
        }

        match test_nodes_run_sync_new(&mut cluster, 0, 1).err() {
            Some(Error::ChainSync(ChainSyncError::Diverged(_))) => {}
            other => panic!("Expected a diverged error, got {:?}", other),
        }

        // still unknown since we don't have a clear leader, as we've diverged from it
        assert_eq!(cluster.chains_synchronizer[0].status, Status::Unknown);

        Ok(())
    }

    fn extract_request_frame_sync_context(
        sync_context: &SyncContext,
    ) -> (NodeID, OwnedTypedFrame<chain_sync_request::Owned>) {
        match sync_context.messages.last().unwrap() {
            SyncContextMessage::ChainSyncRequest(to_node, req) => {
                (to_node.clone(), req.as_owned_unsigned_framed().unwrap())
            }
            _other => panic!("Expected a chain sync request, got another type of message"),
        }
    }

    fn extract_response_frame_sync_context(
        sync_context: &SyncContext,
    ) -> (NodeID, OwnedTypedFrame<chain_sync_response::Owned>) {
        match sync_context.messages.last().unwrap() {
            SyncContextMessage::ChainSyncResponse(to_node, req) => {
                (to_node.clone(), req.as_owned_unsigned_framed().unwrap())
            }
            _other => panic!("Expected a chain sync response, got another type of message"),
        }
    }

    fn test_nodes_run_sync_new(
        cluster: &mut TestCluster,
        node_id_a: usize,
        node_id_b: usize,
    ) -> Result<(usize, usize), Error> {
        let node1 = cluster.get_node(node_id_a).clone();
        let node2 = cluster.get_node(node_id_b).clone();

        let mut count_1_to_2 = 0;
        let mut count_2_to_1 = 0;

        let sync_context = cluster.tick_chain_synchronizer(node_id_a)?;
        if sync_context.messages.is_empty() {
            return Ok((count_1_to_2, count_2_to_1));
        }

        let (to_node, message) = extract_request_frame_sync_context(&sync_context);
        assert_eq!(&to_node, node2.id());
        let mut request = Some(message);
        loop {
            count_1_to_2 += 1;
            let mut sync_context = SyncContext::new();
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
            let mut sync_context = SyncContext::new();
            cluster.chains_synchronizer[node_id_a].handle_sync_response(
                &mut sync_context,
                &node2,
                &mut cluster.chains[node_id_a],
                response,
            )?;
            if sync_context.messages.is_empty() {
                break;
            }
            let (to_node, message) = extract_request_frame_sync_context(&sync_context);
            assert_eq!(&to_node, node2.id());
            request = Some(message);
        }

        Ok((count_1_to_2, count_2_to_1))
    }

    fn test_nodes_expect_chain_equals_new(
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
            node1_last_block.block.frame_data(),
            node2_last_block.block.frame_data()
        );
        assert_eq!(
            node1_last_block.signatures.frame_data(),
            node2_last_block.signatures.frame_data()
        );
    }
}
