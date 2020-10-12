use std::sync::{Arc, Mutex, Once};

use futures::StreamExt;
use prost::Message;
use wasm_bindgen::prelude::*;

use exocore_core::cell::Cell;
use exocore_core::futures::spawn_future_non_send;
use exocore_core::protos::generated::exocore_store::EntityQuery;
use exocore_core::time::Clock;
use exocore_store::remote::{Client, ClientConfiguration, ClientHandle};
use exocore_transport::transport::ConnectionStatus;
use exocore_transport::{InEvent, Libp2pTransport, ServiceType, TransportServiceHandle};

use crate::js::into_js_error;
use crate::watched_query::WatchedQuery;
use exocore_core::protos::{prost::ProstMessageExt, store::MutationRequest};
use exocore_transport::p2p::Libp2pTransportConfig;

static INIT: Once = Once::new();

#[wasm_bindgen]
pub struct ExocoreClient {
    store_handle: Arc<ClientHandle>,
    _inner: Arc<Mutex<Inner>>,
}

struct Inner {
    status_change_callback: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl ExocoreClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        node_config_bytes: js_sys::Uint8Array,
        node_config_format: JsValue,
        status_change_callback: Option<js_sys::Function>,
    ) -> Result<ExocoreClient, JsValue> {
        INIT.call_once(|| {
            wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        });

        let config_bytes = Self::js_bytes_to_vec(node_config_bytes);
        let node_config_format = node_config_format.as_string();
        let node_config_format = node_config_format.as_deref();

        let config = match node_config_format {
            Some("json") => exocore_core::cell::node_config_from_json(config_bytes.as_slice()),
            Some("yaml") => exocore_core::cell::node_config_from_yaml(config_bytes.as_slice()),
            other => panic!("Invalid config format: {:?}", other),
        }
        .expect("Couldn't decode config");

        let (either_cells, local_node) =
            Cell::new_from_local_node_config(config).expect("Couldn't create cell");

        let either_cell = either_cells
            .first()
            .cloned()
            .expect("Couldn't find any cell");

        let cell = either_cell.cell().clone();

        let transport_config = Libp2pTransportConfig::default();
        let mut transport = Libp2pTransport::new(local_node, transport_config);

        let clock = Clock::new();

        let store_handle = transport
            .get_handle(cell.clone(), ServiceType::Store)
            .unwrap();
        let remote_store = Client::new(
            ClientConfiguration::default(),
            cell.clone(),
            clock,
            store_handle,
        )
        .expect("Couldn't create store");

        let store_handle = Arc::new(remote_store.get_handle());

        spawn_future_non_send(async move {
            if let Err(err) = remote_store.run().await {
                error!("Error running remote store: {}", err);
            }
            Ok(())
        });

        let store_handle1 = store_handle.clone();
        spawn_future_non_send(async move {
            store_handle1.on_start().await;
            Ok(())
        });

        let inner = Arc::new(Mutex::new(Inner {
            status_change_callback,
        }));

        let mut client_transport_handle = transport.get_handle(cell, ServiceType::Client).unwrap();
        let inner_clone = inner.clone();
        spawn_future_non_send(async move {
            let mut stream = client_transport_handle.get_stream();

            while let Some(event) = stream.next().await {
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

        spawn_future_non_send(async move {
            info!("Starting libp2p transport...");
            match transport.run().await {
                Ok(_) => info!("Libp2p transport done"),
                Err(err) => error!("Error in libp2p transport: {}", err),
            }

            Ok(())
        });

        Ok(ExocoreClient {
            store_handle,
            _inner: inner,
        })
    }

    #[wasm_bindgen]
    pub fn mutate(&self, mutation_proto_bytes: js_sys::Uint8Array) -> js_sys::Promise {
        let bytes = Self::js_bytes_to_vec(mutation_proto_bytes);
        let entity_mutation =
            MutationRequest::decode(bytes.as_ref()).expect("Couldn't encode query");

        let store_handle = self.store_handle.clone();
        let fut_results = async move {
            let result = store_handle
                .mutate(entity_mutation)
                .await
                .map_err(into_js_error)?;

            let results_data = result.encode_to_vec().map_err(into_js_error)?;
            Ok(js_sys::Uint8Array::from(results_data.as_ref()).into())
        };

        wasm_bindgen_futures::future_to_promise(fut_results)
    }

    #[wasm_bindgen]
    pub fn query(&self, query_proto_bytes: js_sys::Uint8Array) -> js_sys::Promise {
        let bytes = Self::js_bytes_to_vec(query_proto_bytes);
        let entity_query = EntityQuery::decode(bytes.as_ref()).expect("Couldn't encode query");

        let store_handle = self.store_handle.clone();
        let fut_results = async move {
            let result = store_handle
                .query(entity_query)
                .await
                .map_err(into_js_error)?;

            let results_data = result.encode_to_vec().map_err(into_js_error)?;
            Ok(js_sys::Uint8Array::from(results_data.as_ref()).into())
        };

        wasm_bindgen_futures::future_to_promise(fut_results)
    }

    #[wasm_bindgen]
    pub fn watched_query(&self, query_proto_bytes: js_sys::Uint8Array) -> WatchedQuery {
        let bytes = Self::js_bytes_to_vec(query_proto_bytes);
        let entity_query = EntityQuery::decode(bytes.as_ref()).expect("Couldn't encode query");

        WatchedQuery::new(self.store_handle.clone(), entity_query)
    }

    #[wasm_bindgen]
    pub fn store_http_endpoints(&self) -> js_sys::Array {
        let store_node_urls = self
            .store_handle
            .store_node()
            .map(|node| node.http_addresses())
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(|url| JsValue::from(url.to_string()));

        store_node_urls.collect()
    }

    fn js_bytes_to_vec(js_bytes: js_sys::Uint8Array) -> Vec<u8> {
        let mut bytes = vec![0u8; js_bytes.length() as usize];
        js_bytes.copy_to(&mut bytes);
        bytes
    }
}

impl Drop for ExocoreClient {
    fn drop(&mut self) {
        info!("Got dropped");
    }
}
