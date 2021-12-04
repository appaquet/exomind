use exocore_core::{
    cell::{Cell, CellNodes, NodeId},
    framing::CapnpFrameBuilder,
};
use exocore_protos::generated::data_transport_capnp::{
    chain_sync_request, chain_sync_response, pending_sync_request,
};
use exocore_transport::{OutMessage, ServiceType};

use super::{EngineError, Event};
use crate::block::{BlockHeight, BlockOffset};

/// Synchronization context used by `chain_sync`, `pending_sync` and
/// `commit_manager` to dispatch messages to other nodes, and dispatch events to
/// be sent to engine handles.
pub struct SyncContext {
    pub events: Vec<Event>,
    pub messages: Vec<SyncContextMessage>,
    pub sync_state: SyncState,
}

impl SyncContext {
    pub fn new(sync_state: SyncState) -> SyncContext {
        SyncContext {
            events: Vec::new(),
            messages: Vec::new(),
            sync_state,
        }
    }

    pub fn push_pending_sync_request(
        &mut self,
        node_id: NodeId,
        request_builder: CapnpFrameBuilder<pending_sync_request::Owned>,
    ) {
        self.messages.push(SyncContextMessage::PendingSyncRequest(
            node_id,
            request_builder,
        ));
    }

    pub fn push_chain_sync_request(
        &mut self,
        node_id: NodeId,
        request_builder: CapnpFrameBuilder<chain_sync_request::Owned>,
    ) {
        self.messages.push(SyncContextMessage::ChainSyncRequest(
            node_id,
            request_builder,
        ));
    }

    pub fn push_chain_sync_response(
        &mut self,
        node_id: NodeId,
        response_builder: CapnpFrameBuilder<chain_sync_response::Owned>,
    ) {
        self.messages.push(SyncContextMessage::ChainSyncResponse(
            node_id,
            response_builder,
        ));
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }
}

pub enum SyncContextMessage {
    PendingSyncRequest(NodeId, CapnpFrameBuilder<pending_sync_request::Owned>),
    ChainSyncRequest(NodeId, CapnpFrameBuilder<chain_sync_request::Owned>),
    ChainSyncResponse(NodeId, CapnpFrameBuilder<chain_sync_response::Owned>),
}

impl SyncContextMessage {
    pub fn into_out_message(self, cell: &Cell) -> Result<OutMessage, EngineError> {
        let cell_nodes = cell.nodes();
        let dest_node = cell_nodes
            .get(self.dest_node())
            .map(|n| n.node().clone())
            .ok_or_else(|| EngineError::NodeNotFound(self.dest_node().clone()))?;

        let message = match self {
            SyncContextMessage::PendingSyncRequest(_, request_builder) => {
                OutMessage::from_framed_message(cell, ServiceType::Chain, request_builder)?
                    .with_destination(dest_node)
            }
            SyncContextMessage::ChainSyncRequest(_, request_builder) => {
                OutMessage::from_framed_message(cell, ServiceType::Chain, request_builder)?
                    .with_destination(dest_node)
            }
            SyncContextMessage::ChainSyncResponse(_, response_builder) => {
                OutMessage::from_framed_message(cell, ServiceType::Chain, response_builder)?
                    .with_destination(dest_node)
            }
        };

        Ok(message)
    }

    fn dest_node(&self) -> &NodeId {
        match self {
            SyncContextMessage::PendingSyncRequest(to_node, _) => to_node,
            SyncContextMessage::ChainSyncRequest(to_node, _) => to_node,
            SyncContextMessage::ChainSyncResponse(to_node, _) => to_node,
        }
    }
}

/// State of the synchronization, used to communicate information between the
/// `ChainSynchronizer`, `CommitManager` and `PendingSynchronizer`.
#[derive(Default, Clone, Copy)]
pub struct SyncState {
    /// Indicates what is the last block that got cleaned up from pending store,
    /// and that is now only available from the chain. This is used by the
    /// `PendingSynchronizer` to know which operations it should not include
    /// anymore in its requests.
    pub pending_last_cleanup_block: Option<(BlockOffset, BlockHeight)>,
}
