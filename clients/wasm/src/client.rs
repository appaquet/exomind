use log::Level;
use wasm_bindgen::prelude::*;

use exocore_common::cell::Cell;
use exocore_common::crypto::keys::PublicKey;
use exocore_common::node::{LocalNode, Node};
use exocore_common::time::Clock;
use exocore_common::utils::futures::spawn_future_non_send;
use exocore_index::store::remote::{Client, ClientConfiguration, ClientHandle};
use exocore_schema::schema::Schema;
use exocore_transport::{InEvent, TransportHandle, TransportLayer};

use crate::ws::BrowserTransportClient;
use exocore_transport::transport::ConnectionStatus;
use futures::compat::Stream01CompatExt;
use futures::StreamExt;
use std::sync::{Arc, Mutex};

#[wasm_bindgen]
pub struct ExocoreClient {
    _transport: BrowserTransportClient,
    store_handle: Arc<ClientHandle>,
    schema: Arc<Schema>,
    _inner: Arc<Mutex<Inner>>,
}

struct Inner {
    status_change_callback: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl ExocoreClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        url: &str,
        status_change_callback: Option<js_sys::Function>,
    ) -> Result<ExocoreClient, JsValue> {
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
        let remote_store = Client::new(
            ClientConfiguration::default(),
            cell.clone(),
            clock,
            schema.clone(),
            index_handle,
            remote_node.clone(),
        )
        .expect("Couldn't create index");

        let store_handle = Arc::new(
            remote_store
                .get_handle()
                .expect("Couldn't get store handle"),
        );

        spawn_future_non_send(async move {
            if let Err(err) = remote_store.run().await {
                error!("Error starting remote store: {}", err);
            }

            Ok(())
        });

        let store_handle1 = store_handle.clone();
        spawn_future_non_send(async move {
            let start_future = store_handle1.on_start();
            match start_future.await {
                Ok(_) => info!("Remote store started"),
                Err(err) => error!("Error starting remote store: {}", err),
            }

            Ok(())
        });

        let inner = Arc::new(Mutex::new(Inner {
            status_change_callback,
        }));

        let mut client_transport_handle =
            transport.get_handle(cell.clone(), TransportLayer::Client);
        let inner_clone = inner.clone();
        spawn_future_non_send(async move {
            let mut stream = client_transport_handle.get_stream().compat();

            while let Some(event) = stream.next().await {
                let event = if let Ok(event) = event {
                    event
                } else {
                    return Ok(());
                };

                if let InEvent::NodeStatus(_, status) = event {
                    let str_status = match status {
                        ConnectionStatus::Connecting => "connecting",
                        ConnectionStatus::Connected => "connected",
                        ConnectionStatus::Disconnected => "disconnected",
                    };

                    let inner = inner_clone.lock().unwrap();
                    if let Some(func) = &inner.status_change_callback {
                        func.call1(&JsValue::null(), &JsValue::from_str(str_status))
                            .unwrap();
                    }
                }
            }

            Ok(())
        });

        transport.start();

        Ok(ExocoreClient {
            _transport: transport,
            store_handle,
            schema,
            _inner: inner,
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
