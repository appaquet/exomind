use super::entities_index::EntitiesIndex;
use crate::domain::schema::Schema;
use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult};
use crate::store::{AsyncResult, AsyncStore};
use exocore_common::cell::FullCell;
use exocore_common::protos::index_transport_capnp::{mutation_request, query_request};
use exocore_common::protos::MessageType;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_transport::{InMessage, OutMessage, TransportHandle};
use futures::prelude::*;
use futures::sync::mpsc;
use std::sync::{Arc, RwLock, Weak};
use tokio::prelude::*;

///
/// Locally persisted store. It uses a data engine handle and entities index to
/// perform mutations and resolve queries
///
pub struct LocalStore<CS, PS, T>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
    T: TransportHandle,
{
    start_notifier: CompletionNotifier<(), Error>,
    started: bool,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    transport_handle: Option<T>,
    stop_listener: CompletionListener<(), Error>,
}

impl<CS, PS, T> LocalStore<CS, PS, T>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
    T: TransportHandle,
{
    pub fn new(
        cell: FullCell,
        schema: Arc<Schema>,
        data_handle: exocore_data::engine::EngineHandle<CS, PS>,
        index: EntitiesIndex<CS, PS>,
        transport_handle: T,
    ) -> Result<LocalStore<CS, PS, T>, Error> {
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();
        let start_notifier = CompletionNotifier::new();

        let inner = Arc::new(RwLock::new(Inner {
            cell,
            schema,
            index,
            data_handle,
            transport_out: None,
            stop_notifier,
        }));

        Ok(LocalStore {
            start_notifier,
            started: false,
            inner,
            transport_handle: Some(transport_handle),
            stop_listener,
        })
    }

    pub fn get_handle(&self) -> Result<StoreHandle<CS, PS>, Error> {
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
        tokio::spawn(
            out_receiver
                .forward(transport_handle.get_sink().sink_map_err(|_err| ()))
                .map(|_| ()),
        );
        inner.transport_out = Some(out_sender);

        // handle incoming messages
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        tokio::spawn(
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
        );

        // schedule transport handle
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        tokio::spawn(
            transport_handle
                .map(move |_| {
                    info!("Transport is done");
                    Inner::notify_stop("transport completion", &weak_inner1, Ok(()));
                })
                .map_err(move |err| {
                    Inner::notify_stop("transport error", &weak_inner2, Err(err.into()));
                }),
        );

        // schedule data engine events stream
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        let weak_inner3 = Arc::downgrade(&self.inner);
        tokio::spawn(
            inner
                .data_handle
                .take_events_stream()?
                .map_err(|err| err.into())
                .for_each(move |event| {
                    if let Err(err) = Self::handle_data_engine_event(&weak_inner1, event) {
                        if err.is_fatal() {
                            return Err(err);
                        } else {
                            error!("Error handling data engine event: {}", err);
                        }
                    }
                    Ok(())
                })
                .map(move |_| {
                    Inner::notify_stop("data engine event stream completion", &weak_inner2, Ok(()))
                })
                .map_err(move |err| {
                    Inner::notify_stop("data engine event stream", &weak_inner3, Err(err))
                }),
        );

        self.start_notifier.complete(Ok(()));

        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        in_message: InMessage,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let inner = inner.read()?;

        match IncomingMessage::parse_incoming_message(&in_message, &inner.schema)? {
            IncomingMessage::Mutation(mutation) => {
                Self::handle_incoming_mutation_message(weak_inner, in_message, mutation)?;
            }
            IncomingMessage::Query(query) => {
                Self::handle_incoming_query_message(weak_inner, in_message, query)?;
            }
        }

        Ok(())
    }

    fn handle_incoming_query_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        in_message: InMessage,
        query: Query,
    ) -> Result<(), Error> {
        let weak_inner1 = weak_inner.clone();
        let weak_inner2 = weak_inner.clone();
        tokio::spawn(
            Inner::execute_query_async(weak_inner1, query)
                .then(move |result| {
                    let inner = weak_inner2.upgrade().ok_or(Error::InnerUpgrade)?;
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

    fn handle_incoming_mutation_message(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        in_message: InMessage,
        mutation: Mutation,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let inner = inner.read()?;

        let result = inner.write_mutation(mutation);
        if let Err(err) = &result {
            error!("Returning error executing incoming mutation: {}", err);
        }

        let resp_frame = MutationResult::result_to_response_frame(&inner.schema, result)?;
        let message = in_message.to_response_message(&inner.cell, resp_frame)?;
        inner.send_message(message)?;

        Ok(())
    }

    fn handle_data_engine_event(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        event: exocore_data::engine::Event,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut inner = inner.write()?;
        inner.index.handle_data_engine_event(event)?;
        Ok(())
    }
}

impl<CS, PS, T> Future for LocalStore<CS, PS, T>
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

///
/// Inner instance of the store
///
struct Inner<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    cell: FullCell,
    schema: Arc<Schema>,
    index: EntitiesIndex<CS, PS>,
    data_handle: exocore_data::engine::EngineHandle<CS, PS>,
    transport_out: Option<mpsc::UnboundedSender<OutMessage>>,
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

    fn write_mutation_weak(
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        mutation: Mutation,
    ) -> Result<MutationResult, Error> {
        let inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
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
        // TODO: Use a bounded threadpool instead of executing on current executor: https://github.com/appaquet/exocore/issues/113
        future::lazy(|| {
            future::poll_fn(move || {
                let inner = weak_inner.upgrade().ok_or(Error::InnerUpgrade)?;
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

    fn send_message(&self, message: OutMessage) -> Result<(), Error> {
        let transport = self.transport_out.as_ref().ok_or_else(|| {
            Error::Fatal("Tried to send message, but transport_out was none".to_string())
        })?;

        transport.unbounded_send(message).map_err(|_err| {
            Error::Fatal("Tried to send message, but transport_out channel is closed".to_string())
        })?;

        Ok(())
    }
}

///
/// Parsed incoming message via transport
///
enum IncomingMessage {
    Mutation(Mutation),
    Query(Query),
}

impl IncomingMessage {
    fn parse_incoming_message(
        in_message: &InMessage,
        schema: &Arc<Schema>,
    ) -> Result<IncomingMessage, Error> {
        match in_message.message_type {
            <mutation_request::Owned as MessageType>::MESSAGE_TYPE => {
                let mutation_frame = in_message.get_data_as_framed_message()?;
                let mutation = Mutation::from_mutation_request_frame(schema, mutation_frame)?;
                Ok(IncomingMessage::Mutation(mutation))
            }
            <query_request::Owned as MessageType>::MESSAGE_TYPE => {
                let query_frame = in_message.get_data_as_framed_message()?;
                let query = Query::from_query_request_frame(schema, query_frame)?;
                Ok(IncomingMessage::Query(query))
            }
            other => Err(Error::Other(format!(
                "Received message of unknown type: {}",
                other
            ))),
        }
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
}

impl<CS, PS> AsyncStore for StoreHandle<CS, PS>
where
    CS: exocore_data::chain::ChainStore,
    PS: exocore_data::pending::PendingStore,
{
    fn mutate(&self, mutation: Mutation) -> AsyncResult<MutationResult> {
        Box::new(future::result(Inner::write_mutation_weak(
            &self.inner,
            mutation,
        )))
    }

    fn query(&self, query: Query) -> AsyncResult<QueryResult> {
        let weak_inner = self.inner.clone();
        Box::new(Inner::execute_query_async(weak_inner, query))
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::entities_index::EntitiesIndexConfig;
    use super::super::traits_index::TraitsIndexConfig;
    use super::*;
    use crate::domain::entity::{EntityId, Record, Trait, TraitId};
    use crate::domain::schema::tests::create_test_schema;
    use crate::mutation::{MutationResult, PutTraitMutation, TestFailMutation};
    use exocore_common::node::LocalNode;
    use exocore_data::tests_utils::DataTestCluster;
    use exocore_data::{DirectoryChainStore, MemoryPendingStore};
    use exocore_transport::mock::MockTransportHandle;
    use exocore_transport::transport::{MpscHandleSink, MpscHandleStream};
    use exocore_transport::TransportLayer;
    use failure::err_msg;
    use tempdir::TempDir;

    #[test]
    fn store_mutate_query_via_handle() -> Result<(), failure::Error> {
        let mut test_store = TestLocalStore::new()?;
        test_store.start_store()?;

        let mutation = test_store.create_put_contact_mutation("entry1", "contact1", "Hello World");
        let response = test_store.mutate_via_handle(mutation)?;
        test_store
            .cluster
            .wait_operation_committed(0, response.operation_id);

        let query = Query::match_text("hello");
        let results = test_store.query_via_handle(query)?;
        assert_eq!(results.results.len(), 1);

        Ok(())
    }

    #[test]
    fn store_mutate_query_via_transport() -> Result<(), failure::Error> {
        let mut test_store = TestLocalStore::new()?;
        test_store.start_store()?;

        let mutation = test_store.create_put_contact_mutation("entry1", "contact1", "Hello World");
        let response = test_store.mutate_via_transport(mutation)?;
        test_store
            .cluster
            .wait_operation_committed(0, response.operation_id);

        let query = Query::match_text("hello");
        let results = test_store.query_via_transport(query)?;
        assert_eq!(results.results.len(), 1);

        Ok(())
    }

    #[test]
    fn query_error_propagating() -> Result<(), failure::Error> {
        let mut test_store = TestLocalStore::new()?;
        test_store.start_store()?;

        let query = Query::test_fail();
        assert!(test_store.query_via_handle(query).is_err());

        let query = Query::test_fail();
        assert!(test_store.query_via_transport(query).is_err());

        Ok(())
    }

    #[test]
    fn mutation_error_propagating() -> Result<(), failure::Error> {
        let mut test_store = TestLocalStore::new()?;
        test_store.start_store()?;

        let mutation = Mutation::TestFail(TestFailMutation {});
        assert!(test_store.mutate_via_handle(mutation).is_err());

        let mutation = Mutation::TestFail(TestFailMutation {});
        assert!(test_store.mutate_via_transport(mutation).is_err());

        Ok(())
    }

    ///
    /// Utility to test local store
    ///
    pub struct TestLocalStore {
        pub cluster: DataTestCluster,
        pub schema: Arc<Schema>,

        store: Option<LocalStore<DirectoryChainStore, MemoryPendingStore, MockTransportHandle>>,
        store_handle: StoreHandle<DirectoryChainStore, MemoryPendingStore>,
        _temp_dir: TempDir,

        // external node & transport used to communicate with store
        external_node: LocalNode,
        external_transport_sink: Option<MpscHandleSink>,
        external_transport_stream: Option<MpscHandleStream>,
    }

    impl TestLocalStore {
        pub fn new() -> Result<TestLocalStore, failure::Error> {
            let mut cluster = DataTestCluster::new_single_and_start()?;

            let temp_dir = tempdir::TempDir::new("store")?;
            let schema = create_test_schema();

            let index_config = EntitiesIndexConfig {
                pending_index_config: TraitsIndexConfig {
                    indexer_num_threads: Some(1),
                    ..TraitsIndexConfig::default()
                },
                chain_index_config: TraitsIndexConfig {
                    indexer_num_threads: Some(1),
                    ..TraitsIndexConfig::default()
                },
                ..EntitiesIndexConfig::default()
            };
            let index = EntitiesIndex::<DirectoryChainStore, MemoryPendingStore>::open_or_create(
                temp_dir.path(),
                index_config,
                schema.clone(),
                cluster.get_handle(0).try_clone()?,
            )?;

            let transport = cluster
                .transport_hub
                .get_transport(cluster.nodes[0].clone(), TransportLayer::Index);

            let store = LocalStore::new(
                cluster.cells[0].clone(),
                schema.clone(),
                cluster.get_new_handle(0),
                index,
                transport,
            )?;
            let store_handle = store.get_handle()?;

            // external node & transport used to communicate with store
            let external_node = LocalNode::generate();
            let mut external_transport_handle = cluster
                .transport_hub
                .get_transport(external_node.clone(), TransportLayer::Index);
            let external_transport_sink = external_transport_handle.get_sink();
            let external_transport_stream = external_transport_handle.get_stream();
            cluster.runtime.spawn(
                external_transport_handle
                    .map(|_| {
                        info!("Transport handle completed");
                    })
                    .map_err(|err| {
                        error!("Transport handle error: {}", err);
                    }),
            );

            Ok(TestLocalStore {
                cluster,
                schema: schema.clone(),

                store: Some(store),
                store_handle,
                _temp_dir: temp_dir,

                external_node,
                external_transport_sink: Some(external_transport_sink),
                external_transport_stream: Some(external_transport_stream),
            })
        }

        pub fn start_store(&mut self) -> Result<(), failure::Error> {
            let store = self.store.take().unwrap();
            self.cluster.runtime.spawn(
                store
                    .map(|_| {
                        info!("Test store completed");
                    })
                    .map_err(|err| {
                        error!("Test store future failed: {}", err);
                    }),
            );
            self.cluster
                .runtime
                .block_on(self.store_handle.on_start()?)?;
            Ok(())
        }

        pub fn mutate_via_handle(
            &mut self,
            mutation: Mutation,
        ) -> Result<MutationResult, failure::Error> {
            let resp_future = self.store_handle.mutate(mutation);
            self.cluster
                .runtime
                .block_on(resp_future)
                .map_err(|err| err.into())
        }

        pub fn mutate_via_transport(
            &mut self,
            mutation: Mutation,
        ) -> Result<MutationResult, failure::Error> {
            let mutation_frame = mutation.to_mutation_request_frame(&self.schema)?;

            // send message to store
            let follow_id = self.cluster.clocks[0].consistent_time(&self.cluster.nodes[0]);
            let external_cell =
                self.cluster.cells[0].clone_for_local_node(self.external_node.clone());
            let out_message = OutMessage::from_framed_message(
                &external_cell,
                TransportLayer::Index,
                mutation_frame,
            )?
            .with_to_node(self.cluster.nodes[0].node().clone())
            .with_follow_id(follow_id);

            let sink = self.cluster.runtime.block_on(
                self.external_transport_sink
                    .take()
                    .unwrap()
                    .send(out_message),
            )?;
            self.external_transport_sink = Some(sink);

            // wait for response from store
            let (received, stream) = self.cluster.runtime.block_on(
                self.external_transport_stream
                    .take()
                    .unwrap()
                    .into_future()
                    .map_err(|(err, _stream)| {
                        err_msg(format!("Error receiving from stream: {}", err))
                    }),
            )?;
            self.external_transport_stream = Some(stream);

            // read response into mutation response
            let in_msg: InMessage = received.unwrap();
            let resp_frame = in_msg.get_data_as_framed_message()?;
            let response = MutationResult::from_response_frame(&self.schema, resp_frame)?;

            assert_eq!(in_msg.follow_id, Some(follow_id));

            Ok(response)
        }

        pub fn query_via_handle(&mut self, query: Query) -> Result<QueryResult, failure::Error> {
            let resp_future = self.store_handle.query(query);
            self.cluster
                .runtime
                .block_on(resp_future)
                .map_err(|err| err.into())
        }

        pub fn query_via_transport(&mut self, query: Query) -> Result<QueryResult, failure::Error> {
            let query_frame = query.to_query_request_frame(&self.schema)?;

            // send message to store
            let follow_id = self.cluster.clocks[0].consistent_time(&self.cluster.nodes[0]);
            let external_cell =
                self.cluster.cells[0].clone_for_local_node(self.external_node.clone());
            let out_message = OutMessage::from_framed_message(
                &external_cell,
                TransportLayer::Index,
                query_frame,
            )?
            .with_to_node(self.cluster.nodes[0].node().clone())
            .with_follow_id(follow_id);

            let sink = self.cluster.runtime.block_on(
                self.external_transport_sink
                    .take()
                    .unwrap()
                    .send(out_message),
            )?;
            self.external_transport_sink = Some(sink);

            // wait for response from store
            let (received, stream) = self.cluster.runtime.block_on(
                self.external_transport_stream
                    .take()
                    .unwrap()
                    .into_future()
                    .map_err(|(err, _stream)| {
                        err_msg(format!("Error receiving from stream: {}", err))
                    }),
            )?;
            self.external_transport_stream = Some(stream);

            // read response into a results
            let in_msg: InMessage = received.unwrap();
            let resp_frame = in_msg.get_data_as_framed_message()?;
            let results = QueryResult::from_query_frame(&self.schema, resp_frame)?;

            assert_eq!(in_msg.follow_id, Some(follow_id));

            Ok(results)
        }

        pub fn create_put_contact_mutation<E: Into<EntityId>, T: Into<TraitId>, N: Into<String>>(
            &self,
            entity_id: E,
            trait_id: T,
            name: N,
        ) -> Mutation {
            Mutation::PutTrait(PutTraitMutation {
                entity_id: entity_id.into(),
                trt: Trait::new(self.schema.clone(), "contact")
                    .with_id(trait_id.into())
                    .with_value_by_name("name", name.into()),
            })
        }
    }
}
