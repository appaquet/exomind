use std;
use std::ops::RangeBounds;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

use futures::prelude::*;
use futures::sync::mpsc;
use tokio;
use tokio::timer::Interval;

use crate::operation::OperationId;
use exocore_common;
use exocore_common::node::NodeId;
use exocore_common::serialization::protos::data_chain_capnp::pending_operation;
use exocore_common::serialization::protos::data_transport_capnp::{
    chain_sync_request, chain_sync_response, envelope, pending_sync_request,
};
use exocore_common::serialization::protos::MessageType;

use crate::block;
use crate::block::{Block, BlockOffset, BlockRef};
use crate::chain;
use crate::operation;
use crate::operation::{NewOperation, OperationBuilder};
use crate::pending;
use exocore_common::time::Clock;
use exocore_transport::{Error as TransportError, InMessage, OutMessage, TransportHandle};
use itertools::Itertools;

mod chain_sync;
mod commit_manager;
mod errors;
mod pending_sync;
mod request_tracker;

use crate::pending::CommitStatus;
pub use chain_sync::ChainSyncConfig;
pub use commit_manager::CommitManagerConfig;
pub use errors::Error;
use exocore_common::cell::{Cell, CellNodes};
use exocore_common::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
pub use pending_sync::PendingSyncConfig;

#[cfg(any(test, feature = "tests_utils"))]
pub(crate) mod testing;

///
/// Data engine's configuration
///
#[derive(Copy, Clone)]
pub struct Config {
    pub chain_synchronizer_config: ChainSyncConfig,
    pub pending_synchronizer_config: PendingSyncConfig,
    pub commit_manager_config: CommitManagerConfig,
    pub manager_timer_interval: Duration,
    pub handles_events_stream_size: usize,
    pub to_transport_channel_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_synchronizer_config: ChainSyncConfig::default(),
            pending_synchronizer_config: PendingSyncConfig::default(),
            commit_manager_config: CommitManagerConfig::default(),
            manager_timer_interval: Duration::from_secs(1),
            handles_events_stream_size: 1000,
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
    handles_count: usize,
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
            cell,
            clock: clock.clone(),
            pending_store,
            pending_synchronizer,
            chain_store,
            chain_synchronizer,
            commit_manager,
            handles_sender: Vec::new(),
            transport_sender: None,
            sync_state: SyncState::default(),
            stop_notifier,
        }));

        Engine {
            start_notifier,
            config,
            inner,
            handles_count: 0,
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

        let id = self.handles_count;
        self.handles_count += 1;

        let channel_size = self.config.handles_events_stream_size;
        let (events_sender, events_receiver) = mpsc::channel(channel_size);
        unlocked_inner
            .handles_sender
            .push((id, false, events_sender));

        EngineHandle {
            id,
            inner: Arc::downgrade(&self.inner),
            events_receiver: Some(events_receiver),
            start_listener,
            stop_listener,
        }
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
                    .for_each(move |msg| {
                        Self::handle_incoming_message(&weak_inner1, msg)
                            .map_err(|err| {
                                error!("Error handling incoming message: {}", err);
                                err
                            })
                            .or_else(Error::recover_non_fatal_error)
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
            unlocked_inner.notify_handles(&Event::Started);
        }

        info!("Engine started!");
        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        message: InMessage,
    ) -> Result<(), Error> {
        let locked_inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut inner = locked_inner.write()?;

        let envelope_reader: envelope::Reader = message.envelope.get_reader()?;
        debug!(
            "{}: Got message of type {} from node {}",
            inner.cell.local_node().id(),
            envelope_reader.get_type(),
            envelope_reader.get_from_node_id()?
        );

        match envelope_reader.get_type() {
            <pending_sync_request::Owned as MessageType>::MESSAGE_TYPE => {
                let data = envelope_reader.get_data()?;
                let sync_request = TypedCapnpFrame::new(data)?;
                inner.handle_incoming_pending_sync_request(&message, sync_request)?;
            }
            <chain_sync_request::Owned as MessageType>::MESSAGE_TYPE => {
                let data = envelope_reader.get_data()?;
                let sync_request = TypedCapnpFrame::new(data)?;
                inner.handle_incoming_chain_sync_request(&message, sync_request)?;
            }
            <chain_sync_response::Owned as MessageType>::MESSAGE_TYPE => {
                let data = envelope_reader.get_data()?;
                let sync_response = TypedCapnpFrame::new(data)?;
                inner.handle_incoming_chain_sync_response(&message, sync_response)?;
            }
            msg_type => {
                return Err(Error::Other(format!(
                    "Got an unknown message type: message_type={} transport_layer={}",
                    msg_type,
                    envelope_reader.get_layer()
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
///
struct Inner<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    cell: Cell,
    clock: Clock,
    pending_store: PS,
    pending_synchronizer: pending_sync::PendingSynchronizer<PS>,
    chain_store: CS,
    chain_synchronizer: chain_sync::ChainSynchronizer<CS>,
    commit_manager: commit_manager::CommitManager<PS, CS>,
    handles_sender: Vec<(usize, bool, mpsc::Sender<Event>)>,
    transport_sender: Option<mpsc::Sender<OutMessage>>,
    sync_state: SyncState,
    stop_notifier: CompletionNotifier<(), Error>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn handle_add_pending_operation(&mut self, operation: NewOperation) -> Result<(), Error> {
        let mut sync_context = SyncContext::new(self.sync_state);
        self.pending_synchronizer.handle_new_operation(
            &mut sync_context,
            &mut self.pending_store,
            operation,
        )?;
        self.sync_state = sync_context.sync_state;

        // to prevent sending pending operations that may have already been committed, we don't propagate
        // pending store changes unless the chain is synchronized
        if self.chain_is_synchronized() {
            self.send_messages_from_sync_context(&mut sync_context)?;
        }

        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn handle_incoming_pending_sync_request<R: FrameReader>(
        &mut self,
        message: &InMessage,
        request: TypedCapnpFrame<R, pending_sync_request::Owned>,
    ) -> Result<(), Error> {
        // to prevent sending pending operations that may have already been committed, we don't accept
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
        self.notify_handles_from_sync_context(&sync_context);

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
        self.notify_handles_from_sync_context(&sync_context);

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
        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn tick_synchronizers(&mut self) -> Result<(), Error> {
        let mut sync_context = SyncContext::new(self.sync_state);

        self.chain_synchronizer
            .tick(&mut sync_context, &self.chain_store)?;

        // to prevent synchronizing pending operations that may have added to the chain, we should only
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
        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn send_messages_from_sync_context(
        &mut self,
        sync_context: &mut SyncContext,
    ) -> Result<(), Error> {
        if !sync_context.messages.is_empty() {
            // swap out messages from the sync_context struct to consume them
            let mut messages = Vec::new();
            std::mem::swap(&mut sync_context.messages, &mut messages);

            for message in messages {
                let out_message = message.into_out_message(&self.cell)?;
                let transport_sender = self.transport_sender.as_mut().expect(
                    "Transport sender was none, which mean that the transport was never started",
                );
                if let Err(err) = transport_sender.try_send(out_message) {
                    error!(
                        "Error sending message from sync context to transport: {}",
                        err
                    );
                }
            }
        }

        Ok(())
    }

    fn notify_handles_from_sync_context(&mut self, sync_context: &SyncContext) {
        for event in sync_context.events.iter() {
            self.notify_handles(&event)
        }
    }

    fn notify_handles(&mut self, event: &Event) {
        for (id, discontinued, handle_sender) in self.handles_sender.iter_mut() {
            // if we hit a full buffer at last send, the stream got a discontinuity and we need to advise consumer.
            // we try to emit a discontinuity event, and if we succeed (buffer has space), we try to send the next event
            if *discontinued {
                if let Ok(()) = handle_sender.try_send(Event::StreamDiscontinuity) {
                    *discontinued = false;
                } else {
                    continue;
                }
            }

            match handle_sender.try_send(event.clone()) {
                Ok(()) => {}
                Err(ref err) if err.is_full() => {
                    error!("Couldn't send event to handle {} because channel buffer is full. Marking as discontinued", id);
                    *discontinued = true;
                }
                Err(err) => {
                    error!(
                        "Couldn't send event to handle {} for a reason other than channel buffer full: {:}",
                        id,
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

    fn unregister_handle(&mut self, id: usize) {
        let found_index = self
            .handles_sender
            .iter()
            .find_position(|(some_id, _discontinued, _sender)| *some_id == id);

        if let Some((index, _item)) = found_index {
            self.handles_sender.remove(index);
        }

        // if it was last handle, we kill the engine
        if self.handles_sender.is_empty() {
            debug!("Last engine handle got dropped, killing the engine.");
            self.stop_notifier.complete(Ok(()));
        }
    }
}

///
/// Handle ot the Engine, allowing communication with the engine.
/// The engine itself is owned by a future executor.
///
pub struct EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    id: usize,
    inner: Weak<RwLock<Inner<CS, PS>>>,
    events_receiver: Option<mpsc::Receiver<Event>>,
    start_listener: CompletionListener<(), Error>,
    stop_listener: CompletionListener<(), Error>,
}

impl<CS, PS> EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    pub fn on_start(&self) -> Result<impl Future<Item = (), Error = Error>, Error> {
        Ok(self
            .start_listener
            .try_clone()
            .map_err(|_err| Error::Other("Couldn't clone start listener in handle".to_string()))?
            .map_err(|err| match err {
                CompletionError::UserError(err) => err,
                _ => Error::Other("Error in completion error".to_string()),
            }))
    }

    pub fn on_stop(&self) -> Result<impl Future<Item = (), Error = Error>, Error> {
        Ok(self
            .stop_listener
            .try_clone()
            .map_err(|_err| Error::Other("Couldn't clone stop listener in handle".to_string()))?
            .map_err(|err| match err {
                CompletionError::UserError(err) => err,
                _ => Error::Other("Error in completion error".to_string()),
            }))
    }

    pub fn write_entry_operation(&self, data: &[u8]) -> Result<OperationId, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut unlocked_inner = inner.write()?;

        let my_node = unlocked_inner.cell.local_node();
        let operation_id = unlocked_inner.clock.consistent_time(my_node);

        let operation_builder = OperationBuilder::new_entry(operation_id, my_node.id(), data);
        let operation = operation_builder.sign_and_build(&my_node)?;

        unlocked_inner.handle_add_pending_operation(operation)?;

        Ok(operation_id)
    }

    pub fn get_chain_segments(&self) -> Result<Vec<chain::Segment>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;
        Ok(unlocked_inner.chain_store.segments())
    }

    pub fn get_chain_operation(
        &self,
        block_offset: BlockOffset,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let block = unlocked_inner.chain_store.get_block(block_offset)?;
        EngineOperation::from_chain_block(block, operation_id)
    }

    pub fn get_pending_operation(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let operation = unlocked_inner
            .pending_store
            .get_operation(operation_id)?
            .map(EngineOperation::from_pending_operation);

        Ok(operation)
    }

    pub fn get_pending_operations<R: RangeBounds<OperationId>>(
        &self,
        operations_range: R,
    ) -> Result<Vec<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let operations = unlocked_inner
            .pending_store
            .operations_iter(operations_range)?
            .map(EngineOperation::from_pending_operation)
            .collect::<Vec<_>>();
        Ok(operations)
    }

    pub fn get_operation(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        // first check if it's in pending store with a clear commit status
        let pending_operation = unlocked_inner.pending_store.get_operation(operation_id)?;
        let pending_operation = if let Some(pending_operation) = pending_operation {
            if pending_operation.commit_status != CommitStatus::Unknown {
                return Ok(Some(EngineOperation::from_pending_operation(
                    pending_operation,
                )));
            }

            Some(pending_operation)
        } else {
            None
        };

        // if it's not found in pending store, or that it didn't have a clear status, we check in chain
        if let Some(block) = unlocked_inner
            .chain_store
            .get_block_by_operation_id(operation_id)?
        {
            if let Some(chain_operation) = EngineOperation::from_chain_block(block, operation_id)? {
                return Ok(Some(chain_operation));
            }
        }

        // if we're here, the operation was either absent, or just had a unknown status in pending store
        // we return the pending store operation (if any)
        Ok(pending_operation.map(EngineOperation::from_pending_operation))
    }

    ///
    /// Take the events stream receiver out of this `Handle`.
    /// This stream is bounded and consumptions should be non-blocking to prevent losing events.
    /// Calling the engine on every call should be throttled in the case of a big read amplification.
    pub fn take_events_stream(&mut self) -> impl futures::Stream<Item = Event, Error = Error> {
        self.events_receiver
            .take()
            .expect("Get events stream was already called.")
            .map_err(|_err| Error::Other("Error in incoming events stream".to_string()))
    }
}

impl<CS, PS> Drop for EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn drop(&mut self) {
        debug!("Engine handle got dropped.");

        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut unlocked_inner) = inner.write() {
                unlocked_inner.unregister_handle(self.id);
            }
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
                OutMessage::from_framed_message(cell, to_nodes, request_builder)?
            }
            SyncContextMessage::ChainSyncRequest(_, request_builder) => {
                OutMessage::from_framed_message(cell, to_nodes, request_builder)?
            }
            SyncContextMessage::ChainSyncResponse(_, response_builder) => {
                OutMessage::from_framed_message(cell, to_nodes, response_builder)?
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
    pending_last_cleanup_block: Option<(block::BlockOffset, block::BlockDepth)>,
}

impl Default for SyncState {
    fn default() -> Self {
        SyncState {
            pending_last_cleanup_block: None,
        }
    }
}

///
/// Events dispatched to handles to notify changes in the different stores.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The engine is now started
    Started,

    /// The stream of events hit the maximum buffer size, and some events got discarded.
    /// Consumer state should be rebuilt from scratch to prevent having inconsistencies.
    StreamDiscontinuity,

    /// An operation added to the pending store.
    PendingOperationNew(OperationId),

    /// An operation that was previously added got deleted, hence will never end up in a block.
    /// This happens if an operation was invalid or found in the chain later on.
    PendingEntryDelete(OperationId),

    /// A new block got added to the chain.
    ChainBlockNew(BlockOffset),

    /// The chain has diverged from given offset, which mean it will get re-written with new blocks.
    /// Operations after this offset should ignored.
    ChainDiverged(BlockOffset),
}

///
/// Operation that comes either from the chain or from the pending store
///
pub struct EngineOperation {
    pub operation_id: OperationId,
    pub status: EngineOperationStatus,
    pub operation_frame: Arc<super::operation::OperationFrame<Vec<u8>>>,
}

impl EngineOperation {
    fn from_pending_operation(operation: pending::StoredOperation) -> EngineOperation {
        let status = match operation.commit_status {
            pending::CommitStatus::Committed(offset, _depth) => {
                EngineOperationStatus::Committed(offset)
            }
            _ => EngineOperationStatus::Pending,
        };

        EngineOperation {
            operation_id: operation.operation_id,
            status,
            operation_frame: operation.frame,
        }
    }

    fn from_chain_block(
        block: BlockRef,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        if let Some(operation) = block.get_operation(operation_id)? {
            return Ok(Some(EngineOperation {
                operation_id,
                status: EngineOperationStatus::Committed(block.offset),
                operation_frame: Arc::new(operation.to_owned()),
            }));
        }

        Ok(None)
    }
}

impl crate::operation::Operation for EngineOperation {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, operation::Error> {
        Ok(self.operation_frame.get_reader()?)
    }
}

#[derive(Debug, PartialEq)]
pub enum EngineOperationStatus {
    Committed(BlockOffset),
    Pending,
}

impl EngineOperationStatus {
    pub fn is_committed(&self) -> bool {
        match self {
            EngineOperationStatus::Committed(_offset) => true,
            _ => false,
        }
    }
}
