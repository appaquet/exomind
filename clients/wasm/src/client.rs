use futures::{Future, Stream};
use log::Level;
use wasm_bindgen::prelude::*;

use exocore_common::cell::Cell;
use exocore_common::crypto::keys::PublicKey;
use exocore_common::node::{LocalNode, Node};
use exocore_common::time::Clock;
use exocore_index::store::remote::{RemoteStore, StoreConfiguration, StoreHandle};
use exocore_schema::schema::Schema;
use exocore_transport::{InEvent, TransportHandle, TransportLayer};

use crate::js::js_future_spawner;
use crate::ws::BrowserTransportClient;
use exocore_transport::transport::ConnectionStatus;
use std::sync::Arc;

#[wasm_bindgen]
pub struct ExocoreClient {
    _transport: BrowserTransportClient,
    store_handle: Arc<StoreHandle>,
    schema: Arc<Schema>,
}

#[wasm_bindgen]
extern "C" {
    pub fn exocore_client_status(s: &str);
}

#[wasm_bindgen]
impl ExocoreClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<ExocoreClient, JsValue> {
        console_log::init_with_level(Level::Debug).expect("Couldn't init level");

        // TODO: To be cleaned up when cell management will be ironed out: https://github.com/appaquet/exocore/issues/80
        let local_node = LocalNode::generate();
        let cell_pk =
            PublicKey::decode_base58_string("pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ")
                .expect("Couldn't decode cell publickey");
        let cell = Cell::new(cell_pk, local_node.clone());
        let clock = Clock::new();
        let schema = exocore_schema::test_schema::create();

        let remote_node_pk =
            PublicKey::decode_base58_string("pe5ZG43uAcfLxYSGaQgj1w8hQT4GBchEVg5mS2b1EfXcMb")
                .expect("Couldn't decode cell publickey");
        let remote_node = Node::new_from_public_key(remote_node_pk);

        let mut transport = BrowserTransportClient::new(url, remote_node.clone());
        let index_handle = transport.get_handle(cell.clone(), TransportLayer::Index);
        let remote_store = RemoteStore::new(
            StoreConfiguration::default(),
            cell.clone(),
            clock,
            schema.clone(),
            index_handle,
            remote_node.clone(),
            Box::new(js_future_spawner),
        )
        .expect("Couldn't create index");

        let store_handle = remote_store
            .get_handle()
            .expect("Couldn't get store handle");
        js_future_spawner(Box::new(remote_store.map_err(|err| {
            error!("Error starting remote store: {}", err);
        })));

        js_future_spawner(Box::new(
            store_handle
                .on_start()
                .unwrap()
                .and_then(|_| {
                    info!("Remote store started");
                    Ok(())
                })
                .map_err(|_err| ()),
        ));

        let mut client_transport_handle =
            transport.get_handle(cell.clone(), TransportLayer::Client);
        js_future_spawner(Box::new(
            client_transport_handle
                .get_stream()
                .for_each(move |event| {
                    if let InEvent::NodeStatus(_, status) = event {
                        let str_status = match status {
                            ConnectionStatus::Connecting => "connecting",
                            ConnectionStatus::Connected => "connected",
                            ConnectionStatus::Disconnected => "disconnected",
                        };

                        exocore_client_status(str_status);
                    }

                    Ok(())
                })
                .map_err(|_| ()),
        ));

        transport.start();

        Ok(ExocoreClient {
            _transport: transport,
            store_handle: Arc::new(store_handle),
            schema,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn query(&self) -> crate::query::QueryBuilder {
        crate::query::QueryBuilder::new(self.schema.clone(), self.store_handle.clone())
    }

    #[wasm_bindgen(getter)]
    pub fn mutate(&self) -> crate::mutation::MutationBuilder {
        crate::mutation::MutationBuilder::new(self.schema.clone(), self.store_handle.clone())
    }
}

impl Drop for ExocoreClient {
    fn drop(&mut self) {
        info!("Got dropped");
    }
}
