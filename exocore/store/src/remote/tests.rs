use std::{sync::Arc, time::Duration};

use exocore_core::{
    cell::{CellNodeRole, LocalNode},
    futures::spawn_future,
    tests_utils::{assert_equal_res, async_expect_eventually, expect_eventually},
};
use exocore_protos::generated::exocore_store::{EntityQuery, EntityResults, MutationResult};
use exocore_transport::{testing::MockTransportServiceHandle, ServiceType};
use futures::executor::block_on_stream;
use tokio::sync::Mutex;

use super::*;
use crate::{
    error::Error,
    local::TestStore,
    mutation::{MutationBuilder, MutationRequestLike},
    query::QueryBuilder,
    remote::server::{Server, ServerConfiguration},
    store::Store,
};

#[tokio::test(flavor = "multi_thread")]
async fn mutation_and_query() -> anyhow::Result<()> {
    let test_remote_store = Arc::new(Mutex::new(TestRemoteStore::new().await?));
    {
        let mut test_remote_store = test_remote_store.lock().await;
        test_remote_store.start_server().await?;
        test_remote_store.start_client().await?;

        let mutation = test_remote_store
            .local_store
            .create_put_contact_mutation("entity1", "trait1", "hello");
        test_remote_store.send_and_await_mutation(mutation).await?;
    }

    {
        async_expect_eventually(|| async {
            let mut test_remote_store = test_remote_store.lock().await;
            let query = QueryBuilder::matches("hello").build();
            let results = test_remote_store.send_and_await_query(query).await.unwrap();
            assert_equal_res(results.entities.len(), 1)
        })
        .await;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn mutation_return_entities() -> anyhow::Result<()> {
    let test_remote_store = Arc::new(Mutex::new(TestRemoteStore::new().await?));
    let mut test_remote_store = test_remote_store.lock().await;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello")
        .return_entities();
    let mutation_resp = test_remote_store.send_and_await_mutation(mutation).await?;

    assert_eq!(mutation_resp.entities.len(), 1);
    let entity = &mutation_resp.entities[0];
    assert_eq!("entity1", &entity.id);

    assert_eq!(entity.traits.len(), 1);
    let trt = &entity.traits[0];
    assert_eq!("trait1", &trt.id);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn mutation_error_propagation() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new().await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let mutation = MutationBuilder::new().fail_mutation("entity1");
    let result = test_remote_store.send_and_await_mutation(mutation);
    assert!(result.await.is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query_error_propagation() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new().await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello");
    test_remote_store.send_and_await_mutation(mutation).await?;

    let query = QueryBuilder::test(false).build();
    let result = test_remote_store.send_and_await_query(query);
    assert!(result.await.is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query_timeout() -> anyhow::Result<()> {
    let client_config = ClientConfiguration {
        query_timeout: Duration::from_millis(500),
        ..ClientConfiguration::default()
    };

    let mut test_remote_store =
        TestRemoteStore::new_with_configuration(Default::default(), client_config).await?;

    // only start remote, so local won't answer and it should timeout
    test_remote_store.start_client().await?;

    let query = QueryBuilder::matches("hello").build();
    let result = test_remote_store.send_and_await_query(query);
    assert!(result.await.is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn mutation_timeout() -> anyhow::Result<()> {
    let client_config = ClientConfiguration {
        mutation_timeout: Duration::from_millis(500),
        ..ClientConfiguration::default()
    };

    let mut test_remote_store =
        TestRemoteStore::new_with_configuration(Default::default(), client_config).await?;

    // only start remote, so local won't answer and it should timeout
    test_remote_store.start_client().await?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello");
    let result = test_remote_store.send_and_await_mutation(mutation);
    assert!(result.await.is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn watched_query() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new().await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello")
        .return_entities();
    test_remote_store.send_and_await_mutation(mutation).await?;

    let query = QueryBuilder::matches("hello").build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query)?);

    let results = stream.next().unwrap().unwrap();
    assert_eq!(results.entities.len(), 1);

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity2", "trait2", "hello");
    test_remote_store.send_and_await_mutation(mutation).await?;

    let results = stream.next().unwrap().unwrap();
    assert_eq!(results.entities.len(), 2);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn watched_query_error_propagation() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new().await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let query = QueryBuilder::test(false).build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query)?);

    let results = stream.next().unwrap();
    assert!(results.is_err());

    // stream should have been closed
    let results = stream.next();
    assert!(results.is_none());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn watched_query_timeout() -> anyhow::Result<()> {
    let server_config = ServerConfiguration {
        management_timer_interval: Duration::from_millis(100),
        watched_queries_register_timeout: Duration::from_millis(2000),
    };

    // client will re-register itself at higher interval then expected on server,
    // which will result in timing out eventually
    let client_config = ClientConfiguration {
        watched_register_interval: Duration::from_millis(2100),
        watched_re_register_remote_dropped: false,
        ..ClientConfiguration::default()
    };

    let mut test_remote_store =
        TestRemoteStore::new_with_configuration(server_config, client_config).await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let mutation = test_remote_store
        .local_store
        .create_put_contact_mutation("entity1", "trait1", "hello")
        .return_entities();
    test_remote_store.send_and_await_mutation(mutation).await?;

    let query = QueryBuilder::matches("hello").build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query)?);

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

#[tokio::test(flavor = "multi_thread")]
async fn watched_query_re_register_remote_dropped() -> anyhow::Result<()> {
    let server_config = ServerConfiguration {
        management_timer_interval: Duration::from_millis(100),
        ..Default::default()
    };
    let client_config = ClientConfiguration::default();

    let mut test_remote_store =
        TestRemoteStore::new_with_configuration(server_config, client_config).await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let query = QueryBuilder::matches("hello").build();
    let _stream = block_on_stream(test_remote_store.client_handle.watched_query(query)?);

    // watch for query to be registered, then drop it
    expect_eventually(|| {
        let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
        if watched_queries.is_empty() {
            false
        } else {
            test_remote_store
                .local_store
                .store_handle
                .clear_watched_queries();
            true
        }
    });

    // client should detect its been dropped and register again
    expect_eventually(|| {
        let watched_queries = test_remote_store.local_store.store_handle.watched_queries();
        watched_queries.is_empty()
    });

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn watched_drop_cancel() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new().await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

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

#[tokio::test(flavor = "multi_thread")]
async fn client_drop_stops_watched_stream() -> anyhow::Result<()> {
    let mut test_remote_store = TestRemoteStore::new().await?;
    test_remote_store.start_server().await?;
    test_remote_store.start_client().await?;

    let query = QueryBuilder::matches("hello").build();
    let mut stream = block_on_stream(test_remote_store.client_handle.watched_query(query)?);

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
    client: Option<Client<MockTransportServiceHandle>>,
    client_handle: ClientHandle,
}

impl TestRemoteStore {
    async fn new() -> Result<TestRemoteStore, anyhow::Error> {
        let client_config = Default::default();
        let server_config = Default::default();
        Self::new_with_configuration(server_config, client_config).await
    }

    async fn new_with_configuration(
        server_config: ServerConfiguration,
        client_config: ClientConfiguration,
    ) -> Result<TestRemoteStore, anyhow::Error> {
        let mut local_store = TestStore::new().await?;

        local_store.cluster.add_node_role(0, CellNodeRole::Store);

        let local_node = LocalNode::generate();
        let store_client = Client::new(
            client_config,
            local_store.cluster.cells[0].cell().clone(),
            local_store.cluster.clocks[0].clone(),
            local_store
                .cluster
                .transport_hub
                .get_transport(local_node, ServiceType::Store),
        )?;
        let client_handle = store_client.get_handle();

        Ok(TestRemoteStore {
            local_store,
            server_config,
            client: Some(store_client),
            client_handle,
        })
    }

    async fn start_server(&mut self) -> anyhow::Result<()> {
        let store_handle = self.local_store.store.as_ref().unwrap().get_handle();

        self.local_store.start_store().await?;

        let cell = self.local_store.cluster.cells[0].cell().clone();
        let transport = self.local_store.cluster.transport_hub.get_transport(
            self.local_store.cluster.nodes[0].clone(),
            ServiceType::Store,
        );

        let server = Server::new(self.server_config, cell, store_handle, transport)?;
        spawn_future(async move {
            let res = server.run().await;
            info!("Server is done: {:?}", res);
        });

        Ok(())
    }

    async fn start_client(&mut self) -> anyhow::Result<()> {
        let client = self.client.take().unwrap();
        spawn_future(async move {
            let res = client.run().await;
            info!("Client is done: {:?}", res);
        });

        self.client_handle.on_start().await;

        // notify that server is online for client to use id
        let node_id = self.local_store.cluster.cells[0].cell().local_node().id();
        self.local_store
            .cluster
            .transport_hub
            .notify_node_connection_status(
                node_id,
                exocore_transport::transport::ConnectionStatus::Connected,
            );

        Ok(())
    }

    async fn send_and_await_mutation<M: Into<MutationRequestLike> + Send>(
        &mut self,
        request: M,
    ) -> Result<MutationResult, Error> {
        self.client_handle.mutate(request).await
    }

    async fn send_and_await_query(&mut self, query: EntityQuery) -> Result<EntityResults, Error> {
        self.client_handle.query(query).await
    }
}
