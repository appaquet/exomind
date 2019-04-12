#![allow(dead_code)]

use std;
use std::ops::RangeBounds;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

use futures::prelude::*;
use futures::sync::mpsc;
use futures::sync::oneshot;
use tokio;
use tokio::timer::Interval;

use exocore_common;
use exocore_common::node::{Node, NodeID, Nodes};
use exocore_common::serialization::capnp;
use exocore_common::serialization::framed::{
    FrameBuilder, MessageType, OwnedTypedFrame, TypedSliceFrame,
};
use exocore_common::serialization::protos::data_chain_capnp::pending_operation;
use exocore_common::serialization::protos::data_transport_capnp::{
    chain_sync_request, chain_sync_response, envelope, pending_sync_request,
};
use exocore_common::serialization::protos::OperationID;
use exocore_common::serialization::{framed, framed::TypedFrame};

use crate::chain;
use crate::chain::{Block, BlockOffset};
use crate::pending;
use crate::pending::PendingOperation;
use crate::transport;
use crate::transport::OutMessage;
use exocore_common::time::Clock;
use itertools::Itertools;

mod chain_sync;
mod commit_manager;
mod pending_sync;
mod request_tracker;

#[cfg(any(test, feature = "tests_utils"))]
pub(crate) mod testing;

///
/// Data engine's configuration
///
#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub chain_synchronizer_config: chain_sync::ChainSyncConfig,
    pub pending_synchronizer_config: pending_sync::PendingSyncConfig,
    pub commit_manager_config: commit_manager::CommitManagerConfig,
    pub manager_timer_interval: Duration,
    pub handles_events_stream_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_synchronizer_config: chain_sync::ChainSyncConfig::default(),
            pending_synchronizer_config: pending_sync::PendingSyncConfig::default(),
            commit_manager_config: commit_manager::CommitManagerConfig::default(),
            manager_timer_interval: Duration::from_secs(1),
            handles_events_stream_size: 1000,
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
    T: transport::Transport,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    config: Config,
    started: bool,
    clock: Clock,
    transport: Option<T>,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    handles_count: usize,
    completion_receiver: oneshot::Receiver<Result<(), Error>>,
}

impl<T, CS, PS> Engine<T, CS, PS>
where
    T: transport::Transport,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    pub fn new(
        config: Config,
        node_id: NodeID,
        clock: Clock,
        transport: T,
        chain_store: CS,
        pending_store: PS,
        nodes: Nodes,
    ) -> Engine<T, CS, PS> {
        let (completion_sender, completion_receiver) = oneshot::channel();

        let pending_synchronizer = pending_sync::PendingSynchronizer::new(
            node_id.clone(),
            config.pending_synchronizer_config,
        );
        let chain_synchronizer =
            chain_sync::ChainSynchronizer::new(node_id.clone(), config.chain_synchronizer_config);
        let commit_manager = commit_manager::CommitManager::new(
            node_id.clone(),
            config.commit_manager_config,
            clock.clone(),
        );

        let inner = Arc::new(RwLock::new(Inner {
            config,
            node_id,
            nodes,
            clock: clock.clone(),
            pending_store,
            pending_synchronizer,
            chain_store,
            chain_synchronizer,
            commit_manager,
            handles_sender: Vec::new(),
            transport_sender: None,
            completion_sender: Some(completion_sender),
        }));

        Engine {
            config,
            started: false,
            clock,
            inner,
            handles_count: 0,
            transport: Some(transport),
            completion_receiver,
        }
    }

    pub fn get_handle(&mut self) -> Handle<CS, PS> {
        let mut unlocked_inner = self
            .inner
            .write()
            .expect("Inner couldn't get locked, but engine isn't even started yet.");

        let id = self.handles_count;
        self.handles_count += 1;

        let channel_size = self.config.handles_events_stream_size;
        let (events_sender, events_receiver) = mpsc::channel(channel_size);
        unlocked_inner
            .handles_sender
            .push((id, false, events_sender));

        Handle {
            id,
            inner: Arc::downgrade(&self.inner),
            events_receiver: Some(events_receiver),
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
            let (transport_out_channel_sender, transport_out_channel_receiver) = mpsc::unbounded();
            tokio::spawn(
                transport_out_channel_receiver
                    .map_err(|err| {
                        transport::Error::Other(format!(
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
                    .map_err(|err| Error::Other(format!("Interval error: {:?}", err)))
                    .for_each(move |_| {
                        Self::handle_management_timer_tick(&weak_inner1)
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

        self.started = true;
        info!("Engine started!");
        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        message: transport::InMessage,
    ) -> Result<(), Error> {
        let locked_inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut inner = locked_inner.write()?;

        let envelope_reader: envelope::Reader = message.envelope.get_typed_reader()?;
        debug!(
            "{}: Got message of type {} from node {}",
            inner.node_id,
            envelope_reader.get_type(),
            envelope_reader.get_from_node()?
        );

        match envelope_reader.get_type() {
            <pending_sync_request::Owned as MessageType>::MESSAGE_TYPE => {
                let data = envelope_reader.get_data()?;
                let sync_request = TypedSliceFrame::<pending_sync_request::Owned>::new(data)?;
                inner.handle_incoming_pending_sync_request(&message, sync_request)?;
            }
            <chain_sync_request::Owned as MessageType>::MESSAGE_TYPE => {
                let data = envelope_reader.get_data()?;
                let sync_request = TypedSliceFrame::<chain_sync_request::Owned>::new(data)?;
                inner.handle_incoming_chain_sync_request(&message, sync_request)?;
            }
            <chain_sync_response::Owned as MessageType>::MESSAGE_TYPE => {
                let data = envelope_reader.get_data()?;
                let sync_response = TypedSliceFrame::<chain_sync_response::Owned>::new(data)?;
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
        error!("Got an error in future {}: {:?}", future_name, error);

        let locked_inner = if let Some(locked_inner) = weak_inner.upgrade() {
            locked_inner
        } else {
            return;
        };

        let mut inner = if let Ok(inner) = locked_inner.write() {
            inner
        } else {
            return;
        };

        inner.try_complete_engine(Err(Error::Other(format!(
            "Couldn't send to completion channel: {:?}",
            error
        ))));
    }
}

impl<T, CS, PS> Future for Engine<T, CS, PS>
where
    T: transport::Transport,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<()>, Error> {
        if !self.started {
            self.start()?;
        }

        // check if engine was stopped or failed
        let _ = try_ready!(self
            .completion_receiver
            .poll()
            .map_err(|_cancel| Error::Other("Completion receiver has been cancelled".to_string())));

        Ok(Async::Ready(()))
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
    config: Config,
    node_id: NodeID,
    nodes: Nodes,
    clock: Clock,
    pending_store: PS,
    pending_synchronizer: pending_sync::PendingSynchronizer<PS>,
    chain_store: CS,
    chain_synchronizer: chain_sync::ChainSynchronizer<CS>,
    commit_manager: commit_manager::CommitManager<PS, CS>,
    handles_sender: Vec<(usize, bool, mpsc::Sender<Event>)>,
    transport_sender: Option<mpsc::UnboundedSender<transport::OutMessage>>,
    completion_sender: Option<oneshot::Sender<Result<(), Error>>>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn handle_add_pending_operation(
        &mut self,
        operation_frame: OwnedTypedFrame<pending_operation::Owned>,
    ) -> Result<(), Error> {
        let mut sync_context = SyncContext::new();
        // TODO: Make sure chain_synchronizer is Synchronized first: https://github.com/appaquet/exocore/issues/44
        self.pending_synchronizer.handle_new_operation(
            &mut sync_context,
            &self.nodes,
            &mut self.pending_store,
            operation_frame,
        )?;

        self.send_messages_from_sync_context(&mut sync_context)?;
        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn handle_incoming_pending_sync_request<R>(
        &mut self,
        message: &transport::InMessage,
        request: R,
    ) -> Result<(), Error>
    where
        R: TypedFrame<pending_sync_request::Owned>,
    {
        let mut sync_context = SyncContext::new();
        self.pending_synchronizer.handle_incoming_sync_request(
            &message.from,
            &mut sync_context,
            &mut self.pending_store,
            request,
        )?;
        self.send_messages_from_sync_context(&mut sync_context)?;
        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn handle_incoming_chain_sync_request<R>(
        &mut self,
        message: &transport::InMessage,
        request: R,
    ) -> Result<(), Error>
    where
        R: TypedFrame<chain_sync_request::Owned>,
    {
        let mut sync_context = SyncContext::new();
        self.chain_synchronizer.handle_sync_request(
            &mut sync_context,
            &message.from,
            &mut self.chain_store,
            request,
        )?;
        self.send_messages_from_sync_context(&mut sync_context)?;
        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn handle_incoming_chain_sync_response<R>(
        &mut self,
        message: &transport::InMessage,
        response: R,
    ) -> Result<(), Error>
    where
        R: TypedFrame<chain_sync_response::Owned>,
    {
        let mut sync_context = SyncContext::new();
        self.chain_synchronizer.handle_sync_response(
            &mut sync_context,
            &message.from,
            &mut self.chain_store,
            response,
        )?;
        self.send_messages_from_sync_context(&mut sync_context)?;
        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn tick_synchronizers(&mut self) -> Result<(), Error> {
        let mut sync_context = SyncContext::new();

        // TODO: We should only do commit manager & pending synchronizer once chain is synced: https://github.com/appaquet/exocore/issues/44

        self.chain_synchronizer
            .tick(&mut sync_context, &self.chain_store, &self.nodes)?;

        self.commit_manager.tick(
            &mut sync_context,
            &mut self.pending_synchronizer,
            &mut self.pending_store,
            &mut self.chain_synchronizer,
            &mut self.chain_store,
            &self.nodes,
        )?;

        self.pending_synchronizer
            .tick(&mut sync_context, &self.pending_store, &self.nodes)?;

        self.send_messages_from_sync_context(&mut sync_context)?;
        self.notify_handles_from_sync_context(&sync_context);

        Ok(())
    }

    fn send_messages_from_sync_context(
        &mut self,
        sync_context: &mut SyncContext,
    ) -> Result<(), Error> {
        if !sync_context.messages.is_empty() {
            let local_node = self.nodes.get(&self.node_id).ok_or_else(|| {
                Error::Other(format!(
                    "Couldn't find local node {} in nodes list",
                    self.node_id
                ))
            })?;

            // swap out messages from the sync_context struct to consume them
            let mut messages = Vec::new();
            std::mem::swap(&mut sync_context.messages, &mut messages);

            for message in messages {
                let out_message = message.into_out_message(local_node, &self.nodes)?;
                let transport_sender = self.transport_sender.as_ref().expect(
                    "Transport sender was none, which mean that the transport was never started",
                );
                transport_sender.unbounded_send(out_message)?;
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

    fn unregister_handle(&mut self, id: usize) {
        let found_index = self
            .handles_sender
            .iter()
            .find_position(|(some_id, _discontinued, _sender)| *some_id == id);

        if let Some((index, _item)) = found_index {
            self.handles_sender.remove(index);
        }
    }

    fn try_complete_engine(&mut self, result: Result<(), Error>) {
        if let Some(sender) = self.completion_sender.take() {
            let _ = sender.send(result);
        }
    }
}

///
/// Handle ot the Engine, allowing communication with the engine.
/// The engine itself is owned by a future executor.
///
pub struct Handle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    id: usize,
    inner: Weak<RwLock<Inner<CS, PS>>>,
    events_receiver: Option<mpsc::Receiver<Event>>,
}

impl<CS, PS> Handle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    pub fn get_chain_segments(&self) -> Result<Vec<chain::Segment>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;
        Ok(unlocked_inner.chain_store.segments())
    }

    pub fn get_chain_entry(
        &self,
        block_offset: chain::BlockOffset,
        operation_id: OperationID,
    ) -> Result<ChainOperation, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let block = unlocked_inner.chain_store.get_block(block_offset)?;
        let operation = block.get_operation(operation_id)?.ok_or_else(|| {
            Error::NotFound(format!(
                "block_offset={} operation_id={}",
                block_offset, operation_id
            ))
        })?;

        Ok(ChainOperation {
            operation_id,
            status: OperationStatus::Committed,
            operation_frame: operation.to_owned(),
        })
    }

    pub fn write_entry(&self, data: &[u8]) -> Result<OperationID, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut unlocked_inner = inner.write()?;

        let my_node = unlocked_inner
            .nodes
            .get(&unlocked_inner.node_id)
            .ok_or(Error::MyNodeNotFound)?;
        let operation_id = unlocked_inner.clock.consistent_time(my_node);
        let entry_operation =
            PendingOperation::new_entry(operation_id, &unlocked_inner.node_id, data);
        let entry_frame = entry_operation.as_owned_framed(my_node.frame_signer())?;

        unlocked_inner.handle_add_pending_operation(entry_frame)?;

        Ok(operation_id)
    }

    pub fn get_pending_operations<R: RangeBounds<OperationID>>(
        &self,
        operations_range: R,
    ) -> Result<Vec<pending::StoredOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.write()?;

        let operations = unlocked_inner
            .pending_store
            .operations_iter(operations_range)?
            .filter(|pending_operation| {
                // exclude operations that are in the chain
                unlocked_inner
                    .chain_store
                    .get_block_by_operation_id(pending_operation.operation_id)
                    .ok()
                    .and_then(|block| block)
                    .is_none()
            })
            .collect::<Vec<_>>();

        Ok(operations)
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

impl<CS, PS> Drop for Handle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut unlocked_inner) = inner.write() {
                unlocked_inner.unregister_handle(self.id);
            }
        }
    }
}

///
/// Engine errors
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error in transport: {:?}", _0)]
    Transport(#[fail(cause)] transport::Error),
    #[fail(display = "Error in pending store: {:?}", _0)]
    PendingStore(#[fail(cause)] pending::Error),
    #[fail(display = "Error in chain store: {:?}", _0)]
    ChainStore(#[fail(cause)] chain::Error),
    #[fail(display = "Error in pending synchronizer: {:?}", _0)]
    PendingSync(#[fail(cause)] pending_sync::PendingSyncError),
    #[fail(display = "Error in chain synchronizer: {:?}", _0)]
    ChainSync(#[fail(cause)] chain_sync::ChainSyncError),
    #[fail(display = "Error in commit manager: {:?}", _0)]
    CommitManager(commit_manager::CommitManagerError),
    #[fail(display = "Error in framing serialization: {:?}", _0)]
    Framing(#[fail(cause)] framed::Error),
    #[fail(display = "Chain is not initialized")]
    UninitializedChain,
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),
    #[fail(display = "Item not found: {}", _0)]
    NotFound(String),
    #[fail(display = "Local node not found in nodes list")]
    MyNodeNotFound,
    #[fail(display = "Couldn't send message to a mpsc channel because its receiver was dropped")]
    MpscSendDropped,
    #[fail(display = "Inner was dropped or couldn't get locked")]
    InnerUpgrade,
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "A fatal error occurred: {}", _0)]
    Fatal(String),
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Error::PendingStore(inner) => inner.is_fatal(),
            Error::ChainStore(inner) => inner.is_fatal(),
            Error::ChainSync(inner) => inner.is_fatal(),
            Error::MyNodeNotFound
            | Error::MpscSendDropped
            | Error::InnerUpgrade
            | Error::Poisoned
            | Error::Fatal(_) => true,
            _ => false,
        }
    }

    fn recover_non_fatal_error(self) -> Result<(), Error> {
        if !self.is_fatal() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl From<transport::Error> for Error {
    fn from(err: transport::Error) -> Self {
        Error::Transport(err)
    }
}

impl From<pending::Error> for Error {
    fn from(err: pending::Error) -> Self {
        Error::PendingStore(err)
    }
}

impl From<chain::Error> for Error {
    fn from(err: chain::Error) -> Self {
        Error::ChainStore(err)
    }
}

impl From<pending_sync::PendingSyncError> for Error {
    fn from(err: pending_sync::PendingSyncError) -> Self {
        Error::PendingSync(err)
    }
}

impl From<chain_sync::ChainSyncError> for Error {
    fn from(err: chain_sync::ChainSyncError) -> Self {
        Error::ChainSync(err)
    }
}

impl From<commit_manager::CommitManagerError> for Error {
    fn from(err: commit_manager::CommitManagerError) -> Self {
        Error::CommitManager(err)
    }
}

impl From<framed::Error> for Error {
    fn from(err: framed::Error) -> Self {
        Error::Framing(err)
    }
}

impl<T> From<mpsc::SendError<T>> for Error {
    fn from(_err: mpsc::SendError<T>) -> Self {
        Error::MpscSendDropped
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}

///
/// Synchronization context used by `chain_sync`, `pending_sync` and `commit_manager` to dispatch
/// messages to other nodes, and dispatch events to be sent to engine handles.
///
struct SyncContext {
    events: Vec<Event>,
    messages: Vec<SyncContextMessage>,
}

impl SyncContext {
    fn new() -> SyncContext {
        SyncContext {
            events: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn push_pending_sync_request(
        &mut self,
        node_id: NodeID,
        request_builder: FrameBuilder<pending_sync_request::Owned>,
    ) {
        self.messages.push(SyncContextMessage::PendingSyncRequest(
            node_id,
            request_builder,
        ));
    }

    fn push_chain_sync_request(
        &mut self,
        node_id: NodeID,
        request_builder: FrameBuilder<chain_sync_request::Owned>,
    ) {
        self.messages.push(SyncContextMessage::ChainSyncRequest(
            node_id,
            request_builder,
        ));
    }

    fn push_chain_sync_response(
        &mut self,
        node_id: NodeID,
        response_builder: FrameBuilder<chain_sync_response::Owned>,
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
    PendingSyncRequest(NodeID, FrameBuilder<pending_sync_request::Owned>),
    ChainSyncRequest(NodeID, FrameBuilder<chain_sync_request::Owned>),
    ChainSyncResponse(NodeID, FrameBuilder<chain_sync_response::Owned>),
}

impl SyncContextMessage {
    fn into_out_message<'n>(
        self,
        local_node: &'n Node,
        nodes: &'n Nodes,
    ) -> Result<OutMessage, Error> {
        let signer = local_node.frame_signer();

        let to_node = self.to_node();
        let to_nodes = nodes
            .nodes()
            .filter(|n| n.id() == to_node)
            .cloned()
            .collect();

        let message = match self {
            SyncContextMessage::PendingSyncRequest(_, request_builder) => {
                let frame = request_builder.as_owned_framed(signer)?;
                OutMessage::from_framed_message(local_node, to_nodes, frame)?
            }
            SyncContextMessage::ChainSyncRequest(_, request_builder) => {
                let frame = request_builder.as_owned_framed(signer)?;
                OutMessage::from_framed_message(local_node, to_nodes, frame)?
            }
            SyncContextMessage::ChainSyncResponse(_, response_builder) => {
                let frame = response_builder.as_owned_framed(signer)?;
                OutMessage::from_framed_message(local_node, to_nodes, frame)?
            }
        };

        Ok(message)
    }

    fn to_node(&self) -> &str {
        match self {
            SyncContextMessage::PendingSyncRequest(to_node, _) => &to_node,
            SyncContextMessage::ChainSyncRequest(to_node, _) => &to_node,
            SyncContextMessage::ChainSyncResponse(to_node, _) => &to_node,
        }
    }
}

///
/// Events dispatched to handles to notify changes in the different stores.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    ///
    /// The engine is now started
    ///
    Started,

    ///
    /// The stream of events hit the maximum buffer size, and some events got discarded.
    /// Consumer state should be rebuilt from scratch to prevent having inconsistencies.
    ///
    StreamDiscontinuity,

    ///
    /// An operation added to the pending store.
    ///
    PendingOperationNew(OperationID),

    ///
    /// An operation that was previously added got deleted, hence will never end up in a block.
    /// This happens if an operation was invalid or found in the chain later on.
    ///
    PendingEntryDelete(OperationID),

    ///
    /// A new block got added to the chain.
    ///
    ChainBlockNew(BlockOffset),

    ///
    /// The chain has diverged from given offset, which mean it will get re-written with new blocks.
    /// Operations after this offset should ignored.
    ///
    ChainDiverged(BlockOffset),
}

///
/// TODO: Should implement "Operation" trait, and should be named EngineOperation
///       https://github.com/appaquet/exocore/issues/50
pub struct ChainOperation {
    pub operation_id: OperationID,
    pub status: OperationStatus,
    pub operation_frame: OwnedTypedFrame<pending_operation::Owned>,
}

pub enum OperationStatus {
    Committed,
    Pending,
}
