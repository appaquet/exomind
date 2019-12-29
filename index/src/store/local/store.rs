use std::sync::{Arc, Mutex, RwLock, Weak};

use exocore_common::utils::futures::spawn_blocking;
use exocore_common::utils::handle_set::{Handle, HandleSet};
use exocore_schema::schema::Schema;
use futures::channel::{mpsc, oneshot};
use futures::compat::Stream01CompatExt;
use futures::prelude::*;

use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult, WatchToken, WatchedQuery};
use crate::store::local::watched_queries::WatchedQueries;

use super::entities_index::EntitiesIndex;
use exocore_common::cell::Cell;
use exocore_common::time::Clock;
use std::pin::Pin;
use std::task::{Context, Poll};

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

/// Locally persisted store. It forwards mutation requests to the data engine, receives back data events that get indexed
/// by the entities index. Queries are executed by the entities index.
pub struct Store<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    config: StoreConfig,
    handle_set: HandleSet,
    incoming_queries_receiver: mpsc::Receiver<QueryRequest>,
    inner: Arc<RwLock<Inner<CS, PS>>>,
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
        let (incoming_queries_sender, incoming_queries_receiver) =
            mpsc::channel::<QueryRequest>(config.query_channel_size);

        let inner = Arc::new(RwLock::new(Inner {
            cell,
            clock,
            schema,
            index,
            watched_queries: WatchedQueries::new(),
            incoming_queries_sender,
            data_handle,
        }));

        Ok(Store {
            config,
            handle_set: HandleSet::new(),
            incoming_queries_receiver,
            inner,
        })
    }

    pub fn get_handle(&self) -> StoreHandle<CS, PS> {
        StoreHandle {
            config: self.config,
            handle: self.handle_set.get_handle(),
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub async fn run(self) -> Result<(), Error> {
        let config = self.config;

        // incoming queries execution
        let incoming_queries_receiver = self.incoming_queries_receiver;
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        let queries_executor = async move {
            let mut executed_queries = incoming_queries_receiver
                .map(move |watch_request: QueryRequest| {
                    let weak_inner = weak_inner1.clone();
                    async move {
                        let result =
                            Inner::execute_query_async(weak_inner, watch_request.query.clone())
                                .await;
                        (result, watch_request)
                    }
                })
                .buffer_unordered(config.query_parallelism);

            while let Some((result, query_request)) = executed_queries.next().await {
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
                    query_request.reply(result);
                }
            }

            Ok::<(), Error>(())
        };

        let mut events_stream = {
            let mut inner = self.inner.write()?;
            inner.data_handle.take_events_stream()?.compat()
        };

        // schedule data engine events stream
        let (mut watch_check_sender, mut watch_check_receiver) = mpsc::channel(2);
        let weak_inner = Arc::downgrade(&self.inner);
        let data_events_handler = async move {
            // TODO: Should be throttled & buffered https://github.com/appaquet/exocore/issues/130
            while let Some(event) = events_stream.next().await {
                let event = event?;
                if let Err(err) = Inner::handle_data_engine_event(&weak_inner, event).await {
                    error!("Error handling data engine event: {}", err);
                    if err.is_fatal() {
                        return Err(err);
                    }
                }

                // notify query watching. if it's full, it's guaranteed that it will catch those changes on next iteration
                let _ = watch_check_sender.try_send(());
            }
            Ok(())
        };

        // checks if watched queries have their results changed
        let weak_inner = Arc::downgrade(&self.inner);
        let watched_queries_checker = async move {
            while let Some(_) = watch_check_receiver.next().await {
                let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
                let mut inner = inner.write()?;

                for watched_query in inner.watched_queries.queries() {
                    let send_result = inner.incoming_queries_sender.try_send(QueryRequest {
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
            }

            Ok::<(), Error>(())
        };

        info!("Index store started");

        futures::select! {
            _ = self.handle_set.on_handles_dropped().fuse() => (),
            _ = queries_executor.fuse() => (),
            _ = data_events_handler.fuse() => (),
            _ = watched_queries_checker.fuse() => (),
        }

        Ok(())
    }
}

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
    incoming_queries_sender: mpsc::Sender<QueryRequest>,
    data_handle: exocore_data::engine::EngineHandle<CS, PS>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
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

    async fn execute_query_async(
        weak_inner: Weak<RwLock<Inner<CS, PS>>>,
        query: Query,
    ) -> Result<QueryResult, Error> {
        let result = spawn_blocking(move || {
            let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
            let inner = inner.read()?;

            inner.index.search(&query)
        })
        .await;

        result.map_err(|err| Error::Other(format!("Couldn't launch blocking query: {}", err)))?
    }

    async fn handle_data_engine_event(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        event: exocore_data::engine::Event,
    ) -> Result<(), Error> {
        let weak_inner = weak_inner.clone();
        let result = spawn_blocking(move || -> Result<(), Error> {
            let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
            let mut inner = inner.write()?;
            inner.index.handle_data_engine_event(event)
        })
        .await;

        result.map_err(|err| Error::Other(format!("Couldn't launch blocking query: {}", err)))?
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
    handle: Handle,
    inner: Weak<RwLock<Inner<CS, PS>>>,
}

impl<CS, PS> StoreHandle<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    pub async fn on_start(&self) {
        self.handle.on_set_started().await
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

    pub fn query(
        &self,
        query: Query,
    ) -> Result<impl Future<Output = Result<QueryResult, Error>>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write().map_err(|_| Error::Dropped)?;

        let (sender, receiver) = oneshot::channel();

        // ok to dismiss send as sender end will be dropped in case of an error and consumer will be notified by channel being closed
        let _ = inner.incoming_queries_sender.try_send(QueryRequest {
            query,
            sender: QueryRequestSender::Future(sender),
        });

        Ok(async move { receiver.await.map_err(|_err| Error::Cancelled)? })
    }

    pub fn watched_query(&self, query: Query) -> Result<WatchedQueryStream<CS, PS>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write().map_err(|_| Error::Dropped)?;

        let watch_token = query
            .watch_token
            .unwrap_or_else(|| inner.clock.consistent_time(inner.cell.local_node()));
        let watched_query = WatchedQuery {
            query,
            token: watch_token,
        };

        let (sender, receiver) = mpsc::channel(self.config.handle_watch_query_channel_size);

        // ok to dismiss send as sender end will be dropped in case of an error and consumer will be notified by channel being closed
        let _ = inner.incoming_queries_sender.try_send(QueryRequest {
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

impl<CS, PS> futures::Stream for WatchedQueryStream<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    type Item = Result<QueryResult, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_next_unpin(cx)
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
    fn reply(mut self, result: Result<QueryResult, Error>) {
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
    use futures::executor::block_on_stream;
    use std::time::Duration;
    use wasm_timer::Delay;

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
        let mut stream = block_on_stream(test_store.store_handle.watched_query(query)?);

        let result = stream.next().unwrap();
        assert_eq!(result.unwrap().results.len(), 0);

        let mutation = test_store.create_put_contact_mutation("entry1", "contact1", "Hello World");
        let response = test_store.mutate(mutation)?;
        test_store
            .cluster
            .wait_operation_committed(0, response.operation_id);

        let result = stream.next().unwrap();
        assert_eq!(result.unwrap().results.len(), 1);

        let mutation =
            test_store.create_put_contact_mutation("entry2", "contact2", "Something else");
        let response = test_store.mutate(mutation)?;
        test_store
            .cluster
            .wait_operation_committed(0, response.operation_id);

        test_store.cluster.runtime.block_on_std(async move {
            let mut stream = stream.into_inner();
            let delay = Delay::new(Duration::from_secs(1));

            futures::select! {
                res = stream.next().fuse() => {
                    panic!("Got result, should have timed out");
                },
                _ = delay.fuse() => {
                    // alright
                }
            }
        });

        Ok(())
    }

    #[test]
    fn watched_query_failure() -> Result<(), failure::Error> {
        let mut test_store = TestStore::new()?;
        test_store.start_store()?;

        let query = Query::test_fail();
        let mut stream = block_on_stream(test_store.store_handle.watched_query(query)?);

        let result = stream.next().unwrap();
        assert!(result.is_err());

        Ok(())
    }
}
