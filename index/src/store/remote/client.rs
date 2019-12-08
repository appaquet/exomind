use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult, WatchToken, WatchedQuery};
use exocore_common::cell::Cell;
use exocore_common::framing::CapnpFrameBuilder;
use exocore_common::node::Node;
use exocore_common::protos::index_transport_capnp::{
    mutation_response, query_response, unwatch_query_request, watched_query_response,
};
use exocore_common::protos::MessageType;
use exocore_common::time::Instant;
use exocore_common::time::{Clock, ConsistentTimestamp};
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_common::utils::futures::spawn_future;
use exocore_schema::schema::Schema;
use exocore_transport::{
    InEvent, InMessage, OutEvent, OutMessage, TransportHandle, TransportLayer,
};
use futures::prelude::*;
use futures::sync::{mpsc, oneshot};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct ClientConfiguration {
    pub query_timeout: Duration,
    pub mutation_timeout: Duration,
    pub management_interval: Duration,
    pub watched_queries_register_interval: Duration,
    pub watched_query_channel_size: usize,
}

impl Default for ClientConfiguration {
    fn default() -> Self {
        ClientConfiguration {
            query_timeout: Duration::from_secs(10),
            mutation_timeout: Duration::from_secs(5),
            watched_queries_register_interval: Duration::from_secs(10),
            management_interval: Duration::from_millis(100),
            watched_query_channel_size: 1000,
        }
    }
}

/// This implementation of the AsyncStore allow sending all queries and mutations to
/// a remote node's local store running the `Server` component.
pub struct Client<T>
where
    T: TransportHandle,
{
    config: ClientConfiguration,
    start_notifier: CompletionNotifier<(), Error>,
    started: bool,
    inner: Arc<RwLock<Inner>>,
    transport_handle: Option<T>,
    stop_listener: CompletionListener<(), Error>,
}

impl<T> Client<T>
where
    T: TransportHandle,
{
    pub fn new(
        config: ClientConfiguration,
        cell: Cell,
        clock: Clock,
        schema: Arc<Schema>,
        transport_handle: T,
        index_node: Node,
    ) -> Result<Client<T>, Error> {
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();
        let start_notifier = CompletionNotifier::new();

        let inner = Arc::new(RwLock::new(Inner {
            config,
            cell,
            clock,
            schema,
            transport_out: None,
            handles_count: 0,
            index_node,
            pending_queries: HashMap::new(),
            watched_queries: HashMap::new(),
            pending_mutations: HashMap::new(),
            stop_notifier,
        }));

        Ok(Client {
            config,
            start_notifier,
            started: false,
            inner,
            transport_handle: Some(transport_handle),
            stop_listener,
        })
    }

    pub fn get_handle(&self) -> Result<ClientHandle, Error> {
        let mut inner = self.inner.write()?;

        let start_listener = self
            .start_notifier
            .get_listener()
            .expect("Couldn't get a listener on start notifier");

        inner.handles_count += 1;

        Ok(ClientHandle {
            start_listener,
            inner: Arc::downgrade(&self.inner),
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
                            // TODO: Do something with node status
                        }
                    }
                    Ok(())
                })
                .map(|_| ())
                .map_err(move |err| {
                    Inner::notify_stop("incoming transport stream", &weak_inner2, Err(err));
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

        // management timer that checks for timed out queries & register watched queries
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        spawn_future(
            wasm_timer::Interval::new_interval(self.config.management_interval)
                .map_err(|err| Error::Fatal(format!("Management timer error: {}", err)))
                .for_each(move |_| Self::management_timer_process(&weak_inner1))
                .map_err(move |err| {
                    Inner::notify_stop("management timer error", &weak_inner2, Err(err));
                }),
        );

        self.start_notifier.complete(Ok(()));
        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner>>,
        in_message: Box<InMessage>,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;

        let request_id = if let Some(rendez_vous_id) = in_message.rendez_vous_id {
            rendez_vous_id
        } else {
            return Err(Error::Other(format!(
                "Got an InMessage without a rendez_vous_id (type={:?} from={:?})",
                in_message.message_type, in_message.from
            )));
        };

        match IncomingMessage::parse_incoming_message(&in_message, &inner.schema) {
            Ok(IncomingMessage::MutationResponse(mutation)) => {
                if let Some(pending_request) = inner.pending_mutations.remove(&request_id) {
                    let _ = pending_request.result_sender.send(Ok(mutation));
                } else {
                    return Err(Error::Other(format!(
                        "Couldn't find pending mutation for mutation response (request_id={:?} type={:?} from={:?})",
                        request_id, in_message.message_type, in_message.from
                    )));
                }
            }
            Ok(IncomingMessage::QueryResponse(result)) => {
                if let Some(pending_request) = inner.pending_queries.remove(&request_id) {
                    let _ = pending_request.result_sender.send(Ok(result));
                } else if let Some(watched_query) = inner.watched_queries.get_mut(&request_id) {
                    let _ = watched_query.result_sender.try_send(Ok(result));
                } else {
                    return Err(Error::Other(format!(
                        "Couldn't find pending query for query response (request_id={:?} type={:?} from={:?})",
                        request_id, in_message.message_type, in_message.from
                    )));
                }
            }
            Err(err) => {
                if let Some(pending_request) = inner.pending_mutations.remove(&request_id) {
                    let _ = pending_request.result_sender.send(Err(err));
                } else if let Some(mut watched_query) = inner.watched_queries.remove(&request_id) {
                    let _ = watched_query.result_sender.try_send(Err(err));
                } else if let Some(pending_request) = inner.pending_queries.remove(&request_id) {
                    let _ = pending_request.result_sender.send(Err(err));
                }
            }
        }

        Ok(())
    }

    fn management_timer_process(weak_inner: &Weak<RwLock<Inner>>) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;

        let query_timeout = inner.config.query_timeout;
        Inner::check_map_requests_timeouts(&mut inner.pending_queries, query_timeout);

        let mutation_timeout = inner.config.mutation_timeout;
        Inner::check_map_requests_timeouts(&mut inner.pending_mutations, mutation_timeout);

        inner.send_watched_queries_keepalive();

        Ok(())
    }
}

impl<T> Future for Client<T>
where
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

struct Inner {
    config: ClientConfiguration,
    cell: Cell,
    clock: Clock,
    schema: Arc<Schema>,
    transport_out: Option<mpsc::UnboundedSender<OutEvent>>,
    handles_count: usize,
    index_node: Node,
    pending_queries: HashMap<ConsistentTimestamp, PendingRequest<QueryResult>>,
    watched_queries: HashMap<ConsistentTimestamp, WatchedQueryRequest>,
    pending_mutations: HashMap<ConsistentTimestamp, PendingRequest<MutationResult>>,
    stop_notifier: CompletionNotifier<(), Error>,
}

impl Inner {
    fn send_mutation(
        &mut self,
        mutation: Mutation,
    ) -> Result<oneshot::Receiver<Result<MutationResult, Error>>, Error> {
        let (result_sender, receiver) = futures::oneshot();

        let request_id = self.clock.consistent_time(self.cell.local_node());
        let request_frame = mutation.to_mutation_request_frame(&self.schema)?;
        let message =
            OutMessage::from_framed_message(&self.cell, TransportLayer::Index, request_frame)?
                .with_to_node(self.index_node.clone())
                .with_expiration(Some(Instant::now() + self.config.mutation_timeout))
                .with_rendez_vous_id(request_id);
        self.send_message(message)?;

        self.pending_mutations.insert(
            request_id,
            PendingRequest {
                request_id,
                result_sender,
                send_time: Instant::now(),
            },
        );

        Ok(receiver)
    }

    fn send_query(
        &mut self,
        query: Query,
    ) -> Result<
        (
            ConsistentTimestamp,
            oneshot::Receiver<Result<QueryResult, Error>>,
        ),
        Error,
    > {
        let (result_sender, receiver) = futures::oneshot();

        let request_id = self.clock.consistent_time(self.cell.local_node());
        let request_frame = query.to_request_frame(&self.schema)?;
        let message =
            OutMessage::from_framed_message(&self.cell, TransportLayer::Index, request_frame)?
                .with_to_node(self.index_node.clone())
                .with_expiration(Some(Instant::now() + self.config.query_timeout))
                .with_rendez_vous_id(request_id);
        self.send_message(message)?;

        self.pending_queries.insert(
            request_id,
            PendingRequest {
                request_id,
                result_sender,
                send_time: Instant::now(),
            },
        );

        Ok((request_id, receiver))
    }

    fn watch_query(
        &mut self,
        watched_query: WatchedQuery,
    ) -> Result<
        (
            ConsistentTimestamp,
            mpsc::Receiver<Result<QueryResult, Error>>,
        ),
        Error,
    > {
        let (result_sender, receiver) = mpsc::channel(self.config.watched_query_channel_size);
        let request_id = self.clock.consistent_time(self.cell.local_node());
        let watched_query = WatchedQueryRequest {
            request_id,
            result_sender,
            query: watched_query,
            last_register: Instant::now(),
        };

        self.send_watch_query(&watched_query)?;
        self.watched_queries.insert(request_id, watched_query);

        Ok((request_id, receiver))
    }

    fn send_watch_query(&self, watched_query: &WatchedQueryRequest) -> Result<(), Error> {
        let request_frame = watched_query.query.to_request_frame(&self.schema)?;
        let message =
            OutMessage::from_framed_message(&self.cell, TransportLayer::Index, request_frame)?
                .with_to_node(self.index_node.clone())
                .with_rendez_vous_id(watched_query.request_id);

        self.send_message(message)
    }

    fn send_unwatch_query(&self, token: WatchToken) -> Result<(), Error> {
        let mut frame_builder = CapnpFrameBuilder::<unwatch_query_request::Owned>::new();
        let mut message_builder = frame_builder.get_builder();
        message_builder.set_token(token.0);

        let message =
            OutMessage::from_framed_message(&self.cell, TransportLayer::Index, frame_builder)?
                .with_to_node(self.index_node.clone());

        self.send_message(message)
    }

    fn check_map_requests_timeouts<T>(
        requests: &mut HashMap<ConsistentTimestamp, PendingRequest<T>>,
        timeout: Duration,
    ) {
        let mut timed_out_requests = Vec::new();
        for request in requests.values() {
            if request.send_time.elapsed() > timeout {
                timed_out_requests.push(request.request_id);
            }
        }

        for request_id in timed_out_requests {
            if let Some(request) = requests.remove(&request_id) {
                let _ = request
                    .result_sender
                    .send(Err(Error::Timeout(request.send_time.elapsed(), timeout)));
            }
        }
    }

    fn send_watched_queries_keepalive(&mut self) {
        let register_interval = self.config.watched_queries_register_interval;

        let mut sent_queries = Vec::new();
        for (token, query) in &self.watched_queries {
            if query.last_register.elapsed() > register_interval {
                if let Err(err) = self.send_watch_query(query) {
                    error!("Couldn't send watch query: {}", err);
                }
                sent_queries.push(*token);
            }
        }

        for token in &sent_queries {
            let query = self.watched_queries.get_mut(token).unwrap();
            query.last_register = Instant::now();
        }
    }

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

    fn notify_stop(future_name: &str, weak_inner: &Weak<RwLock<Inner>>, res: Result<(), Error>) {
        match &res {
            Ok(()) => info!("Local store has completed"),
            Err(err) => error!("Got an error in future {}: {}", future_name, err),
        }

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

        inner.stop_notifier.complete(res);
    }
}

///
/// Parsed incoming message via transport
///
enum IncomingMessage {
    MutationResponse(MutationResult),
    QueryResponse(QueryResult),
}

impl IncomingMessage {
    fn parse_incoming_message(
        in_message: &InMessage,
        schema: &Arc<Schema>,
    ) -> Result<IncomingMessage, Error> {
        match in_message.message_type {
            <mutation_response::Owned as MessageType>::MESSAGE_TYPE => {
                let mutation_frame = in_message.get_data_as_framed_message()?;
                let mutation_result = MutationResult::from_response_frame(schema, mutation_frame)?;
                Ok(IncomingMessage::MutationResponse(mutation_result))
            }
            <query_response::Owned as MessageType>::MESSAGE_TYPE => {
                let query_frame = in_message.get_data_as_framed_message()?;
                let query_result = QueryResult::from_query_frame(schema, query_frame)?;
                Ok(IncomingMessage::QueryResponse(query_result))
            }
            <watched_query_response::Owned as MessageType>::MESSAGE_TYPE => {
                let query_frame = in_message.get_data_as_framed_message()?;
                let query_result = QueryResult::from_query_frame(schema, query_frame)?;
                Ok(IncomingMessage::QueryResponse(query_result))
            }
            other => Err(Error::Other(format!(
                "Received message of unknown type: {}",
                other
            ))),
        }
    }
}

///
/// Query or mutation request for which we're waiting a response
///
struct PendingRequest<T> {
    request_id: ConsistentTimestamp,
    result_sender: oneshot::Sender<Result<T, Error>>,
    send_time: Instant,
}

struct WatchedQueryRequest {
    request_id: ConsistentTimestamp,
    query: WatchedQuery,
    result_sender: mpsc::Sender<Result<QueryResult, Error>>,
    last_register: Instant,
}

///
/// Async handle to the store
///
pub struct ClientHandle {
    start_listener: CompletionListener<(), Error>,
    inner: Weak<RwLock<Inner>>,
}

impl ClientHandle {
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

    pub fn mutate(
        &self,
        mutation: Mutation,
    ) -> Result<impl Future<Item = MutationResult, Error = Error>, Error> {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return Err(Error::Dropped),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(err) => return Err(err.into()),
        };

        Ok(inner
            .send_mutation(mutation)?
            .map_err(|_| Error::Other("Mutation future channel closed".to_string()))
            .and_then(|res| res))
    }

    pub fn query(&self, query: Query) -> Result<QueryFuture, Error> {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return Err(Error::Dropped),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(err) => return Err(err.into()),
        };

        let (request_id, receiver) = inner.send_query(query)?;
        Ok(QueryFuture {
            receiver,
            request_id,
        })
    }

    pub fn watched_query(&self, query: Query) -> Result<WatchedQueryStream, Error> {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return Err(Error::Dropped),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(err) => return Err(err.into()),
        };

        let watch_token = query
            .watch_token
            .unwrap_or_else(|| inner.clock.consistent_time(inner.cell.local_node()));
        let watch_query = WatchedQuery {
            query,
            token: watch_token,
        };

        let (request_id, receiver) = inner.watch_query(watch_query)?;
        Ok(WatchedQueryStream {
            inner: self.inner.clone(),
            watch_token,
            request_id,
            receiver,
        })
    }

    pub fn cancel_query(&self, query_id: ConsistentTimestamp) -> Result<(), Error> {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return Err(Error::Dropped),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(err) => return Err(err.into()),
        };

        if let Some(query) = inner.watched_queries.remove(&query_id) {
            debug!("Cancelling watched query {:?}", query_id);
            let _ = inner.send_unwatch_query(query.query.token);
        } else {
            debug!("Cancelling query {:?}", query_id);
            inner.pending_queries.remove(&query_id);
        }

        Ok(())
    }
}

impl Drop for ClientHandle {
    fn drop(&mut self) {
        debug!("Client handle got dropped");
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.handles_count -= 1;

                if inner.handles_count == 0 {
                    info!("Last handle got dropped. Stopping client.");
                    inner.stop_notifier.complete(Ok(()));
                }
            }
        }
    }
}

pub struct QueryFuture {
    receiver: oneshot::Receiver<Result<QueryResult, Error>>,
    request_id: ConsistentTimestamp,
}

impl QueryFuture {
    pub fn query_id(&self) -> ConsistentTimestamp {
        self.request_id
    }
}

impl Future for QueryFuture {
    type Item = QueryResult;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<QueryResult>, Error> {
        match self.receiver.poll() {
            Ok(Async::Ready(Ok(res))) => Ok(Async::Ready(res)),
            Ok(Async::Ready(Err(err))) => Err(err),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(Error::Other(
                "Error polling query result future".to_string(),
            )),
        }
    }
}

pub struct WatchedQueryStream {
    inner: Weak<RwLock<Inner>>,
    watch_token: WatchToken,
    request_id: ConsistentTimestamp,
    receiver: mpsc::Receiver<Result<QueryResult, Error>>,
}

impl WatchedQueryStream {
    pub fn query_id(&self) -> ConsistentTimestamp {
        self.request_id
    }
}

impl Stream for WatchedQueryStream {
    type Item = QueryResult;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        let res = self.receiver.poll();
        match res {
            Ok(Async::Ready(Some(Ok(result)))) => Ok(Async::Ready(Some(result))),
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
            Ok(Async::Ready(Some(Err(err)))) => Err(err),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(Error::Other(
                "Error polling watch query channel".to_string(),
            )),
        }
    }
}

impl Drop for WatchedQueryStream {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.watched_queries.remove(&self.request_id);
                let _ = inner.send_unwatch_query(self.watch_token);
            }
        }
    }
}
