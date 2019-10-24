use crate::block::{BlockHeight, BlockOffset};
use crate::chain;
use crate::operation;
use crate::operation::NewOperation;
use crate::pending;
use exocore_common;
use exocore_common::cell::{Cell, CellNodes};
use exocore_common::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_common::node::NodeId;
use exocore_common::protos::data_transport_capnp::{
    chain_sync_request, chain_sync_response, pending_sync_request,
};
use exocore_common::protos::MessageType;
use exocore_common::time::Clock;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_transport::{
    Error as TransportError, InEvent, InMessage, OutEvent, OutMessage, TransportHandle,
    TransportLayer,
};
use futures::prelude::*;
use futures::sync::mpsc;
use std;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;
use tokio;
use tokio::timer::Interval;

mod chain_sync;
mod commit_manager;
mod error;
mod handle;
mod pending_sync;
mod request_tracker;
#[cfg(test)]
pub(crate) mod testing;

pub use chain_sync::ChainSyncConfig;
pub use commit_manager::CommitManagerConfig;
pub use error::Error;
pub use handle::{EngineHandle, EngineOperation, EngineOperationStatus, Event};
pub use pending_sync::PendingSyncConfig;
pub use request_tracker::RequestTrackerConfig;

///
/// Data engine's configuration
///
#[derive(Copy, Clone)]
pub struct Config {
    pub chain_synchronizer_config: ChainSyncConfig,
    pub pending_synchronizer_config: PendingSyncConfig,
    pub commit_manager_config: CommitManagerConfig,
    pub manager_timer_interval: Duration,
    pub events_stream_buffer_size: usize,
    pub to_transport_channel_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_synchronizer_config: ChainSyncConfig::default(),
            pending_synchronizer_config: PendingSyncConfig::default(),
            commit_manager_config: CommitManagerConfig::default(),
            manager_timer_interval: Duration::from_secs(1),
            events_stream_buffer_size: 1000,
            to_transport_channel_size: 3000,
        }
    }
}

///
/// The data engine manages storage and replication of data among the nodes of the cell.
///
/// It contains 2 stores:
///   * Pending store: temporary store in which operations are stored until they get commited to chain
///   * Chain store: persistent store using a block-chain like data structure
///
pub struct Engine<T, CS, PS>
where
    T: TransportHandle,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    start_notifier: CompletionNotifier<(), Error>,
    config: Config,
    transport: Option<T>,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    stop_listener: CompletionListener<(), Error>,
}

impl<T, CS, PS> Engine<T, CS, PS>
where
    T: TransportHandle,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    pub fn new(
        config: Config,
        clock: Clock,
        transport: T,
        chain_store: CS,
        pending_store: PS,
        cell: Cell,
    ) -> Engine<T, CS, PS> {
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();
        let start_notifier = CompletionNotifier::new();

        let pending_synchronizer = pending_sync::PendingSynchronizer::new(
            config.pending_synchronizer_config,
            cell.clone(),
            clock.clone(),
        );
        let chain_synchronizer = chain_sync::ChainSynchronizer::new(
            config.chain_synchronizer_config,
            cell.clone(),
            clock.clone(),
        );
        let commit_manager = commit_manager::CommitManager::new(
            config.commit_manager_config,
            cell.clone(),
            clock.clone(),
        );

        let inner = Arc::new(RwLock::new(Inner {
            config,
            cell,
            clock: clock.clone(),
            pending_store,
            pending_synchronizer,
            chain_store,
            chain_synchronizer,
            commit_manager,
            events_stream_sender: Vec::new(),
            handles_next_id: 0,
            handles_count: 0,
            transport_sender: None,
            sync_state: SyncState::default(),
            stop_notifier,
        }));

        Engine {
            start_notifier,
            config,
            inner,
            transport: Some(transport),
            stop_listener,
        }
    }

    pub fn get_handle(&mut self) -> EngineHandle<CS, PS> {
        let mut unlocked_inner = self
            .inner
            .write()
            .expect("Inner couldn't get locked, but engine isn't even started yet.");

        let start_listener = self
            .start_notifier
            .get_listener()
            .expect("Couldn't get start listener for handle");

        let stop_listener = unlocked_inner
            .stop_notifier
            .get_listener()
            .expect("Couldn't get stop listener for handle");

        let handle_id = unlocked_inner.get_new_handle_id();
        EngineHandle::new(
            handle_id,
            Arc::downgrade(&self.inner),
            start_listener,
            stop_listener,
        )
    }

    fn start(&mut self) -> Result<(), Error> {
        let mut transport = self
            .transport
            .take()
            .ok_or_else(|| Error::Other("Transport was none in engine".to_string()))?;

        let transport_in_stream = transport.get_stream();
        let transport_out_sink = transport.get_sink();

        // create channel to send messages to transport's sink
        {
            let weak_inner = Arc::downgrade(&self.inner);
            let (transport_out_channel_sender, transport_out_channel_receiver) =
                mpsc::channel(self.config.to_transport_channel_size);
            tokio::spawn(
                transport_out_channel_receiver
                    .map_err(|err| {
                        TransportError::Other(format!(
                            "Couldn't send to transport_out channel's receiver: {:?}",
                            err
                        ))
                    })
                    .forward(transport_out_sink)
                    .map(|_| ())
                    .map_err(move |err| {
                        Self::handle_spawned_future_error(
                            "transport incoming stream",
                            &weak_inner,
                            Error::Transport(err),
                        );
                    }),
            );

            let mut unlocked_inner = self.inner.write()?;
            unlocked_inner.transport_sender = Some(transport_out_channel_sender);
        }

        // handle transport's incoming messages
        {
            let weak_inner1 = Arc::downgrade(&self.inner);
            let weak_inner2 = Arc::downgrade(&self.inner);
            tokio::spawn(
                transport_in_stream
                    .map_err(Error::Transport)
                    .for_each(move |event| {
                        match event {
                            InEvent::Message(msg) => {
                                Self::handle_incoming_message(&weak_inner1, msg)
                                    .map_err(|err| {
                                        error!("Error handling incoming message: {}", err);
                                        err
                                    })
                                    .or_else(Error::recover_non_fatal_error)
                            }
                            InEvent::NodeStatus(_, _) => {
                                // TODO: Do something with the node status

                                Ok(())
                            }
                        }
                    })
                    .map_err(move |err| {
                        Self::handle_spawned_future_error(
                            "transport incoming stream",
                            &weak_inner2,
                            err,
                        );
                    }),
            );
        }

        // management timer
        {
            let weak_inner1 = Arc::downgrade(&self.inner);
            let weak_inner2 = Arc::downgrade(&self.inner);
            tokio::spawn({
                Interval::new_interval(self.config.manager_timer_interval)
                    .map_err(|err| Error::Other(format!("Interval error: {}", err)))
                    .for_each(move |_| {
                        Self::handle_management_timer_tick(&weak_inner1)
                            .map_err(|err| {
                                error!("Error in management timer tick: {}", err);
                                err
                            })
                            .or_else(Error::recover_non_fatal_error)
                    })
                    .map_err(move |err| {
                        Self::handle_spawned_future_error("management timer", &weak_inner2, err);
                    })
            });
        }

        // schedule transport
        {
            let weak_inner1 = Arc::downgrade(&self.inner);
            tokio::spawn({
                transport.map_err(move |err| {
                    Self::handle_spawned_future_error("transport", &weak_inner1, err.into());
                })
            });
        }

        {
            let mut unlocked_inner = self.inner.write()?;
            let chain_last_block = unlocked_inner.chain_store.get_last_block()?;
            if chain_last_block.is_none() {
                warn!("{}: Chain has not been initialized (no genesis block). May not be able to start if no other nodes are found.",
                      unlocked_inner.cell.local_node().id(),
                )
            }
            unlocked_inner.dispatch_event(&Event::Started);
        }

        info!("Engine started!");
        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        message: Box<InMessage>,
    ) -> Result<(), Error> {
        let locked_inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut inner = locked_inner.write()?;

        debug!(
            "{}: Got message of type {} from node {}",
            inner.cell.local_node().id(),
            message.message_type,
            message.from.id(),
        );

        match message.message_type {
            <pending_sync_request::Owned as MessageType>::MESSAGE_TYPE => {
                let sync_request = message.get_data_as_framed_message()?;
                inner.handle_incoming_pending_sync_request(&message, sync_request)?;
            }
            <chain_sync_request::Owned as MessageType>::MESSAGE_TYPE => {
                let sync_request = message.get_data_as_framed_message()?;
                inner.handle_incoming_chain_sync_request(&message, sync_request)?;
            }
            <chain_sync_response::Owned as MessageType>::MESSAGE_TYPE => {
                let sync_response = message.get_data_as_framed_message()?;
                inner.handle_incoming_chain_sync_response(&message, sync_response)?;
            }
            msg_type => {
                return Err(Error::Other(format!(
                    "Got an unknown message type: message_type={} transport_layer={:?}",
                    msg_type, message.layer,
                )));
            }
        }

        Ok(())
    }

    fn handle_management_timer_tick(weak_inner: &Weak<RwLock<Inner<CS, PS>>>) -> Result<(), Error> {
        let locked_inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut inner = locked_inner.write()?;

        inner.tick_synchronizers()?;

        Ok(())
    }

    fn handle_spawned_future_error(
        future_name: &str,
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        error: Error,
    ) {
        error!("Got an error in future {}: {}", future_name, error);

        let locked_inner = if let Some(locked_inner) = weak_inner.upgrade() {
            locked_inner
        } else {
            return;
        };

        let inner = if let Ok(inner) = locked_inner.read() {
            inner
        } else {
            return;
        };

        inner.stop_notifier.complete(Err(Error::Other(format!(
            "Couldn't send to completion channel: {:?}",
            error
        ))));
    }
}

impl<T, CS, PS> Future for Engine<T, CS, PS>
where
    T: TransportHandle,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<()>, Error> {
        // first, make sure transport is started
        if let Some(transport_handle) = &self.transport {
            try_ready!(transport_handle.on_start().poll());
        }

        // start the engine if it's not started
        if !self.start_notifier.is_complete() {
            let start_res = self.start();
            self.start_notifier.complete(start_res);
        }

        // check if engine got stopped
        self.stop_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            _ => Error::Other("Error in completion error".to_string()),
        })
    }
}

///
/// Inner instance of the engine, since the engine is owned by the executor. The executor owns a strong
/// reference to this Inner, while handles own weak references.
///
pub(crate) struct Inner<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    config: Config,
    cell: Cell,
    clock: Clock,
    pending_store: PS,
    pending_synchronizer: pending_sync::PendingSynchronizer<PS>,
    chain_store: CS,
    chain_synchronizer: chain_sync::ChainSynchronizer<CS>,
    commit_manager: commit_manager::CommitManager<PS, CS>,
    events_stream_sender: Vec<(usize, bool, mpsc::Sender<Event>)>,
    handles_next_id: usize,
    handles_count: usize,
    transport_sender: Option<mpsc::Sender<OutEvent>>,
    sync_state: SyncState,
    stop_notifier: CompletionNotifier<(), Error>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn handle_new_operation(&mut self, operation: NewOperation) -> Result<(), Error> {
        let mut sync_context = SyncContext::new(self.sync_state);
        self.pending_synchronizer.handle_new_operation(
            &mut sync_context,
            &mut self.pending_store,
            operation,
        )?;
        self.sync_state = sync_context.sync_state;

        // to prevent sending operations that may have already been committed, we don't propagate
        // pending store changes unless the chain is synchronized
        if self.chain_is_synchronized() {
            self.send_messages_from_sync_context(&mut sync_context)?;
        }

        self.dispatch_events_from_sync_context(&sync_context);

        Ok(())
    }

    fn handle_incoming_pending_sync_request<R: FrameReader>(
        &mut self,
        message: &InMessage,
        request: TypedCapnpFrame<R, pending_sync_request::Owned>,
    ) -> Result<(), Error> {
        // to prevent sending operations that may have already been committed, we don't accept
        // any pending sync requests until the chain is synchronized
        if !self.chain_is_synchronized() {
            return Ok(());
        }

        let mut sync_context = SyncContext::new(self.sync_state);
        self.pending_synchronizer.handle_incoming_sync_request(
            &message.from,
            &mut sync_context,
            &mut self.pending_store,
            request,
        )?;
        self.sync_state = sync_context.sync_state;

        self.send_messages_from_sync_context(&mut sync_context)?;
        self.dispatch_events_from_sync_context(&sync_context);

        Ok(())
    }

    fn handle_incoming_chain_sync_request<F: FrameReader>(
        &mut self,
        message: &InMessage,
        request: TypedCapnpFrame<F, chain_sync_request::Owned>,
    ) -> Result<(), Error> {
        let mut sync_context = SyncContext::new(self.sync_state);
        self.chain_synchronizer.handle_sync_request(
            &mut sync_context,
            &message.from,
            &mut self.chain_store,
            request,
        )?;
        self.sync_state = sync_context.sync_state;

        self.send_messages_from_sync_context(&mut sync_context)?;
        self.dispatch_events_from_sync_context(&sync_context);

        Ok(())
    }

    fn handle_incoming_chain_sync_response<F: FrameReader>(
        &mut self,
        message: &InMessage,
        response: TypedCapnpFrame<F, chain_sync_response::Owned>,
    ) -> Result<(), Error> {
        let mut sync_context = SyncContext::new(self.sync_state);
        self.chain_synchronizer.handle_sync_response(
            &mut sync_context,
            &message.from,
            &mut self.chain_store,
            response,
        )?;
        self.sync_state = sync_context.sync_state;

        self.send_messages_from_sync_context(&mut sync_context)?;
        self.dispatch_events_from_sync_context(&sync_context);

        Ok(())
    }

    fn tick_synchronizers(&mut self) -> Result<(), Error> {
        let mut sync_context = SyncContext::new(self.sync_state);

        self.chain_synchronizer
            .tick(&mut sync_context, &self.chain_store)?;

        // to prevent synchronizing operations that may have added to the chain, we should only
        // start doing commit management & pending synchronization once the chain is synchronized
        if self.chain_is_synchronized() {
            // commit manager should always be ticked before pending synchronizer so that it may
            // remove operations that don't need to be synchronized anymore (ex: been committed)
            self.commit_manager.tick(
                &mut sync_context,
                &mut self.pending_synchronizer,
                &mut self.pending_store,
                &mut self.chain_store,
            )?;

            self.pending_synchronizer
                .tick(&mut sync_context, &self.pending_store)?;
        }

        self.sync_state = sync_context.sync_state;

        self.send_messages_from_sync_context(&mut sync_context)?;
        self.dispatch_events_from_sync_context(&sync_context);

        Ok(())
    }

    fn send_messages_from_sync_context(
        &mut self,
        sync_context: &mut SyncContext,
    ) -> Result<(), Error> {
        if sync_context.messages.is_empty() {
            return Ok(());
        }

        // swap out messages from the sync_context struct to consume them
        let mut messages = Vec::new();
        std::mem::swap(&mut sync_context.messages, &mut messages);

        for message in messages {
            let out_message = message.into_out_message(&self.cell)?;
            let transport_sender = self.transport_sender.as_mut().expect(
                "Transport sender was none, which mean that the transport was never started",
            );

            if let Err(err) = transport_sender.try_send(OutEvent::Message(out_message)) {
                error!(
                    "Error sending message from sync context to transport: {}",
                    err
                );
            }
        }

        Ok(())
    }

    fn get_new_handle_id(&mut self) -> usize {
        let id = self.handles_next_id;
        self.handles_next_id += 1;
        self.handles_count += 1;
        id
    }

    fn get_new_events_stream(&mut self, handle_id: usize) -> mpsc::Receiver<Event> {
        let channel_size = self.config.events_stream_buffer_size;
        let (events_sender, events_receiver) = mpsc::channel(channel_size);
        self.events_stream_sender
            .push((handle_id, false, events_sender));

        events_receiver
    }

    fn dispatch_events_from_sync_context(&mut self, sync_context: &SyncContext) {
        for event in sync_context.events.iter() {
            self.dispatch_event(&event)
        }
    }

    fn dispatch_event(&mut self, event: &Event) {
        for (handle_id, discontinued, stream_sender) in self.events_stream_sender.iter_mut() {
            // if we hit a full buffer at last send, the stream got a discontinuity and we need to advise consumer.
            // we try to emit a discontinuity event, and if we succeed (buffer has space), we try to send the next event
            if *discontinued {
                if let Ok(()) = stream_sender.try_send(Event::StreamDiscontinuity) {
                    *discontinued = false;
                } else {
                    continue;
                }
            }

            match stream_sender.try_send(event.clone()) {
                Ok(()) => {}
                Err(ref err) if err.is_full() => {
                    error!("Couldn't send event to handle {} because channel buffer is full. Marking as discontinued", handle_id);
                    *discontinued = true;
                }
                Err(err) => {
                    error!(
                        "Couldn't send event to handle {} for a reason other than channel buffer full: {:}",
                        handle_id,
                        err
                    );
                }
            }
        }
    }

    fn chain_is_synchronized(&self) -> bool {
        let chain_sync_status = self.chain_synchronizer.status();
        chain_sync_status == chain_sync::Status::Synchronized
    }

    fn unregister_handle(&mut self, handle_id: usize) {
        // remove all streams that this handle created
        let mut previous_streams = Vec::new();
        std::mem::swap(&mut self.events_stream_sender, &mut previous_streams);
        for (one_handle_id, discontinued, sender) in previous_streams {
            if one_handle_id != handle_id {
                self.events_stream_sender
                    .push((handle_id, discontinued, sender));
            }
        }

        // if it was last handle, we kill the engine
        self.handles_count -= 1;
        if self.handles_count == 0 {
            debug!("Last engine handle got dropped, killing the engine.");
            self.stop_notifier.complete(Ok(()));
        }
    }
}

///
/// Synchronization context used by `chain_sync`, `pending_sync` and `commit_manager` to dispatch
/// messages to other nodes, and dispatch events to be sent to engine handles.
///
struct SyncContext {
    events: Vec<Event>,
    messages: Vec<SyncContextMessage>,
    sync_state: SyncState,
}

impl SyncContext {
    fn new(sync_state: SyncState) -> SyncContext {
        SyncContext {
            events: Vec::new(),
            messages: Vec::new(),
            sync_state,
        }
    }

    fn push_pending_sync_request(
        &mut self,
        node_id: NodeId,
        request_builder: CapnpFrameBuilder<pending_sync_request::Owned>,
    ) {
        self.messages.push(SyncContextMessage::PendingSyncRequest(
            node_id,
            request_builder,
        ));
    }

    fn push_chain_sync_request(
        &mut self,
        node_id: NodeId,
        request_builder: CapnpFrameBuilder<chain_sync_request::Owned>,
    ) {
        self.messages.push(SyncContextMessage::ChainSyncRequest(
            node_id,
            request_builder,
        ));
    }

    fn push_chain_sync_response(
        &mut self,
        node_id: NodeId,
        response_builder: CapnpFrameBuilder<chain_sync_response::Owned>,
    ) {
        self.messages.push(SyncContextMessage::ChainSyncResponse(
            node_id,
            response_builder,
        ));
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }
}

enum SyncContextMessage {
    PendingSyncRequest(NodeId, CapnpFrameBuilder<pending_sync_request::Owned>),
    ChainSyncRequest(NodeId, CapnpFrameBuilder<chain_sync_request::Owned>),
    ChainSyncResponse(NodeId, CapnpFrameBuilder<chain_sync_response::Owned>),
}

impl SyncContextMessage {
    fn into_out_message(self, cell: &Cell) -> Result<OutMessage, Error> {
        let cell_nodes = cell.nodes();
        let to_nodes = if let Some(node) = cell_nodes.get(self.to_node()) {
            vec![node.clone()]
        } else {
            vec![]
        };

        let message = match self {
            SyncContextMessage::PendingSyncRequest(_, request_builder) => {
                OutMessage::from_framed_message(cell, TransportLayer::Data, request_builder)?
                    .with_to_nodes(to_nodes)
            }
            SyncContextMessage::ChainSyncRequest(_, request_builder) => {
                OutMessage::from_framed_message(cell, TransportLayer::Data, request_builder)?
                    .with_to_nodes(to_nodes)
            }
            SyncContextMessage::ChainSyncResponse(_, response_builder) => {
                OutMessage::from_framed_message(cell, TransportLayer::Data, response_builder)?
                    .with_to_nodes(to_nodes)
            }
        };

        Ok(message)
    }

    fn to_node(&self) -> &NodeId {
        match self {
            SyncContextMessage::PendingSyncRequest(to_node, _) => to_node,
            SyncContextMessage::ChainSyncRequest(to_node, _) => to_node,
            SyncContextMessage::ChainSyncResponse(to_node, _) => to_node,
        }
    }
}

///
/// State of the synchronization, used to communicate information between the `ChainSynchronizer`,
/// `CommitManager` and `PendingSynchronizer`.
///
#[derive(Clone, Copy)]
struct SyncState {
    ///
    /// Indicates what is the last block that got cleaned up from pending store, and that
    /// is now only available from the chain. This is used by the `PendingSynchronizer` to
    /// know which operations it should not include anymore in its requests.
    ///
    pending_last_cleanup_block: Option<(BlockOffset, BlockHeight)>,
}

impl Default for SyncState {
    fn default() -> Self {
        SyncState {
            pending_last_cleanup_block: None,
        }
    }
}
