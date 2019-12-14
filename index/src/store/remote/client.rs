use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll};
use std::time::Duration;

use futures::channel::{mpsc, oneshot};
use futures::compat::{Future01CompatExt, Sink01CompatExt, Stream01CompatExt};
use futures::prelude::*;

use exocore_common::cell::Cell;
use exocore_common::framing::CapnpFrameBuilder;
use exocore_common::node::Node;
use exocore_common::protos::index_transport_capnp::{
    mutation_response, query_response, unwatch_query_request, watched_query_response,
};
use exocore_common::protos::MessageType;
use exocore_common::time::Instant;
use exocore_common::time::{Clock, ConsistentTimestamp};
use exocore_common::utils::completion_notifier::{CompletionListener, CompletionNotifier};
use exocore_schema::schema::Schema;
use exocore_transport::{
    InEvent, InMessage, OutEvent, OutMessage, TransportHandle, TransportLayer,
};

use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult, WatchToken, WatchedQuery};

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

    pub async fn run(mut self) -> Result<(), Error> {
        let mut transport_handle = self
            .transport_handle
            .take()
            .expect("Transport handle was already consumed");

        // create a channel through which we will receive message from our handles to be sent to transport
        let out_receiver = {
            let mut inner = self.inner.write()?;
            let (out_sender, out_receiver) = mpsc::unbounded();
            inner.transport_out = Some(out_sender);
            out_receiver
        };

        // send outgoing messages to transport
        let mut transport_sink = transport_handle.get_sink().sink_compat();
        let transport_sender = async move {
            let mut receiver = out_receiver;

            while let Some(item) = receiver.next().await {
                transport_sink.send(item).await?;
            }

            Ok::<(), Error>(())
        };

        // handle incoming messages from transport
        let weak_inner = Arc::downgrade(&self.inner);
        let mut transport_stream = transport_handle.get_stream().compat();
        let transport_receiver = async move {
            while let Some(event) = transport_stream.next().await {
                if let InEvent::Message(msg) = event? {
                    if let Err(err) = Inner::handle_incoming_message(&weak_inner, msg) {
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

        // management timer that checks for timed out queries & register watched queries
        let weak_inner = Arc::downgrade(&self.inner);
        let management_interval = self.config.management_interval;
        let manager = async move {
            let mut timer = wasm_timer::Interval::new(management_interval);

            while let Some(_) = timer.next().await {
                Inner::management_timer_process(&weak_inner)?;
            }

            Ok::<(), Error>(())
        };

        // notify handles that we have started
        self.start_notifier.complete(Ok(()));

        // wait for handles to be completed
        let stop_listener = self.stop_listener.compat();

        futures::select! {
            _ = transport_sender.fuse() => (),
            _ = transport_receiver.fuse() => (),
            _ = manager.fuse() => (),
            _ = transport_handle.compat().fuse() => (),
            _ = stop_listener.fuse() => (),
        };

        Ok(())
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

    fn send_mutation(
        &mut self,
        mutation: Mutation,
    ) -> Result<oneshot::Receiver<Result<MutationResult, Error>>, Error> {
        let (result_sender, receiver) = oneshot::channel();

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
        let (result_sender, receiver) = oneshot::channel();

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
    pub async fn on_start(&self) -> Result<(), Error> {
        let listener = {
            self.start_listener
                .try_clone()
                .map_err(|_err| Error::Dropped)?
                .compat()
        };

        listener.await.map_err(|_err| Error::Dropped)
    }

    pub async fn mutate(&self, mutation: Mutation) -> Result<MutationResult, Error> {
        let result = {
            let inner = match self.inner.upgrade() {
                Some(inner) => inner,
                None => return Err(Error::Dropped),
            };
            let mut inner = match inner.write() {
                Ok(inner) => inner,
                Err(err) => return Err(err.into()),
            };

            inner.send_mutation(mutation)?
        };

        result.await.map_err(|_err| Error::Cancelled)?
    }

    pub fn query(&self, query: Query) -> QueryFuture {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return Error::Dropped.into(),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(_err) => return Error::Poisoned.into(),
        };

        let (request_id, receiver) = match inner.send_query(query) {
            Ok((request_id, receiver)) => (request_id, receiver),
            Err(err) => return err.into(),
        };

        QueryFuture {
            result: Ok(receiver),
            request_id,
        }
    }

    pub fn watched_query(&self, query: Query) -> WatchedQueryStream {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return WatchedQueryStream::from_error(self.inner.clone(), Error::Dropped),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(err) => return WatchedQueryStream::from_error(self.inner.clone(), err.into()),
        };

        let watch_token = query
            .watch_token
            .unwrap_or_else(|| inner.clock.consistent_time(inner.cell.local_node()));

        let watch_query = WatchedQuery {
            query,
            token: watch_token,
        };

        let (request_id, receiver) = match inner.watch_query(watch_query) {
            Ok(tup) => tup,
            Err(err) => return WatchedQueryStream::from_error(self.inner.clone(), err),
        };

        WatchedQueryStream {
            inner: self.inner.clone(),
            watch_token: Some(watch_token),
            request_id,
            result: Ok(receiver),
        }
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
    result: Result<oneshot::Receiver<Result<QueryResult, Error>>, Error>,
    request_id: ConsistentTimestamp,
}

impl QueryFuture {
    pub fn query_id(&self) -> ConsistentTimestamp {
        self.request_id
    }
}

impl From<Error> for QueryFuture {
    fn from(err: Error) -> Self {
        QueryFuture {
            result: Err(err),
            request_id: ConsistentTimestamp(0),
        }
    }
}

impl Future for QueryFuture {
    type Output = Result<QueryResult, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.result.as_mut() {
            Err(err) => Poll::Ready(Err(err.clone())),
            Ok(receiver) => receiver
                .poll_unpin(cx)
                .map(|res| res.map_err(|_| Error::Cancelled).and_then(|res| res))
                .map_err(|_err| Error::Cancelled),
        }
    }
}

pub struct WatchedQueryStream {
    inner: Weak<RwLock<Inner>>,
    watch_token: Option<WatchToken>,
    request_id: ConsistentTimestamp,
    result: Result<mpsc::Receiver<Result<QueryResult, Error>>, Error>,
}

impl WatchedQueryStream {
    pub fn query_id(&self) -> ConsistentTimestamp {
        self.request_id
    }

    fn from_error(inner: Weak<RwLock<Inner>>, err: Error) -> Self {
        WatchedQueryStream {
            inner,
            watch_token: None,
            result: Err(err),
            request_id: ConsistentTimestamp(0),
        }
    }
}

impl Stream for WatchedQueryStream {
    type Item = Result<QueryResult, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.result.as_mut() {
            Err(err) => Poll::Ready(Some(Err(err.clone()))),
            Ok(stream) => stream.poll_next_unpin(cx),
        }
    }
}

impl Drop for WatchedQueryStream {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.watched_queries.remove(&self.request_id);

                if let Some(watch_token) = self.watch_token {
                    let _ = inner.send_unwatch_query(watch_token);
                }
            }
        }
    }
}
