#![deny(bare_trait_objects)]

#[macro_use]
extern crate log;

use exocore_common::cell::Cell;
use exocore_common::crypto::keys::PublicKey;
use exocore_common::node::{LocalNode, Node};
use exocore_common::time::Clock;
use exocore_index::domain::schema::Schema;
use exocore_index::domain::serialization::with_schema;
use exocore_index::mutation::Mutation;
use exocore_index::query::Query;
use exocore_index::store::remote::{RemoteStore, StoreConfiguration, StoreHandle};
use exocore_index::store::AsyncStore;
use exocore_transport::TransportLayer;
use failure::err_msg;
use futures::Future;
use log::Level;
use std::fmt::Display;
use std::sync::Arc;
use stdweb;
use wasm_bindgen::prelude::*;

mod ws;
use ws::BrowserTransportClient;

#[wasm_bindgen]
pub struct ExocoreClient {
    _transport: BrowserTransportClient,
    store_handle: StoreHandle,
    schema: Arc<Schema>,
}

#[wasm_bindgen]
impl ExocoreClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<ExocoreClient, JsValue> {
        stdweb::initialize();

        console_log::init_with_level(Level::Debug).expect("Couldn't init level");

        // TODO: To be cleaned up when cell management will be ironed out: https://github.com/appaquet/exocore/issues/80
        let local_node = LocalNode::generate();
        let cell_pk =
            PublicKey::decode_base58_string("pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ")
                .expect("Couldn't decode cell publickey");
        let cell = Cell::new(cell_pk, local_node.clone());
        let clock = Clock::new();
        let schema = create_test_schema();

        let remote_node_pk =
            PublicKey::decode_base58_string("pe5ZG43uAcfLxYSGaQgj1w8hQT4GBchEVg5mS2b1EfXcMb")
                .expect("Couldn't decode cell publickey");
        let remote_node = Node::new_from_public_key(remote_node_pk);

        let mut transport = BrowserTransportClient::new(url, remote_node.clone());
        let index_handle = transport.get_handle(cell.clone(), TransportLayer::Index);
        let remote_store = RemoteStore::new(
            StoreConfiguration::default(),
            cell,
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

        transport.start();

        Ok(ExocoreClient {
            _transport: transport,
            store_handle,
            schema,
        })
    }

    #[wasm_bindgen]
    pub fn mutate(&mut self, query_json: &JsValue) -> js_sys::Promise {
        let mutation = with_schema(&self.schema, || query_json.into_serde::<Mutation>());
        let mutation = match mutation {
            Ok(mutation) => mutation,
            Err(err) => {
                return wasm_bindgen_futures::future_to_promise(futures::failed(into_js_error(
                    err_msg(format!("Couldn't parse mutation: {}", err)),
                )));
            }
        };

        let schema = self.schema.clone();
        let fut_result = self
            .store_handle
            .mutate(mutation)
            .map(move |res| {
                with_schema(&schema, || JsValue::from_serde(&res)).unwrap_or_else(into_js_error)
            })
            .map_err(into_js_error);
        wasm_bindgen_futures::future_to_promise(fut_result)
    }

    #[wasm_bindgen]
    pub fn query(&mut self, query_json: &JsValue) -> js_sys::Promise {
        let query = with_schema(&self.schema, || query_json.into_serde::<Query>());
        let query = match query {
            Ok(query) => query,
            Err(err) => {
                return wasm_bindgen_futures::future_to_promise(futures::failed(into_js_error(
                    err_msg(format!("Couldn't parse query: {}", err)),
                )));
            }
        };

        let schema = self.schema.clone();
        let fut_result = self
            .store_handle
            .query(query)
            .map(move |res| {
                with_schema(&schema, || JsValue::from_serde(&res)).unwrap_or_else(into_js_error)
            })
            .map_err(into_js_error);
        wasm_bindgen_futures::future_to_promise(fut_result)
    }
}

impl Drop for ExocoreClient {
    fn drop(&mut self) {
        info!("Got dropped");
    }
}

fn into_js_error<E: Display>(err: E) -> JsValue {
    let js_error = js_sys::Error::new(&format!("Error executing query: {}", err));
    JsValue::from(js_error)
}

// TODO: To be moved https://github.com/appaquet/exocore/issues/123
fn js_future_spawner(future: Box<dyn Future<Item = (), Error = ()> + Send>) {
    wasm_bindgen_futures::spawn_local(future);
}

// TODO: To be cleaned up in https://github.com/appaquet/exocore/issues/104
pub fn create_test_schema() -> Arc<Schema> {
    Arc::new(
        Schema::parse(
            r#"
        namespaces:
            - name: exocore
              traits:
                - id: 0
                  name: contact
                  fields:
                    - id: 0
                      name: name
                      type: string
                      indexed: true
                    - id: 1
                      name: email
                      type: string
                      indexed: true
                - id: 1
                  name: email
                  fields:
                    - id: 0
                      name: subject
                      type: string
                      indexed: true
                    - id: 1
                      name: body
                      type: string
                      indexed: true
        "#,
        )
        .unwrap(),
    )
}
