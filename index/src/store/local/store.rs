use std::sync::{Arc, Mutex, RwLock, Weak};

use futures::prelude::*;
use futures::sync::{mpsc, oneshot};
use tokio::prelude::*;

use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_common::utils::futures::spawn_future;
use exocore_schema::schema::Schema;

use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult, WatchToken, WatchedQuery};
use crate::store::local::watched_queries::WatchedQueries;

use super::entities_index::EntitiesIndex;
use exocore_common::cell::Cell;
use exocore_common::time::Clock;

/// Config for `Store`
#[derive(Clone, Copy)]
pub struct StoreConfig {
    pub query_channel_size: usize,
    pub query_parallelism: usize,
    pub handle_watch_query_channel_size: usize,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            query_channel_size: 1000,
            query_parallelism: 4,
            handle_watch_query_channel_size: 1000,
        }
    }
}

/// Locally persisted store. It uses a data engine handle and entities index to
/// perform mutations and resolve queries.
pub struct Store<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    start_notifier: CompletionNotifier<(), Error>,
    config: StoreConfig,
    started: bool,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    stop_listener: CompletionListener<(), Error>,
}

impl<CS, PS> Store<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    pub fn new(
        config: StoreConfig,
        cell: Cell,
        clock: Clock,
        schema: Arc<Schema>,
        data_handle: exocore_data::engine::EngineHandle<CS, PS>,
        index: EntitiesIndex<CS, PS>,
    ) -> Result<Store<CS, PS>, Error> {
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();
        let start_notifier = CompletionNotifier::new();

        let watched = WatchedQueries::new();
        let inner = Arc::new(RwLock::new(Inner {
            cell,
            clock,
            schema,
            index,
            watched_queries: watched,
            queries_sender: None,
            data_handle,
            stop_notifier,
        }));

        Ok(Store {
            start_notifier,
            config,
            started: false,
            inner,
            stop_listener,
        })
    }

    pub fn get_handle(&self) -> StoreHandle<CS, PS> {
        let start_listener = self
            .start_notifier
            .get_listener()
            .expect("Couldn't get a listener on start notifier");

        StoreHandle {
            config: self.config,
            start_listener,
            inner: Arc::downgrade(&self.inner),
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        // incoming queries execution
        let (queries_sender, queries_receiver) = mpsc::channel(self.config.query_channel_size);
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        let weak_inner3 = Arc::downgrade(&self.inner);
        spawn_future(
            queries_receiver
                .map_err(|_| Error::Dropped)
                .map(move |watch_request: QueryRequest| {
                    debug!("Executing new query");
                    Inner::execute_query_async(weak_inner1.clone(), watch_request.query.clone())
                        .then(|result| Ok((result, watch_request)))
                })
                .buffer_unordered(self.config.query_parallelism)
                .for_each(move |(result, query_request)| {
                    let inner = weak_inner2.upgrade().ok_or(Error::Dropped)?;
                    let inner = inner.read()?;

                    match &result {
                        Ok(_) => debug!("Got query result"),
                        Err(err) => warn!("Error executing query: err={}", err),
                    }

                    let should_reply = match (&query_request.sender, &result) {
                        (QueryRequestSender::Stream(sender, watch_token), Ok(result)) => {
                            inner.watched_queries.update_query_results(
                                *watch_token,
                                &query_request.query,
                                result,
                                sender.clone(),
                            )
                        }

                        (QueryRequestSender::Stream(_, watch_token), Err(_err)) => {
                            inner.watched_queries.unwatch_query(*watch_token);
                            true
                        }

                        (QueryRequestSender::Future(_), _result) => true,
                    };

                    if should_reply {
                        query_request.send(result);
                    }

                    Ok(())
                })
                .map_err(move |_| {
                    Inner::notify_stop(
                        "watched query events stream",
                        &weak_inner3,
                        Err(Error::Dropped),
                    )
                }),
        );
        inner.queries_sender = Some(queries_sender);

        // schedule data engine events stream
        let (mut watch_check_sender, watch_check_receiver) = mpsc::channel(2);
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        let weak_inner3 = Arc::downgrade(&self.inner);
        spawn_future(
            inner
                .data_handle
                .take_events_stream()?
                // TODO: Should be throttled & buffered https://github.com/appaquet/exocore/issues/130
                .map_err(|err| err.into())
                .for_each(move |event| {
                    if let Err(err) = Self::handle_data_engine_event(&weak_inner1, event) {
                        if err.is_fatal() {
                            return Err(err);
                        } else {
                            error!("Error handling data engine event: {}", err);
                        }
                    }

                    // notify query watching. if it's full, it's guaranteed that it will catch those changes on next iteration
                    let _ = watch_check_sender.try_send(());

                    Ok(())
                })
                .map(move |_| {
                    Inner::notify_stop("data engine event stream completion", &weak_inner2, Ok(()))
                })
                .map_err(move |err| {
                    Inner::notify_stop("data engine event stream", &weak_inner3, Err(err))
                }),
        );

        // checks if watched queries have their results changed
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        spawn_future(
            watch_check_receiver
                .map_err(|_| Error::Dropped)
                .for_each(move |_| {
                    let inner = weak_inner1.upgrade().ok_or(Error::Dropped)?;
                    let mut inner = inner.write()?;

                    for watched_query in inner.watched_queries.queries() {
                        let send_result =
                            inner
                                .queries_sender
                                .as_mut()
                                .unwrap()
                                .try_send(QueryRequest {
                                    query: watched_query.query.as_ref().clone(),
                                    sender: QueryRequestSender::Stream(
                                        watched_query.sender.clone(),
                                        watched_query.token,
                                    ),
                                });

                        if let Err(err) = send_result {
                            error!(
                                "Error sending to watched query. Removing it from queries: {}",
                                err
                            );
                            inner.watched_queries.unwatch_query(watched_query.token);
                        }
                    }

                    Ok(())
                })
                .map_err(move |_| {
                    Inner::notify_stop(
                        "watched queries checker stream",
                        &weak_inner2,
                        Err(Error::Dropped),
                    )
                }),
        );

        self.start_notifier.complete(Ok(()));
        info!("Index store started");

        Ok(())
    }

    fn handle_data_engine_event(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        event: exocore_data::engine::Event,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;
        inner.index.handle_data_engine_event(event)?;
        Ok(())
    }
}

impl<CS, PS> Future for Store<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
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
struct Inner<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    cell: Cell,
    clock: Clock,
    schema: Arc<Schema>,
    index: EntitiesIndex<CS, PS>,
    watched_queries: WatchedQueries,
    queries_sender: Option<mpsc::Sender<QueryRequest>>,
    data_handle: exocore_data::engine::EngineHandle<CS, PS>,
    stop_notifier: CompletionNotifier<(), Error>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
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

    fn write_mutation_weak(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        mutation: Mutation,
    ) -> Result<MutationResult, Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let inner = inner.read()?;
        inner.write_mutation(mutation)
    }

    fn write_mutation(&self, mutation: Mutation) -> Result<MutationResult, Error> {
        #[cfg(test)]
        {
            if let Mutation::TestFail(_mutation) = &mutation {
                return Err(Error::Other("TestFail mutation".to_string()));
            }
        }

        let json_mutation = mutation.to_json(self.schema.clone())?;
        let operation_id = self
            .data_handle
            .write_entry_operation(json_mutation.as_bytes())?;

        Ok(MutationResult { operation_id })
    }

    fn execute_query_async(
        weak_inner: Weak<RwLock<Inner<CS, PS>>>,
        query: Query,
    ) -> impl Future<Item = QueryResult, Error = Error> {
        future::lazy(|| {
            future::poll_fn(move || {
                let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
                let inner = inner.read()?;

                let res = tokio_threadpool::blocking(|| inner.index.search(&query));
                match res {
                    Ok(Async::Ready(Ok(results))) => Ok(Async::Ready(results)),
                    Ok(Async::Ready(Err(err))) => Err(err),
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Err(err) => Err(Error::Other(format!(
                        "Error executing query in blocking block: {}",
                        err
                    ))),
                }
            })
        })
        .map_err(|err| Error::Other(format!("Error executing query in blocking block: {}", err)))
    }
}

///
/// Handle to the store, allowing communication to the store asynchronously
///
pub struct StoreHandle<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    config: StoreConfig,
    start_listener: CompletionListener<(), Error>,
    inner: Weak<RwLock<Inner<CS, PS>>>,
}

impl<CS, PS> StoreHandle<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
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

    #[cfg(test)]
    pub(crate) fn watched_queries(&self) -> Vec<WatchToken> {
        let inner = self.inner.upgrade().unwrap();
        let inner = inner.read().unwrap();

        let mut tokens = Vec::new();
        for query in inner.watched_queries.queries() {
            tokens.push(query.token);
        }

        tokens
    }

    pub fn mutate(&self, mutation: Mutation) -> Result<MutationResult, Error> {
        Inner::write_mutation_weak(&self.inner, mutation)
    }

    pub fn query(&self, query: Query) -> Result<QueryFuture, Error> {
        let inner = if let Some(inner) = self.inner.upgrade() {
            inner
        } else {
            return Err(Error::Dropped);
        };

        let mut inner = if let Ok(inner) = inner.write() {
            inner
        } else {
            return Err(Error::Dropped);
        };

        let (sender, receiver) = oneshot::channel();
        let new_sender = inner.queries_sender.as_mut().expect("Queries sender channel was not initialized. A query was made before the store was started.");

        // ok to dismiss send as sender end will be dropped in case of an error and consumer will be notified by channel being closed
        let _ = new_sender.try_send(QueryRequest {
            query,
            sender: QueryRequestSender::Future(sender),
        });

        Ok(QueryFuture { receiver })
    }

    pub fn watched_query(&self, query: Query) -> Result<WatchedQueryStream<CS, PS>, Error> {
        let inner = if let Some(inner) = self.inner.upgrade() {
            inner
        } else {
            return Err(Error::Dropped);
        };

        let mut inner = if let Ok(inner) = inner.write() {
            inner
        } else {
            return Err(Error::Dropped);
        };

        let watch_token = query
            .watch_token
            .unwrap_or_else(|| inner.clock.consistent_time(inner.cell.local_node()));
        let watched_query = WatchedQuery {
            query,
            token: watch_token,
        };

        let (sender, receiver) = mpsc::channel(self.config.handle_watch_query_channel_size);
        let new_sender = inner.queries_sender.as_mut().expect("Queries sender channel was not initialized. A query was made before the store was started.");

        // ok to dismiss send as sender end will be dropped in case of an error and consumer will be notified by channel being closed
        let _ = new_sender.try_send(QueryRequest {
            query: watched_query.query,
            sender: QueryRequestSender::Stream(Arc::new(Mutex::new(sender)), watch_token),
        });

        Ok(WatchedQueryStream {
            watch_token,
            inner: self.inner.clone(),
            receiver,
        })
    }
}

/// Future result of a query.
pub struct QueryFuture {
    receiver: oneshot::Receiver<Result<QueryResult, Error>>,
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

/// A query received through the `watched_query` method that needs to be watched and notified
/// when new changes happen to the store that would affects its results.
pub struct WatchedQueryStream<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    watch_token: WatchToken,
    inner: Weak<RwLock<Inner<CS, PS>>>,
    receiver: mpsc::Receiver<Result<QueryResult, Error>>,
}

impl<CS, PS> Stream for WatchedQueryStream<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
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

impl<CS, PS> Drop for WatchedQueryStream<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(inner) = inner.read() {
                inner.watched_queries.unwatch_query(self.watch_token);
            }
        }
    }
}

/// Incoming query to be executed, or re-scheduled watched query to be re-executed.
struct QueryRequest {
    query: Query,
    sender: QueryRequestSender,
}

enum QueryRequestSender {
    Stream(
        Arc<Mutex<mpsc::Sender<Result<QueryResult, Error>>>>,
        WatchToken,
    ),
    Future(oneshot::Sender<Result<QueryResult, Error>>),
}

impl QueryRequest {
    fn send(mut self, result: Result<QueryResult, Error>) {
        match self.sender {
            QueryRequestSender::Future(sender) => {
                let _ = sender.send(result);
            }
            QueryRequestSender::Stream(ref mut sender, _token) => {
                if let Ok(mut sender) = sender.lock() {
                    let _ = sender.try_send(result);
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::mutation::TestFailMutation;

    use super::*;
    use crate::store::local::TestStore;
    use std::time::Duration;

    #[test]
    fn store_mutate_query_via_handle() -> Result<(), failure::Error> {
        let mut test_store = TestStore::new()?;
        test_store.start_store()?;

        let mutation = test_store.create_put_contact_mutation("entry1", "contact1", "Hello World");
        let response = test_store.mutate(mutation)?;
        test_store
            .cluster
            .wait_operation_committed(0, response.operation_id);

        let query = Query::match_text("hello");
        let results = test_store.query(query)?;
        assert_eq!(results.results.len(), 1);

        Ok(())
    }

    #[test]
    fn query_error_propagating() -> Result<(), failure::Error> {
        let mut test_store = TestStore::new()?;
        test_store.start_store()?;

        let query = Query::test_fail();
        assert!(test_store.query(query).is_err());

        Ok(())
    }

    #[test]
    fn mutation_error_propagating() -> Result<(), failure::Error> {
        let mut test_store = TestStore::new()?;
        test_store.start_store()?;

        let mutation = Mutation::TestFail(TestFailMutation {});
        assert!(test_store.mutate(mutation).is_err());

        Ok(())
    }

    #[test]
    fn watched_query() -> Result<(), failure::Error> {
        let mut test_store = TestStore::new()?;
        test_store.start_store()?;

        let query = Query::match_text("hello");
        let stream = test_store.store_handle.watched_query(query)?;

        let (result, stream) = test_store
            .cluster
            .runtime
            .block_on(stream.into_future())
            .map_err(|_| ())
            .unwrap();
        assert_eq!(result.unwrap().results.len(), 0);

        let mutation = test_store.create_put_contact_mutation("entry1", "contact1", "Hello World");
        let response = test_store.mutate(mutation)?;
        test_store
            .cluster
            .wait_operation_committed(0, response.operation_id);

        let (result, stream) = test_store
            .cluster
            .runtime
            .block_on(stream.into_future())
            .map_err(|_| ())
            .unwrap();
        assert_eq!(result.unwrap().results.len(), 1);

        let mutation =
            test_store.create_put_contact_mutation("entry2", "contact2", "Something else");
        let response = test_store.mutate(mutation)?;
        test_store
            .cluster
            .wait_operation_committed(0, response.operation_id);

        let result = test_store
            .cluster
            .runtime
            .block_on(stream.into_future().timeout(Duration::from_secs(1)));

        match &result {
            Err(err) if err.is_elapsed() => {
                // fine
            }
            _ => {
                panic!("Expected timeout, got something else");
            }
        }

        Ok(())
    }

    #[test]
    fn watched_query_failure() -> Result<(), failure::Error> {
        let mut test_store = TestStore::new()?;
        test_store.start_store()?;

        let query = Query::test_fail();
        let stream = test_store.store_handle.watched_query(query)?;

        let result = test_store.cluster.runtime.block_on(stream.into_future());
        assert!(result.is_err());

        Ok(())
    }
}
