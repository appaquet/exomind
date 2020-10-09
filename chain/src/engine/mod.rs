use crate::chain;
use crate::operation;
use crate::operation::NewOperation;
use crate::pending;
use exocore_core::cell::Cell;
use exocore_core::framing::{FrameReader, TypedCapnpFrame};
use exocore_core::futures::{interval, spawn_blocking};
use exocore_core::protos::generated::data_transport_capnp::{
    chain_sync_request, chain_sync_response, pending_sync_request,
};
use exocore_core::protos::generated::MessageType;
use exocore_core::time::Clock;
use exocore_core::utils::handle_set::HandleSet;
use exocore_transport::{InEvent, InMessage, OutEvent, TransportServiceHandle};
use futures::channel::mpsc;
use futures::future::FutureExt;
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, RwLock, Weak};

pub use chain_sync::ChainSyncConfig;
pub use commit_manager::CommitManagerConfig;
pub use config::EngineConfig;
pub use error::EngineError;
pub use handle::{EngineHandle, EngineOperation, EngineOperationStatus, Event};
pub use pending_sync::PendingSyncConfig;
pub use request_tracker::RequestTrackerConfig;
pub use sync_context::{SyncContext, SyncContextMessage, SyncState};

mod chain_sync;
mod commit_manager;
mod config;
mod error;
mod handle;
mod pending_sync;
mod request_tracker;
mod sync_context;
#[cfg(test)]
pub(crate) mod testing;

/// The chain engine manages storage and replication of data among the nodes of
/// the cell.
///
/// It contains 2 stores:
///   * Pending store: temporary store in which operations are stored until they
///     get commited to chain
///   * Chain store: persistent store using a block-chain like data structure
pub struct Engine<T, CS, PS>
where
    T: TransportServiceHandle,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    config: EngineConfig,
    transport: T,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    handle_set: HandleSet,
}

impl<T, CS, PS> Engine<T, CS, PS>
where
    T: TransportServiceHandle,
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    pub fn new(
        config: EngineConfig,
        clock: Clock,
        transport: T,
        chain_store: CS,
        pending_store: PS,
        cell: Cell,
    ) -> Engine<T, CS, PS> {
        let pending_synchronizer = pending_sync::PendingSynchronizer::new(
            config.pending_sync_config,
            cell.clone(),
            clock.clone(),
        );
        let chain_synchronizer = chain_sync::ChainSynchronizer::new(
            config.chain_sync_config.clone(),
            cell.clone(),
            clock.clone(),
        );
        let commit_manager = commit_manager::CommitManager::new(
            config.commit_manager_config,
            cell.clone(),
            clock.clone(),
        );

        let inner = Arc::new(RwLock::new(Inner {
            config: config.clone(),
            cell,
            clock,
            pending_store,
            pending_synchronizer,
            chain_store,
            chain_synchronizer,
            commit_manager,
            events_stream_sender: Vec::new(),
            transport_sender: None,
            sync_state: SyncState::default(),
        }));

        Engine {
            config,
            inner,
            transport,
            handle_set: HandleSet::new(),
        }
    }

    pub fn get_handle(&mut self) -> EngineHandle<CS, PS> {
        EngineHandle::new(Arc::downgrade(&self.inner), self.handle_set.get_handle())
    }

    pub async fn run(mut self) -> Result<(), EngineError> {
        let config = self.config;

        let (transport_out_sender, mut transport_out_receiver) =
            mpsc::channel(config.to_transport_channel_size);
        let mut transport_out_sink = self.transport.get_sink();
        let outgoing_transport_handler = async move {
            while let Some(event) = transport_out_receiver.next().await {
                if let Err(err) = transport_out_sink.send(event).await {
                    error!("Error sending to transport sink: {}", err);
                }
            }
        };

        let mut transport_in_stream = self.transport.get_stream();
        let weak_inner = Arc::downgrade(&self.inner);
        let incoming_transport_handler = async move {
            while let Some(event) = transport_in_stream.next().await {
                let result = Self::handle_incoming_event(weak_inner.clone(), event).await;
                if let Err(err) = result {
                    error!("Error handling incoming message: {}", err);
                    if err.is_fatal() {
                        return;
                    }
                }
            }
        };

        let weak_inner = Arc::downgrade(&self.inner);
        let management_timer = async move {
            let mut interval = interval(config.manager_timer_interval);
            while interval.next().await.is_some() {
                let result = Self::handle_management_timer_tick(weak_inner.clone()).await;
                if let Err(err) = result {
                    error!("Error in management timer: {}", err);
                    if err.is_fatal() {
                        return;
                    }
                }
            }
        };

        {
            let mut unlocked_inner = self.inner.write()?;
            unlocked_inner.transport_sender = Some(transport_out_sender);

            let chain_last_block = unlocked_inner.chain_store.get_last_block()?;
            if chain_last_block.is_none() {
                warn!("{}: Chain has not been initialized (no genesis block). May not be able to start if no other nodes are found.",
                      unlocked_inner.cell,
                )
            }

            unlocked_inner.dispatch_event(&Event::Started);
        }

        info!("Engine started");
        futures::select! {
            _ = outgoing_transport_handler.fuse() => (),
            _ = incoming_transport_handler.fuse() => (),
            _ = management_timer.fuse() => (),
            _ = self.handle_set.on_handles_dropped().fuse() => (),
            _ = self.transport.fuse() => (),
        }
        info!("Engine done");

        Ok(())
    }

    async fn handle_incoming_event(
        weak_inner: Weak<RwLock<Inner<CS, PS>>>,
        event: InEvent,
    ) -> Result<(), EngineError> {
        match event {
            InEvent::Message(msg) => Self::handle_incoming_message(weak_inner, msg).await,
            InEvent::NodeStatus(_, _) => {
                // unhandled for now, but could be used by synchronizers
                Ok(())
            }
        }
    }

    async fn handle_incoming_message(
        weak_inner: Weak<RwLock<Inner<CS, PS>>>,
        message: Box<InMessage>,
    ) -> Result<(), EngineError> {
        let join_result = spawn_blocking(move || {
            let locked_inner = weak_inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
            let mut inner = locked_inner.write()?;

            debug!(
                "{}: Got message of type {} from node {}",
                inner.cell,
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
                    return Err(EngineError::Other(format!(
                        "Got an unknown message type: message_type={} service_type={:?}",
                        msg_type, message.service_type,
                    )));
                }
            }

            Ok(())
        })
        .await;

        match join_result {
            Ok(res) => res,
            Err(err) => Err(EngineError::Fatal(format!(
                "Error joining blocking spawn: {}",
                err
            ))),
        }
    }

    async fn handle_management_timer_tick(
        weak_inner: Weak<RwLock<Inner<CS, PS>>>,
    ) -> Result<(), EngineError> {
        let join_result = spawn_blocking(move || {
            let locked_inner = weak_inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
            let mut inner = locked_inner.write()?;
            inner.tick_synchronizers()?;

            Ok(())
        })
        .await;

        match join_result {
            Ok(res) => res,
            Err(err) => Err(EngineError::Fatal(format!(
                "Error joining blocking spawn: {}",
                err
            ))),
        }
    }
}

pub(crate) struct Inner<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    config: EngineConfig,
    cell: Cell,
    clock: Clock,
    pending_store: PS,
    pending_synchronizer: pending_sync::PendingSynchronizer<PS>,
    chain_store: CS,
    chain_synchronizer: chain_sync::ChainSynchronizer<CS>,
    commit_manager: commit_manager::CommitManager<PS, CS>,
    events_stream_sender: Vec<(usize, bool, mpsc::Sender<Event>)>,
    transport_sender: Option<mpsc::Sender<OutEvent>>,
    sync_state: SyncState,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn handle_new_operation(&mut self, operation: NewOperation) -> Result<(), EngineError> {
        let mut sync_context = SyncContext::new(self.sync_state);
        self.pending_synchronizer.handle_new_operation(
            &mut sync_context,
            &mut self.pending_store,
            operation,
        )?;
        self.sync_state = sync_context.sync_state;

        // to prevent sending operations that may have already been committed, we don't
        // propagate pending store changes unless the chain is synchronized
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
    ) -> Result<(), EngineError> {
        // to prevent sending operations that may have already been committed, we don't
        // accept any pending sync requests until the chain is synchronized
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
    ) -> Result<(), EngineError> {
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
    ) -> Result<(), EngineError> {
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

    fn tick_synchronizers(&mut self) -> Result<(), EngineError> {
        let mut sync_context = SyncContext::new(self.sync_state);

        self.chain_synchronizer
            .tick(&mut sync_context, &self.chain_store)?;

        // to prevent synchronizing operations that may have added to the chain, we
        // should only start doing commit management & pending synchronization
        // once the chain is synchronized
        if self.chain_is_synchronized() {
            // commit manager should always be ticked before pending synchronizer so that it
            // may remove operations that don't need to be synchronized anymore
            // (ex: been committed)
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
    ) -> Result<(), EngineError> {
        if sync_context.messages.is_empty() {
            return Ok(());
        }

        // swap out messages from the sync_context struct to consume them
        let mut messages = Vec::new();
        std::mem::swap(&mut sync_context.messages, &mut messages);

        for message in messages {
            let out_message = message.into_out_message(&self.cell)?;
            let transport_sender = self.transport_sender.as_mut().unwrap();
            if let Err(err) = transport_sender.try_send(OutEvent::Message(out_message)) {
                error!(
                    "Error sending message from sync context to transport: {}",
                    err
                );
            }
        }

        Ok(())
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
            // if we hit a full buffer at last send, the stream got a discontinuity and we
            // need to advise consumer. we try to emit a discontinuity event,
            // and if we succeed (buffer has space), we try to send the next event
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
                    error!(
                        "Couldn't send event to handle {} because channel buffer is full. Marking as discontinued",
                        handle_id
                    );
                    *discontinued = true;
                }
                Err(err) => {
                    error!(
                        "Couldn't send event to handle {} for a reason other than channel buffer full: {:}",
                        handle_id, err
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
    }
}
