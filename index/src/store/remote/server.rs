use std::sync::{Arc, RwLock, Weak};

use futures::prelude::*;
use futures::sync::{mpsc, oneshot};

use exocore_common::cell::Cell;
use exocore_common::protos::index_transport_capnp::{
    mutation_request, query_request, unwatch_query_request, watched_query_request,
};
use exocore_common::protos::MessageType;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_common::utils::futures::spawn_future;
use exocore_schema::schema::Schema;
use exocore_transport::{InEvent, InMessage, OutEvent, OutMessage, TransportHandle};

use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult, WatchToken, WatchedQuery};
use exocore_common::time::ConsistentTimestamp;
use exocore_transport::messages::MessageReplyToken;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct ServerConfiguration {
    pub watched_queries_register_timeout: Duration,
    pub management_timer_interval: Duration,
}

impl Default for ServerConfiguration {
    fn default() -> Self {
        ServerConfiguration {
            watched_queries_register_timeout: Duration::from_secs(30),
            management_timer_interval: Duration::from_millis(500),
        }
    }
}

pub struct Server<CS, PS, T>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
    T: TransportHandle,
{
    config: ServerConfiguration,
    start_notifier: CompletionNotifier<(), Error>,
    started: bool,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    transport_handle: Option<T>,
    stop_listener: CompletionListener<(), Error>,
}

impl<CS, PS, T> Server<CS, PS, T>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
    T: TransportHandle,
{
    pub fn new(
        config: ServerConfiguration,
        cell: Cell,
        schema: Arc<Schema>,
        store_handle: crate::store::local::StoreHandle<CS, PS>,
        transport_handle: T,
    ) -> Result<Server<CS, PS, T>, Error> {
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();
        let start_notifier = CompletionNotifier::new();

        let inner = Arc::new(RwLock::new(Inner {
            config,
            cell,
            schema,
            store_handle,
            watched_queries: HashMap::new(),
            transport_out: None,
            stop_notifier,
        }));

        Ok(Server {
            config,
            start_notifier,
            started: false,
            inner,
            transport_handle: Some(transport_handle),
            stop_listener,
        })
    }

    fn start(&mut self) -> Result<(), Error> {
        let mut transport_handle = self
            .transport_handle
            .take()
            .expect("Transport handle was already consumed");

        let mut inner = self.inner.write()?;

        // send outgoing messages to transport
        let (out_sender, out_receiver) = mpsc::unbounded();
        spawn_future(
            out_receiver
                .forward(transport_handle.get_sink().sink_map_err(|_err| ()))
                .map(|_| ()),
        );
        inner.transport_out = Some(out_sender);

        // handle incoming messages
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        spawn_future(
            transport_handle
                .get_stream()
                .map_err(|err| Error::Fatal(format!("Error in incoming transport stream: {}", err)))
                .for_each(move |event| {
                    debug!("Got an incoming message");
                    match event {
                        InEvent::Message(msg) => {
                            if let Err(err) = Self::handle_incoming_message(&weak_inner1, msg) {
                                if err.is_fatal() {
                                    return Err(err);
                                } else {
                                    error!("Couldn't process incoming message: {}", err);
                                }
                            }
                        }
                        InEvent::NodeStatus(_, _) => {
                            // TODO: Do something
                        }
                    }

                    Ok(())
                })
                .map(|_| ())
                .map_err(move |err| {
                    Inner::notify_stop("incoming transport stream", &weak_inner2, Err(err));
                }),
        );

        // management time
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        spawn_future(
            tokio::timer::Interval::new_interval(self.config.management_timer_interval)
                .map_err(|err| Error::Fatal(format!("Management timer error: {}", err)))
                .for_each(move |_| Self::management_timer_process(&weak_inner1))
                .map_err(move |err| {
                    Inner::notify_stop("management timer error", &weak_inner2, Err(err));
                }),
        );

        // schedule transport handle
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        spawn_future(
            transport_handle
                .map(move |_| {
                    info!("Transport is done");
                    Inner::notify_stop("transport completion", &weak_inner1, Ok(()));
                })
                .map_err(move |err| {
                    Inner::notify_stop("transport error", &weak_inner2, Err(err.into()));
                }),
        );

        self.start_notifier.complete(Ok(()));
        info!("Remote store server started");

        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        in_message: Box<InMessage>,
    ) -> Result<(), Error> {
        let parsed_message = {
            let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
            let inner = inner.read()?;
            IncomingMessage::parse_incoming_message(&in_message, &inner.schema)?
        };

        match parsed_message {
            IncomingMessage::Mutation(mutation) => {
                Self::handle_incoming_mutation_message(weak_inner, in_message, mutation)?;
            }
            IncomingMessage::Query(query) => {
                Self::handle_incoming_query_message(weak_inner, in_message, query)?;
            }
            IncomingMessage::WatchedQuery(query) => {
                Self::handle_incoming_watched_query_message(weak_inner, in_message, query)?;
            }
            IncomingMessage::UnwatchQuery(token) => {
                Self::handle_unwatch_query(weak_inner, token)?;
            }
        }

        Ok(())
    }

    fn handle_incoming_query_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        in_message: Box<InMessage>,
        query: Query,
    ) -> Result<(), Error> {
        let weak_inner1 = weak_inner.clone();
        let future_result = {
            let inner = weak_inner1.upgrade().ok_or(Error::Dropped)?;
            let inner = inner.read()?;
            inner.store_handle.query(query.clone())
        }?;

        let weak_inner2 = weak_inner.clone();
        spawn_future(
            future_result
                .then(move |result| {
                    let inner = weak_inner2.upgrade().ok_or(Error::Dropped)?;
                    let inner = inner.read()?;

                    if let Err(err) = &result {
                        error!("Returning error executing incoming query: {}", err);
                    }

                    let resp_frame = QueryResult::result_to_response_frame(&inner.schema, result)?;
                    let message = in_message.to_response_message(&inner.cell, resp_frame)?;
                    inner.send_message(message)?;

                    Ok(())
                })
                .map_err(|err: Error| {
                    error!("Error executing incoming query: {}", err);
                }),
        );

        Ok(())
    }

    fn handle_incoming_watched_query_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        in_message: Box<InMessage>,
        watched_query: WatchedQuery,
    ) -> Result<(), Error> {
        let weak_inner1 = weak_inner.clone();
        let weak_inner2 = weak_inner.clone();
        let weak_inner3 = weak_inner.clone();

        let watch_token = watched_query.token;
        let (result_stream, timeout_receiver) = {
            // check if this query already exists. if so, just update its last register
            let inner = weak_inner1.upgrade().ok_or(Error::Dropped)?;
            let mut inner = inner.write()?;
            if let Some(watch_query) = inner.watched_queries.get_mut(&watch_token) {
                watch_query.last_register = Instant::now();
                return Ok(());
            }

            // register query
            let (timeout_sender, timeout_receiver) = oneshot::channel();
            let registered_watched_query = RegisteredWatchedQuery {
                last_register: Instant::now(),
                _timeout_sender: timeout_sender,
            };
            inner
                .watched_queries
                .insert(watch_token, registered_watched_query);

            let query = watched_query.query.clone().with_watch_token(watch_token);
            let result_stream = inner.store_handle.watched_query(query)?;

            (result_stream, timeout_receiver)
        };

        let reply_token1 = in_message.get_reply_token()?;
        let reply_token2 = reply_token1.clone();
        spawn_future(
            result_stream
                .then(move |result| -> Result<(), Error> {
                    let inner = weak_inner2.upgrade().ok_or(Error::Dropped)?;
                    let inner = inner.read()?;

                    if let Err(err) = &result {
                        error!("Returning error executing incoming query: {}", err);
                    }

                    Self::reply_query_result(&inner, &reply_token1, result)?;

                    Ok(())
                })
                .for_each(|_| Ok(()))
                .map_err(|err| {
                    error!("Error in watched query stream: {}", err);
                })
                .select({
                    // channel will be dropped and stream killed if we have a registering timeout
                    timeout_receiver.map_err(|_| ())
                })
                .map_err(move |_| {
                    if let Some(inner) = weak_inner3.upgrade() {
                        if let Ok(inner) = inner.read() {
                            let _ = Self::reply_query_result(
                                &inner,
                                &reply_token2,
                                Err(Error::Other(
                                    "Error or timeout in watched query".to_string(),
                                )),
                            );
                        }
                    }
                })
                .map(|_| ()),
        );

        Ok(())
    }

    fn reply_query_result(
        inner: &Inner<CS, PS>,
        reply_token: &MessageReplyToken,
        result: Result<QueryResult, Error>,
    ) -> Result<(), Error> {
        let resp_frame = QueryResult::result_to_response_frame(&inner.schema, result)?;
        let message = reply_token.to_response_message(&inner.cell, resp_frame)?;
        inner.send_message(message)?;
        Ok(())
    }

    fn handle_incoming_mutation_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        in_message: Box<InMessage>,
        mutation: Mutation,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let inner = inner.read()?;
        let result = inner.store_handle.mutate(mutation);

        if let Err(err) = &result {
            error!("Returning error executing incoming mutation: {}", err);
        }

        let resp_frame = MutationResult::result_to_response_frame(&inner.schema, result)?;
        let message = in_message.to_response_message(&inner.cell, resp_frame)?;
        inner.send_message(message)?;

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
                error!(
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

impl<CS, PS, T> Future for Server<CS, PS, T>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
    T: TransportHandle,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.started {
            self.start()?;
            self.started = true;
        }

        // check if store got stopped
        self.stop_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            _ => Error::Other("Error in completion error".to_string()),
        })
    }
}

struct Inner<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    config: ServerConfiguration,
    cell: Cell,
    schema: Arc<Schema>,
    store_handle: crate::store::local::StoreHandle<CS, PS>,
    watched_queries: HashMap<WatchToken, RegisteredWatchedQuery>,
    transport_out: Option<mpsc::UnboundedSender<OutEvent>>,
    stop_notifier: CompletionNotifier<(), Error>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    fn send_message(&self, message: OutMessage) -> Result<(), Error> {
        let transport = self.transport_out.as_ref().ok_or_else(|| {
            Error::Fatal("Tried to send message, but transport_out was none".to_string())
        })?;

        transport
            .unbounded_send(OutEvent::Message(message))
            .map_err(|_err| {
                Error::Fatal(
                    "Tried to send message, but transport_out channel is closed".to_string(),
                )
            })?;

        Ok(())
    }

    fn notify_stop(
        future_name: &str,
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        res: Result<(), Error>,
    ) {
        match &res {
            Ok(()) => info!("Local store has completed"),
            Err(err) => error!("Got an error in future {}: {}", future_name, err),
        }

        if let Some(locked_inner) = weak_inner.upgrade() {
            if let Ok(inner) = locked_inner.read() {
                inner.stop_notifier.complete(res);
            }
        };
    }
}

enum IncomingMessage {
    Mutation(Mutation),
    Query(Query),
    WatchedQuery(WatchedQuery),
    UnwatchQuery(WatchToken),
}

impl IncomingMessage {
    fn parse_incoming_message(
        in_message: &InMessage,
        schema: &Arc<Schema>,
    ) -> Result<IncomingMessage, Error> {
        match in_message.message_type {
            <mutation_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame = in_message.get_data_as_framed_message()?;
                let mutation = Mutation::from_mutation_request_frame(schema, frame)?;
                Ok(IncomingMessage::Mutation(mutation))
            }
            <query_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame = in_message.get_data_as_framed_message()?;
                let query = Query::from_request_frame(schema, frame)?;
                Ok(IncomingMessage::Query(query))
            }
            <watched_query_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame = in_message.get_data_as_framed_message()?;
                let query = WatchedQuery::from_request_frame(schema, frame)?;
                Ok(IncomingMessage::WatchedQuery(query))
            }
            <unwatch_query_request::Owned as MessageType>::MESSAGE_TYPE => {
                let frame =
                    in_message.get_data_as_framed_message::<unwatch_query_request::Owned>()?;
                let reader = frame.get_reader()?;
                let watch_token = ConsistentTimestamp(reader.get_token());
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
    _timeout_sender: oneshot::Sender<()>,
}
