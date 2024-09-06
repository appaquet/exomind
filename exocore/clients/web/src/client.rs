use std::{rc::Rc, sync::Mutex, time::Duration};

use exocore_core::{cell::Cell, futures::spawn_future_non_send, time::Clock};
use exocore_protos::{
    generated::exocore_store::EntityQuery, prost::Message, store::MutationRequest,
};
use exocore_store::{
    remote::{Client, ClientConfiguration, ClientHandle},
    store::Store,
};
use exocore_transport::{
    p2p::Libp2pTransportConfig, transport::ConnectionStatus, InEvent, Libp2pTransport, ServiceType,
    TransportServiceHandle,
};
use futures::StreamExt;
use wasm_bindgen::prelude::*;

use crate::{js::into_js_error, node::LocalNode, watched_query::WatchedQuery};

#[wasm_bindgen]
pub struct ExocoreClient {
    clock: Clock,
    cell: Cell,
    store_handle: Rc<ClientHandle>,
    _inner: Rc<Mutex<Inner>>,
}

struct Inner {
    status_change_callback: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl ExocoreClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        node: &LocalNode,
        status_change_callback: Option<js_sys::Function>,
    ) -> Result<ExocoreClient, JsValue> {
        let local_node = node.node.clone();
        let either_cells = Cell::from_local_node(local_node.clone()).expect("Couldn't create cell");

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
            clock.clone(),
            store_handle,
        )
        .expect("Couldn't create store");

        let store_handle = Rc::new(remote_store.get_handle());

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

        let inner = Rc::new(Mutex::new(Inner {
            status_change_callback,
        }));

        let mut client_transport_handle = transport
            .get_handle(cell.clone(), ServiceType::Client)
            .unwrap();
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
                        if let Err(err) =
                            func.call1(&JsValue::null(), &JsValue::from_str(str_status))
                        {
                            error!("Error calling status report callback: {:?}", err);
                        }
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
            clock,
            cell,
            store_handle,
            _inner: inner,
        })
    }

    #[wasm_bindgen]
    pub fn cell_generate_auth_token(&self, expiration_days: f64) -> Result<String, JsValue> {
        let expiration = if expiration_days > 0.0 {
            let now = self.clock.consistent_time(self.cell.local_node().node());
            Some(now + Duration::from_secs(expiration_days as u64 * 86400))
        } else {
            None
        };

        let auth_token =
            exocore_core::sec::auth_token::AuthToken::new(&self.cell, &self.clock, expiration)
                .map_err(|err| into_js_error("generating auth token", err))?;

        let auth_token_bs58 = auth_token.encode_base58_string();
        Ok(auth_token_bs58)
    }

    #[wasm_bindgen]
    pub fn store_mutate(&self, mutation_proto_bytes: js_sys::Uint8Array) -> js_sys::Promise {
        let bytes = js_bytes_to_vec(mutation_proto_bytes);
        let entity_mutation =
            MutationRequest::decode(bytes.as_ref()).expect("Couldn't encode query");

        let store_handle = self.store_handle.clone();
        let fut_results = async move {
            let result = store_handle
                .mutate(entity_mutation)
                .await
                .map_err(|err| into_js_error("mutating", err))?;

            let results_data = result.encode_to_vec();
            Ok(js_sys::Uint8Array::from(results_data.as_ref()).into())
        };

        wasm_bindgen_futures::future_to_promise(fut_results)
    }

    #[wasm_bindgen]
    pub fn store_query(&self, query_proto_bytes: js_sys::Uint8Array) -> js_sys::Promise {
        let bytes = js_bytes_to_vec(query_proto_bytes);
        let entity_query = EntityQuery::decode(bytes.as_ref()).expect("Couldn't encode query");

        let store_handle = self.store_handle.clone();
        let fut_results = async move {
            let result = store_handle
                .query(entity_query)
                .await
                .map_err(|err| into_js_error("querying", err))?;

            let results_data = result.encode_to_vec();
            Ok(js_sys::Uint8Array::from(results_data.as_ref()).into())
        };

        wasm_bindgen_futures::future_to_promise(fut_results)
    }

    #[wasm_bindgen]
    pub fn store_watched_query(&self, query_proto_bytes: js_sys::Uint8Array) -> WatchedQuery {
        let bytes = js_bytes_to_vec(query_proto_bytes);
        let entity_query = EntityQuery::decode(bytes.as_ref()).expect("Couldn't encode query");

        WatchedQuery::new(self.store_handle.clone(), entity_query)
    }

    #[wasm_bindgen]
    pub fn store_http_endpoints(&self) -> js_sys::Array {
        let store_node_urls = self
            .store_handle
            .store_node()
            .map(|node| node.http_addresses())
            .unwrap_or_default()
            .into_iter()
            .map(|url| JsValue::from(url.to_string()));

        store_node_urls.collect()
    }
}

impl Drop for ExocoreClient {
    fn drop(&mut self) {
        info!("Got dropped");
    }
}

fn js_bytes_to_vec(js_bytes: js_sys::Uint8Array) -> Vec<u8> {
    let mut bytes = vec![0u8; js_bytes.length() as usize];
    js_bytes.copy_to(&mut bytes);
    bytes
}
