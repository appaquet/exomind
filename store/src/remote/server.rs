use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

use futures::channel::{mpsc, oneshot};
use futures::{FutureExt, SinkExt, StreamExt};

use exocore_core::cell::Cell;
use exocore_core::futures::{interval, OwnedSpawnSet};
use exocore_core::protos::generated::exocore_store::{EntityQuery, EntityResults, MutationRequest};
use exocore_core::protos::generated::store_transport_capnp::{
    mutation_request, query_request, unwatch_query_request, watched_query_request,
};
use exocore_core::protos::{generated::MessageType, store::MutationResult};
use exocore_core::time::{Duration, Instant};
use exocore_transport::{InEvent, InMessage, OutEvent, OutMessage, TransportServiceHandle};

use crate::error::Error;
use crate::query::WatchToken;

pub struct Server<CS, PS, T>
where
    CS: exocore_chain::chain::ChainStore,
    PS: exocore_chain::pending::PendingStore,
    T: TransportServiceHandle,
{
    config: ServerConfiguration,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    transport_handle: T,
    transport_out_receiver: mpsc::UnboundedReceiver<OutEvent>,
}

impl<CS, PS, T> Server<CS, PS, T>
where
    CS: exocore_chain::chain::ChainStore,
    PS: exocore_chain::pending::PendingStore,
    T: TransportServiceHandle,
{
    pub fn new(
        config: ServerConfiguration,
        cell: Cell,
        store_handle: crate::local::StoreHandle<CS, PS>,
        transport_handle: T,
    ) -> Result<Server<CS, PS, T>, Error> {
        let (transport_out_sender, transport_out_receiver) = mpsc::unbounded();

        let inner = Arc::new(RwLock::new(Inner {
            config,
            cell,
            store_handle,
            watched_queries: HashMap::new(),
            transport_out_sender,
        }));

        Ok(Server {
            config,
            inner,
            transport_handle,
            transport_out_receiver,
        })
    }

    pub async fn run(self) -> Result<(), Error> {
        let mut transport_handle = self.transport_handle;

        // send outgoing messages to transport
        let mut transport_sink = transport_handle.get_sink();
        let mut transport_out_receiver = self.transport_out_receiver;
        let transport_sender = async move {
            while let Some(event) = transport_out_receiver.next().await {
                transport_sink.send(event).await?;
            }
            Ok::<(), Error>(())
        };

        // handle incoming messages
        let weak_inner = Arc::downgrade(&self.inner);
        let mut transport_stream = transport_handle.get_stream();
        let transport_receiver = async move {
            let mut spawn_set = OwnedSpawnSet::new();

            while let Some(event) = transport_stream.next().await {
                // cleanup any queries that have completed
                spawn_set = spawn_set.cleanup().await;

                if let InEvent::Message(msg) = event {
                    trace!(
                        "Got an incoming message. Spawn set has {} items",
                        spawn_set.len()
                    );
                    if let Err(err) =
                        Self::handle_incoming_message(&weak_inner, &mut spawn_set, msg)
                    {
                        if err.is_fatal() {
                            return Err(err);
                        } else {
                            error!("Couldn't process incoming message: {}", err);
                        }
                    }
                }
            }

            Ok::<(), Error>(())
        };

        // management timer
        let weak_inner = Arc::downgrade(&self.inner);
        let management_timer_interval = self.config.management_timer_interval;
        let management_timer = async move {
            let mut interval = interval(management_timer_interval);
            while interval.next().await.is_some() {
                Self::management_timer_process(&weak_inner)?;
            }
            Ok::<(), Error>(())
        };

        info!("Remote store server started");

        futures::select! {
            _ = transport_sender.fuse() => (),
            _ = transport_receiver.fuse() => (),
            _ = management_timer.fuse() => (),
            _ = transport_handle.fuse() => (),
        };

        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        spawn_set: &mut OwnedSpawnSet<()>,
        in_message: Box<InMessage>,
    ) -> Result<(), Error> {
        let parsed_message = IncomingMessage::parse_incoming_message(&in_message)?;

        match parsed_message {
            IncomingMessage::Mutation(mutation) => {
                Self::handle_incoming_mutation_message(
                    weak_inner, spawn_set, in_message, mutation,
                )?;
            }
            IncomingMessage::Query(query) => {
                Self::handle_incoming_query_message(weak_inner, spawn_set, in_message, query)?;
            }
            IncomingMessage::WatchedQuery(query) => {
                Self::handle_incoming_watched_query_message(
                    weak_inner,
                    spawn_set,
                    in_message.as_ref(),
                    query,
                )?;
            }
            IncomingMessage::UnwatchQuery(token) => {
                Self::handle_unwatch_query(weak_inner, token)?;
            }
        }

        Ok(())
    }

    fn handle_incoming_query_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        spawn_set: &mut OwnedSpawnSet<()>,
        in_message: Box<InMessage>,
        query: Box<EntityQuery>,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;

        let future_result = {
            let inner = inner.read()?;
            let store_handle = inner.store_handle.clone();

            async move { store_handle.query(query.as_ref().clone()).await }
        };

        let weak_inner = weak_inner.clone();
        let send_response = move |result: Result<EntityResults, Error>| -> Result<(), Error> {
            let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
            let inner = inner.read()?;

            let resp_frame = crate::query::query_results_to_response_frame(result)?;
            let message = in_message.to_response_message(&inner.cell, resp_frame)?;

            inner.send_message(message)?;

            Ok(())
        };

        spawn_set.spawn(async move {
            let result = future_result.await;

            if let Err(err) = &result {
                error!("Returning error executing incoming query: {}", err);
            }

            if let Err(err) = send_response(result) {
                error!("Error sending response for incoming query: {}", err);
            }
        });

        Ok(())
    }

    fn handle_incoming_watched_query_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        spawn_set: &mut OwnedSpawnSet<()>,
        in_message: &InMessage,
        query: Box<EntityQuery>,
    ) -> Result<(), Error> {
        let watch_token = query.watch_token;

        let weak_inner1 = weak_inner.clone();
        let (result_stream, drop_receiver) = {
            // check if this query already exists. if so, just update its last register
            let inner = weak_inner1.upgrade().ok_or(Error::Dropped)?;
            let mut inner = inner.write()?;
            if let Some(watch_query) = inner.watched_queries.get_mut(&watch_token) {
                watch_query.last_register = Instant::now();
                return Ok(());
            }

            // register query
            let (drop_sender, drop_receiver) = oneshot::channel();
            let registered_watched_query = RegisteredWatchedQuery {
                last_register: Instant::now(),
                _drop_sender: drop_sender,
            };
            inner
                .watched_queries
                .insert(watch_token, registered_watched_query);

            let result_stream = inner.store_handle.watched_query(query.as_ref().clone())?;

            (result_stream, drop_receiver)
        };

        let weak_inner1 = weak_inner.clone();
        let reply_token = in_message.get_reply_token()?;
        let send_response = move |result: Result<EntityResults, Error>| -> Result<(), Error> {
            let inner = weak_inner1.upgrade().ok_or(Error::Dropped)?;
            let inner = inner.read()?;

            let resp_frame = crate::query::query_results_to_response_frame(result)?;
            let message = reply_token.to_response_message(&inner.cell, resp_frame)?;
            inner.send_message(message)?;

            Ok(())
        };

        spawn_set.spawn(async move {
            let send_response1 = send_response.clone();
            let stream_consumer = async move {
                let mut result_stream = result_stream;
                while let Some(result) = result_stream.next().await {
                    if let Err(err) = &result {
                        error!("Returning error executing incoming query: {}", err);
                    }

                    if let Err(err) = send_response1(result) {
                        error!("Error sending response to watched query: {}", err);
                        break;
                    }
                }
            };

            futures::select! {
                _ = stream_consumer.fuse() => (),
                _ = drop_receiver.fuse() => {
                    debug!("Registered query with token {:?} got dropped", watch_token);
                   let _ = send_response(Err(Error::Dropped));
                },
            };
        });

        Ok(())
    }

    fn handle_incoming_mutation_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        spawn_set: &mut OwnedSpawnSet<()>,
        in_message: Box<InMessage>,
        request: Box<MutationRequest>,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;

        let future_result = {
            let inner = inner.read()?;
            let store_handle = inner.store_handle.clone();

            async move { store_handle.mutate(request.as_ref().clone()).await }
        };

        let weak_inner = weak_inner.clone();
        let send_response = move |result: Result<MutationResult, Error>| -> Result<(), Error> {
            let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
            let inner = inner.read()?;

            let resp_frame = crate::mutation::mutation_result_to_response_frame(result)?;
            let message = in_message.to_response_message(&inner.cell, resp_frame)?;

            inner.send_message(message)?;

            Ok(())
        };

        spawn_set.spawn(async move {
            let result = future_result.await;

            if let Err(err) = &result {
                error!("Returning error executing incoming mutation: {}", err);
            }

            if let Err(err) = send_response(result) {
                error!("Error sending response for incoming mutation: {}", err);
            }
        });

        Ok(())
    }

    fn handle_unwatch_query(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        token: WatchToken,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;
        inner.watched_queries.remove(&token);
        Ok(())
    }

    fn management_timer_process(weak_inner: &Weak<RwLock<Inner<CS, PS>>>) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;

        let timeout_duration = inner.config.watched_queries_register_timeout;
        let mut timed_out_tokens = Vec::new();
        for (token, watched_query) in &mut inner.watched_queries {
            if watched_query.last_register.elapsed() > timeout_duration {
                debug!(
                    "Watched query with token={:?} timed out after {:?}, dropping it",
                    token,
                    watched_query.last_register.elapsed(),
                );
                timed_out_tokens.push(*token);
            }
        }

        for token in timed_out_tokens {
            inner.watched_queries.remove(&token);
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct ServerConfiguration {
    pub watched_queries_register_timeout: Duration,
    pub management_timer_interval: Duration,
}

impl Default for ServerConfiguration {
    fn default() -> Self {
        ServerConfiguration {
            watched_queries_register_timeout: Duration::from_secs(20),
            management_timer_interval: Duration::from_millis(500),
        }
    }
}

struct Inner<CS, PS>
where
    CS: exocore_chain::chain::ChainStore,
    PS: exocore_chain::pending::PendingStore,
{
    config: ServerConfiguration,
    cell: Cell,
    store_handle: crate::local::StoreHandle<CS, PS>,
    watched_queries: HashMap<WatchToken, RegisteredWatchedQuery>,
    transport_out_sender: mpsc::UnboundedSender<OutEvent>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: exocore_chain::chain::ChainStore,
    PS: exocore_chain::pending::PendingStore,
{
    fn send_message(&self, message: OutMessage) -> Result<(), Error> {
        self.transport_out_sender
            .unbounded_send(OutEvent::Message(message))
            .map_err(|_err| {
                Error::Fatal(
                    "Tried to send message, but transport_out channel is closed".to_string(),
                )
            })?;

        Ok(())
    }
}

enum IncomingMessage {
    Mutation(Box<MutationRequest>),
    Query(Box<EntityQuery>),
    WatchedQuery(Box<EntityQuery>),
    UnwatchQuery(WatchToken),
}

impl IncomingMessage {
    fn parse_incoming_message(in_message: &InMessage) -> Result<IncomingMessage, Error> {
        match in_message.message_type {
            <mutation_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame = in_message.get_data_as_framed_message()?;
                let mutation = crate::mutation::mutation_from_request_frame(frame)?;
                Ok(IncomingMessage::Mutation(Box::new(mutation)))
            }
            <query_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame = in_message.get_data_as_framed_message()?;
                let query = crate::query::query_from_request_frame(frame)?;
                Ok(IncomingMessage::Query(Box::new(query)))
            }
            <watched_query_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame = in_message.get_data_as_framed_message()?;
                let query = crate::query::query_from_request_frame(frame)?;
                Ok(IncomingMessage::WatchedQuery(Box::new(query)))
            }
            <unwatch_query_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame =
                    in_message.get_data_as_framed_message::<unwatch_query_request::Owned>()?;
                let reader = frame.get_reader()?;
                let watch_token = reader.get_token();
                Ok(IncomingMessage::UnwatchQuery(watch_token))
            }
            other => Err(Error::Other(format!(
                "Received message of unknown type: {}",
                other
            ))),
        }
    }
}

struct RegisteredWatchedQuery {
    last_register: Instant,

    // selected by stream's future to get killed if we drop this query for timeout
    _drop_sender: oneshot::Sender<()>,
}
