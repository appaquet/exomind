use std::sync::{Arc, Mutex};

use futures::StreamExt;
use prost::Message;
use wasm_bindgen::prelude::*;

use exocore_core::cell::Cell;
use exocore_core::crypto::keys::{Keypair, PublicKey};
use exocore_core::futures::spawn_future_non_send;
use exocore_core::node::{LocalNode, Node};
use exocore_core::protos::generated::exocore_index::{EntityMutation, EntityQuery};
use exocore_core::time::Clock;
use exocore_index::store::remote::{Client, ClientConfiguration, ClientHandle};
use exocore_transport::transport::ConnectionStatus;
use exocore_transport::{InEvent, Libp2pTransport, TransportHandle, TransportLayer};

use crate::js::into_js_error;
use crate::watched_query::WatchedQuery;
use exocore_core::protos::prost::ProstMessageExt;
use exocore_transport::lp2p::Libp2pTransportConfig;

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
        url: &str,
        status_change_callback: Option<js_sys::Function>,
    ) -> Result<ExocoreClient, JsValue> {
        wasm_logger::init(wasm_logger::Config::default());
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        //        // TODO: To be cleaned up when cell management will be ironed out: https://github.com/appaquet/exocore/issues/80
        //        let local_node = LocalNode::generate();
        //        let cell_pk =
        //            PublicKey::decode_base58_string("pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ")
        //                .expect("Couldn't decode cell publickey");
        //        let cell = Cell::new(cell_pk, local_node);
        //        let clock = Clock::new();
        //
        //        let remote_node_pk =
        //            PublicKey::decode_base58_string("pe5ZG43uAcfLxYSGaQgj1w8hQT4GBchEVg5mS2b1EfXcMb")
        //                .expect("Couldn't decode cell publickey");
        //        let remote_node = Node::new_from_public_key(remote_node_pk);
        //
        //        let mut transport = BrowserTransportClient::new(url, remote_node.clone());

        // TODO: To be cleaned up when cell management will be ironed out: https://github.com/appaquet/exocore/issues/80
        let local_node = LocalNode::new_from_keypair(Keypair::decode_base58_string("ae4WbDdfhv3416xs8S2tQgczBarmR8HKABvPCmRcNMujdVpDzuCJVQADVeqkqwvDmqYUUjLqv7kcChyCYn8R9BNgXP").unwrap());

        let transport_config = Libp2pTransportConfig::default();
        let mut transport = Libp2pTransport::new(local_node.clone(), transport_config);

        let cell_pk =
            PublicKey::decode_base58_string("pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ")
                .expect("Couldn't decode cell publickey");
        let cell = Cell::new(cell_pk, local_node);
        let clock = Clock::new();

        let remote_node_pk =
            PublicKey::decode_base58_string("peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP")
                .expect("Couldn't decode cell publickey");
        let remote_node = Node::new_from_public_key(remote_node_pk);
        let remote_addr = "/ip4/127.0.0.1/tcp/3341/ws"
            .parse()
            .expect("Couldn't parse remote node addr");
        remote_node.add_address(remote_addr);
        {
            cell.nodes_mut().add(remote_node.clone());
        }

        let index_handle = transport
            .get_handle(cell.clone(), TransportLayer::Index)
            .unwrap();
        let remote_store = Client::new(
            ClientConfiguration::default(),
            cell.clone(),
            clock,
            index_handle,
            remote_node,
        )
        .expect("Couldn't create index");

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

        let mut client_transport_handle =
            transport.get_handle(cell, TransportLayer::Client).unwrap();
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
    pub fn mutate(&self, mutation_bytes: js_sys::Uint8Array) -> js_sys::Promise {
        info!("Got mutation");

        let mut bytes = vec![0u8; mutation_bytes.length() as usize];
        mutation_bytes.copy_to(&mut bytes);
        let entity_mutation =
            EntityMutation::decode(bytes.as_ref()).expect("Couldn't encode query");

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
    pub fn query(&self, query_bytes: js_sys::Uint8Array) -> js_sys::Promise {
        let mut bytes = vec![0u8; query_bytes.length() as usize];
        query_bytes.copy_to(&mut bytes);
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
    pub fn watched_query(&self, query_bytes: js_sys::Uint8Array) -> WatchedQuery {
        let mut bytes = vec![0u8; query_bytes.length() as usize];
        query_bytes.copy_to(&mut bytes);
        let entity_query = EntityQuery::decode(bytes.as_ref()).expect("Couldn't encode query");

        WatchedQuery::new(self.store_handle.clone(), entity_query)
    }
}

impl Drop for ExocoreClient {
    fn drop(&mut self) {
        info!("Got dropped");
    }
}
