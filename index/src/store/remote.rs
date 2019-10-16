use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult};
use crate::store::{AsyncResult, AsyncStore};
use exocore_common::cell::Cell;
use exocore_common::node::Node;
use exocore_common::protos::index_transport_capnp::{mutation_response, query_response};
use exocore_common::protos::MessageType;
use exocore_common::time::Instant;
use exocore_common::time::{Clock, ConsistentTimestamp};
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_schema::schema::Schema;
use exocore_transport::{InMessage, OutMessage, TransportHandle, TransportLayer};
use futures::prelude::*;
use futures::sync::{mpsc, oneshot};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

// TODO: To be moved https://github.com/appaquet/exocore/issues/123
pub type FutureSpawner = Box<dyn Fn(Box<dyn Future<Item = (), Error = ()> + Send>) + Send>;

///
/// Configuration for remote store
///
#[derive(Debug, Clone, Copy)]
pub struct StoreConfiguration {
    /// Duration before considering query has timed out if not responded
    pub query_timeout: Duration,

    /// Duration before considering mutation has timed out if not responded
    pub mutation_timeout: Duration,

    /// Interval at which we should check for pending queries and mutations timeout
    pub timeout_check_interval: Duration,
}

impl Default for StoreConfiguration {
    fn default() -> Self {
        StoreConfiguration {
            query_timeout: Duration::from_secs(10),
            mutation_timeout: Duration::from_secs(5),
            timeout_check_interval: Duration::from_millis(100),
        }
    }
}

///
/// This implementation of the AsyncStore allow sending all queries and mutations to
/// a remote node's local store.
///
pub struct RemoteStore<T>
where
    T: TransportHandle,
{
    config: StoreConfiguration,
    future_spawner: FutureSpawner,
    start_notifier: CompletionNotifier<(), Error>,
    started: bool,
    inner: Arc<RwLock<Inner>>,
    transport_handle: Option<T>,
    stop_listener: CompletionListener<(), Error>,
}

impl<T> RemoteStore<T>
where
    T: TransportHandle,
{
    pub fn new(
        config: StoreConfiguration,
        cell: Cell,
        clock: Clock,
        schema: Arc<Schema>,
        transport_handle: T,
        index_node: Node,
        future_spawner: FutureSpawner,
    ) -> Result<RemoteStore<T>, Error> {
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();
        let start_notifier = CompletionNotifier::new();

        let inner = Arc::new(RwLock::new(Inner {
            config,
            cell,
            clock,
            schema,
            transport_out: None,
            index_node,
            pending_queries: HashMap::new(),
            pending_mutations: HashMap::new(),
            stop_notifier,
        }));

        Ok(RemoteStore {
            config,
            future_spawner,
            start_notifier,
            started: false,
            inner,
            transport_handle: Some(transport_handle),
            stop_listener,
        })
    }

    pub fn get_handle(&self) -> Result<StoreHandle, Error> {
        let start_listener = self
            .start_notifier
            .get_listener()
            .expect("Couldn't get a listener on start notifier");
        Ok(StoreHandle {
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
        (self.future_spawner)(Box::new(
            out_receiver
                .forward(transport_handle.get_sink().sink_map_err(|_err| ()))
                .map(|_| ()),
        ));
        inner.transport_out = Some(out_sender);

        // handle incoming messages
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        (self.future_spawner)(Box::new(
            transport_handle
                .get_stream()
                .map_err(|err| Error::Fatal(format!("Error in incoming transport stream: {}", err)))
                .for_each(move |in_message| {
                    if let Err(err) = Self::handle_incoming_message(&weak_inner1, in_message) {
                        if err.is_fatal() {
                            return Err(err);
                        } else {
                            error!("Couldn't process incoming message: {}", err);
                        }
                    }
                    Ok(())
                })
                .map(|_| ())
                .map_err(move |err| {
                    Inner::notify_stop("incoming transport stream", &weak_inner2, Err(err));
                }),
        ));

        // schedule transport handle
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        (self.future_spawner)(Box::new(
            transport_handle
                .map(move |_| {
                    info!("Transport is done");
                    Inner::notify_stop("transport completion", &weak_inner1, Ok(()));
                })
                .map_err(move |err| {
                    Inner::notify_stop("transport error", &weak_inner2, Err(err.into()));
                }),
        ));

        // schedule query & mutation requests timeout checker
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        (self.future_spawner)(Box::new(
            wasm_timer::Interval::new_interval(self.config.timeout_check_interval)
                .map_err(|err| Error::Fatal(format!("Timer error: {}", err)))
                .for_each(move |_| Self::check_requests_timout(&weak_inner1))
                .map_err(move |err| {
                    Inner::notify_stop("timeout check error", &weak_inner2, Err(err));
                }),
        ));

        self.start_notifier.complete(Ok(()));
        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner>>,
        in_message: InMessage,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut inner = inner.write()?;

        let request_id = if let Some(rendez_vous_id) = in_message.rendez_vous_id {
            rendez_vous_id
        } else {
            return Err(Error::Other(format!(
                "Got an InMessage without a follow id (type={:?} from={:?})",
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
            Ok(IncomingMessage::QueryResponse(query)) => {
                if let Some(pending_request) = inner.pending_queries.remove(&request_id) {
                    let _ = pending_request.result_sender.send(Ok(query));
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
                } else if let Some(pending_request) = inner.pending_queries.remove(&request_id) {
                    let _ = pending_request.result_sender.send(Err(err));
                }
            }
        }

        Ok(())
    }

    fn check_requests_timout(weak_inner: &Weak<RwLock<Inner>>) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut inner = inner.write()?;

        let query_timeout = inner.config.query_timeout;
        Inner::check_map_requests_timeouts(&mut inner.pending_queries, query_timeout);

        let mutation_timeout = inner.config.mutation_timeout;
        Inner::check_map_requests_timeouts(&mut inner.pending_mutations, mutation_timeout);

        Ok(())
    }
}

impl<T> Future for RemoteStore<T>
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

///
/// Inner instance of the store
///
struct Inner {
    config: StoreConfiguration,
    cell: Cell,
    clock: Clock,
    schema: Arc<Schema>,
    transport_out: Option<mpsc::UnboundedSender<OutMessage>>,
    index_node: Node,
    pending_queries: HashMap<ConsistentTimestamp, PendingRequest<QueryResult>>,
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
    ) -> Result<oneshot::Receiver<Result<QueryResult, Error>>, Error> {
        let (result_sender, receiver) = futures::oneshot();

        let request_id = self.clock.consistent_time(self.cell.local_node());
        let request_frame = query.to_query_request_frame(&self.schema)?;
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

        Ok(receiver)
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

    fn send_message(&self, message: OutMessage) -> Result<(), Error> {
        let transport = self.transport_out.as_ref().ok_or_else(|| {
            Error::Fatal("Tried to send message, but transport_out was none".to_string())
        })?;

        transport.unbounded_send(message).map_err(|_err| {
            Error::Fatal("Tried to send message, but transport_out channel is closed".to_string())
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

///
/// Async handle to the store
///
pub struct StoreHandle {
    start_listener: CompletionListener<(), Error>,
    inner: Weak<RwLock<Inner>>,
}

impl StoreHandle {
    pub fn on_start(&self) -> Result<impl Future<Item = (), Error = Error>, Error> {
        // TODO: Should only return result
        Ok(self
            .start_listener
            .try_clone()
            .map_err(|_err| Error::Other("Couldn't clone start listener in handle".to_string()))?
            .map_err(|err| match err {
                CompletionError::UserError(err) => err,
                _ => Error::Other("Error in completion error".to_string()),
            }))
    }
}

impl AsyncStore for StoreHandle {
    fn mutate(&self, mutation: Mutation) -> AsyncResult<MutationResult> {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return Box::new(futures::failed(Error::InnerUpgrade)),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(err) => return Box::new(futures::failed(err.into())),
        };

        match inner.send_mutation(mutation) {
            Ok(receiver) => Box::new(receiver.then(|res| match res {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(_err) => Err(Error::Other("Mutation got cancelled".to_string())),
            })),
            Err(err) => Box::new(futures::failed(err)),
        }
    }

    fn query(&self, query: Query) -> AsyncResult<QueryResult> {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return Box::new(futures::failed(Error::InnerUpgrade)),
        };
        let mut inner = match inner.write() {
            Ok(inner) => inner,
            Err(err) => return Box::new(futures::failed(err.into())),
        };

        match inner.send_query(query) {
            Ok(receiver) => Box::new(receiver.then(|res| match res {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(_err) => Err(Error::Other("Query got cancelled".to_string())),
            })),
            Err(err) => Box::new(futures::failed(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutation::TestFailMutation;
    use crate::store::local::store::tests::TestLocalStore;
    use exocore_common::node::LocalNode;
    use exocore_common::tests_utils::expect_eventually;
    use exocore_transport::mock::MockTransportHandle;

    #[test]
    fn mutation_and_query() -> Result<(), failure::Error> {
        let mut test_remote_store = TestRemoteStore::new()?;
        test_remote_store.start_local()?;
        test_remote_store.start_remote()?;

        let mutation = test_remote_store
            .local_store
            .create_put_contact_mutation("entity1", "trait1", "hello");
        test_remote_store.send_and_await_mutation(mutation)?;

        expect_eventually(|| {
            let query = Query::match_text("hello");
            let results = test_remote_store.send_and_await_query(query).unwrap();
            results.results.len() == 1
        });

        Ok(())
    }

    #[test]
    fn mutation_error_propagation() -> Result<(), failure::Error> {
        let mut test_remote_store = TestRemoteStore::new()?;
        test_remote_store.start_local()?;
        test_remote_store.start_remote()?;

        let mutation = Mutation::TestFail(TestFailMutation {});
        let result = test_remote_store.send_and_await_mutation(mutation);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn query_error_propagation() -> Result<(), failure::Error> {
        let mut test_remote_store = TestRemoteStore::new()?;
        test_remote_store.start_local()?;
        test_remote_store.start_remote()?;

        let mutation = test_remote_store
            .local_store
            .create_put_contact_mutation("entity1", "trait1", "hello");
        test_remote_store.send_and_await_mutation(mutation)?;

        let query = Query::test_fail();
        let result = test_remote_store.send_and_await_query(query);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn query_timeout() -> Result<(), failure::Error> {
        let config = StoreConfiguration {
            query_timeout: Duration::from_millis(500),
            ..StoreConfiguration::default()
        };

        let mut test_remote_store = TestRemoteStore::new_with_configuration(config)?;

        // only start remote, so local won't answer and it should timeout
        test_remote_store.start_remote()?;

        let query = Query::match_text("hello");
        let result = test_remote_store.send_and_await_query(query);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn mutation_timeout() -> Result<(), failure::Error> {
        let config = StoreConfiguration {
            mutation_timeout: Duration::from_millis(500),
            ..StoreConfiguration::default()
        };

        let mut test_remote_store = TestRemoteStore::new_with_configuration(config)?;

        // only start remote, so local won't answer and it should timeout
        test_remote_store.start_remote()?;

        let mutation = test_remote_store
            .local_store
            .create_put_contact_mutation("entity1", "trait1", "hello");
        let result = test_remote_store.send_and_await_mutation(mutation);
        assert!(result.is_err());

        Ok(())
    }

    struct TestRemoteStore {
        local_store: TestLocalStore,
        remote_store: Option<RemoteStore<MockTransportHandle>>,
        remote_handle: StoreHandle,
    }

    impl TestRemoteStore {
        fn new() -> Result<TestRemoteStore, failure::Error> {
            let config = StoreConfiguration::default();
            Self::new_with_configuration(config)
        }

        fn new_with_configuration(
            config: StoreConfiguration,
        ) -> Result<TestRemoteStore, failure::Error> {
            let local_store = TestLocalStore::new()?;

            let local_node = LocalNode::generate();
            let remote_store = RemoteStore::new(
                config,
                local_store.cluster.cells[0].cell().clone(),
                local_store.cluster.clocks[0].clone(),
                local_store.schema.clone(),
                local_store
                    .cluster
                    .transport_hub
                    .get_transport(local_node, TransportLayer::Index),
                local_store.cluster.nodes[0].node().clone(),
                Box::new(tokio_future_spawner),
            )?;

            let remote_handle = remote_store.get_handle()?;

            Ok(TestRemoteStore {
                local_store,

                remote_store: Some(remote_store),
                remote_handle,
            })
        }

        fn start_local(&mut self) -> Result<(), failure::Error> {
            self.local_store.start_store()?;
            Ok(())
        }

        fn start_remote(&mut self) -> Result<(), failure::Error> {
            let remote_store = self.remote_store.take().unwrap();
            self.local_store
                .cluster
                .runtime
                .spawn(remote_store.map_err(|err| {
                    error!("Error spawning remote store: {}", err);
                }));

            self.local_store
                .cluster
                .runtime
                .block_on(self.remote_handle.on_start()?)?;

            Ok(())
        }

        fn send_and_await_mutation(&mut self, mutation: Mutation) -> Result<MutationResult, Error> {
            self.local_store
                .cluster
                .runtime
                .block_on(self.remote_handle.mutate(mutation))
        }

        fn send_and_await_query(&mut self, query: Query) -> Result<QueryResult, Error> {
            self.local_store
                .cluster
                .runtime
                .block_on(self.remote_handle.query(query))
        }
    }

    fn tokio_future_spawner(future: Box<dyn Future<Item = (), Error = ()> + Send>) {
        tokio::spawn(future);
    }
}
