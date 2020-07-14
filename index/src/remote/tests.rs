use std::time::Duration;

use futures::executor::block_on_stream;

use exocore_core::cell::{CellNodeRole, LocalNode};
use exocore_core::protos::generated::exocore_index::{EntityQuery, EntityResults, MutationResult};
use exocore_core::tests_utils::expect_eventually;
use exocore_transport::mock::MockTransportHandle;
use exocore_transport::TransportLayer;

use crate::error::Error;
use crate::local::TestStore;
use crate::mutation::{MutationBuilder, MutationRequestLike};
use crate::query::QueryBuilder;
use crate::remote::server::{Server, ServerConfiguration};

use super::*;

#[test]
fn mutation_and_query() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello");
    test_remote_store.send_and_await_mutation(mutation)?;

    expect_eventually(|| {
        let query = QueryBuilder::matches("hello").build();
        let results = test_remote_store.send_and_await_query(query).unwrap();
        results.entities.len() == 1
    });

    Ok(())
}

#[test]
fn mutation_error_propagation() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let mutation = MutationBuilder::new().fail_mutation("entity1");
    let result = test_remote_store.send_and_await_mutation(mutation);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn query_error_propagation() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello");
    test_remote_store.send_and_await_mutation(mutation)?;

    let query = QueryBuilder::failed().build();
    let result = test_remote_store.send_and_await_query(query);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn query_timeout() -> anyhow::Result<()> {
    let client_config = ClientConfiguration {
        query_timeout: Duration::from_millis(500),
        ..ClientConfiguration::default()
    };

    let mut test_remote_store =
        TestRemoteStore::new_with_configuration(Default::default(), client_config)?;

    // only start remote, so local won't answer and it should timeout
    test_remote_store.start_client()?;

    let query = QueryBuilder::matches("hello").build();
    let result = test_remote_store.send_and_await_query(query);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn mutation_timeout() -> anyhow::Result<()> {
    let client_config = ClientConfiguration {
        mutation_timeout: Duration::from_millis(500),
        ..ClientConfiguration::default()
    };

    let mut test_remote_store =
        TestRemoteStore::new_with_configuration(Default::default(), client_config)?;

    // only start remote, so local won't answer and it should timeout
    test_remote_store.start_client()?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello");
    let result = test_remote_store.send_and_await_mutation(mutation);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn watched_query() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello");
    test_remote_store.send_and_await_mutation(mutation)?;

    let query = QueryBuilder::matches("hello").build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query));

    let results = stream.next().unwrap().unwrap();
    assert_eq!(results.entities.len(), 1);

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity2", "trait2", "hello");
    test_remote_store.send_and_await_mutation(mutation)?;

    let results = stream.next().unwrap().unwrap();
    assert_eq!(results.entities.len(), 2);

    Ok(())
}

#[test]
fn watched_query_error_propagation() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let query = QueryBuilder::failed().build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query));

    let results = stream.next().unwrap();
    assert!(results.is_err());

    // stream should have been closed
    let results = stream.next();
    assert!(results.is_none());

    Ok(())
}

#[test]
fn watched_query_timeout() -> anyhow::Result<()> {
    let server_config = ServerConfiguration {
        management_timer_interval: Duration::from_millis(100),
        watched_queries_register_timeout: Duration::from_millis(1000),
    };

    // client will re-register itself at higher interval then expected on server,
    // which will result in timing out eventually
    let client_config = ClientConfiguration {
        watched_queries_register_interval: Duration::from_millis(1100),
        ..ClientConfiguration::default()
    };

    let mut test_remote_store =
        TestRemoteStore::new_with_configuration(server_config, client_config)?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello");
    test_remote_store.send_and_await_mutation(mutation)?;

    let query = QueryBuilder::matches("hello").build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query));

    let results = stream.next().unwrap().unwrap();
    assert_eq!(results.entities.len(), 1);

    let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
    assert_eq!(watched_queries.len(), 1);

    // wait for watched query to be removed on server because of timeout
    expect_eventually(|| {
        let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
        watched_queries.is_empty()
    });

    // stream should be sent an error and then closed
    let res = stream.next().unwrap();
    assert!(res.is_err());
    let res = stream.next();
    assert!(res.is_none());

    Ok(())
}

#[test]
fn watched_drop_unregisters() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let query = QueryBuilder::matches("hello").build();
    let stream = test_remote_store.client_handle.watched_query(query);

    // wait for watched query to registered
    expect_eventually(|| {
        let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
        !watched_queries.is_empty()
    });

    // drop stream
    drop(stream);

    // query should be unwatched
    expect_eventually(|| {
        let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
        watched_queries.is_empty()
    });

    Ok(())
}

#[test]
fn watched_cancel() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let query = QueryBuilder::matches("hello").build();
    let stream = test_remote_store.client_handle.watched_query(query);
    let query_id = stream.query_id();

    // wait for watched query to registered
    expect_eventually(|| {
        let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
        !watched_queries.is_empty()
    });

    test_remote_store.client_handle.cancel_query(query_id)?;

    // query should be unwatched
    expect_eventually(|| {
        let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
        watched_queries.is_empty()
    });

    Ok(())
}

#[test]
fn client_drop_stops_watched_stream() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new()?;
    test_remote_store.start_server()?;
    test_remote_store.start_client()?;

    let query = QueryBuilder::matches("hello").build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query));

    let results = stream.next().unwrap();
    assert!(results.is_ok());

    // drop remote store client
    let TestRemoteStore { client_handle, .. } = test_remote_store;
    drop(client_handle);

    // stream should have been closed because it got dropped
    let res = stream.next();
    assert!(res.is_none());

    Ok(())
}

struct TestRemoteStore {
    local_store: TestStore,
    server_config: ServerConfiguration,
    client: Option<Client<MockTransportHandle>>,
    client_handle: ClientHandle,
}

impl TestRemoteStore {
    fn new() -> Result<TestRemoteStore, anyhow::Error> {
        let client_config = Default::default();
        let server_config = Default::default();
        Self::new_with_configuration(server_config, client_config)
    }

    fn new_with_configuration(
        server_config: ServerConfiguration,
        client_config: ClientConfiguration,
    ) -> Result<TestRemoteStore, anyhow::Error> {
        let mut local_store = TestStore::new()?;

        local_store
            .cluster
            .add_node_role(0, CellNodeRole::IndexStore);

        let local_node = LocalNode::generate();
        let store_client = Client::new(
            client_config,
            local_store.cluster.cells[0].cell().clone(),
            local_store.cluster.clocks[0].clone(),
            local_store
                .cluster
                .transport_hub
                .get_transport(local_node, TransportLayer::Index),
        )?;
        let client_handle = store_client.get_handle();

        Ok(TestRemoteStore {
            local_store,
            server_config,
            client: Some(store_client),
            client_handle,
        })
    }

    fn start_server(&mut self) -> anyhow::Result<()> {
        let store_handle = self.local_store.store.as_ref().unwrap().get_handle();

        self.local_store.start_store()?;

        let cell = self.local_store.cluster.cells[0].cell().clone();
        let transport = self.local_store.cluster.transport_hub.get_transport(
            self.local_store.cluster.nodes[0].clone(),
            TransportLayer::Index,
        );

        let server = Server::new(self.server_config, cell, store_handle, transport)?;
        self.local_store.cluster.runtime.spawn(async move {
            let res = server.run().await;
            info!("Server is done: {:?}", res);
        });

        Ok(())
    }

    fn start_client(&mut self) -> anyhow::Result<()> {
        let client = self.client.take().unwrap();
        self.local_store.cluster.runtime.spawn(async move {
            let res = client.run().await;
            info!("Client is done: {:?}", res);
        });

        futures::executor::block_on(self.client_handle.on_start());

        Ok(())
    }

    fn send_and_await_mutation<M: Into<MutationRequestLike>>(
        &mut self,
        request: M,
    ) -> Result<MutationResult, Error> {
        futures::executor::block_on(self.client_handle.mutate(request))
    }

    fn send_and_await_query(&mut self, query: EntityQuery) -> Result<EntityResults, Error> {
        futures::executor::block_on(self.client_handle.query(query))
    }
}
