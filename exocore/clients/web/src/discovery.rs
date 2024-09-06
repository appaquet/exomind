use std::{rc::Rc, time::Duration};

use exocore_core::cell::{CellConfigExt, CellNodeConfigExt, LocalNodeConfigExt};
use exocore_discovery::{Client, DEFAULT_DISCO_SERVER};
use exocore_protos::core::{node_cell_config, CellConfig, NodeCellConfig};
use futures::{channel::oneshot, future::Shared, FutureExt};
use wasm_bindgen::prelude::*;

use crate::{
    js::{into_js_error, wrap_js_error},
    node::LocalNode,
};

#[wasm_bindgen]
pub struct Discovery {
    client: Rc<Client>,

    // used to cancel ongoing join when dropped
    _drop_sender: oneshot::Sender<()>,
    drop_receiver: Shared<oneshot::Receiver<()>>,
}

#[wasm_bindgen]
impl Discovery {
    #[wasm_bindgen(constructor)]
    pub fn new(service_url: Option<String>) -> Discovery {
        let service_url = service_url.as_deref().unwrap_or(DEFAULT_DISCO_SERVER);
        let client = Client::new(service_url).expect("couldn't create discovery client");

        let (drop_sender, drop_receiver) = oneshot::channel();

        Discovery {
            client: Rc::new(client),
            _drop_sender: drop_sender,
            drop_receiver: drop_receiver.shared(),
        }
    }

    pub fn join_cell(
        &self,
        local_node: &LocalNode,
        pin_callback: js_sys::Function,
    ) -> js_sys::Promise {
        let client = self.client.clone();
        let local_node = local_node.clone();

        let join_fut = async move {
            let roles = Vec::new(); // thin client, no roles for now
            let cell_node = local_node.config.create_cell_node_config(roles);
            let cell_node_yml = cell_node
                .to_yaml_string()
                .map_err(|err| into_js_error("couldn't convert to yaml config", err))?;

            let create_resp = client
                .create(cell_node_yml.as_bytes(), true)
                .await
                .map_err(|err| into_js_error("sending config to discovery service", err))?;

            let pin = create_resp.pin.to_formatted_string();
            pin_callback
                .call1(&JsValue::null(), &JsValue::from_str(pin.as_str()))
                .map_err(|err| wrap_js_error("calling pin callback", err))?;

            let reply_pin = create_resp.reply_pin.ok_or("expected reply pin")?;
            let get_cell_resp = client
                .get_loop(reply_pin, Duration::from_secs(60))
                .await
                .map_err(|err| into_js_error("getting config to discovery service", err))?;

            let get_cell_payload = get_cell_resp.decode_payload().map_err(|err| {
                into_js_error("couldn't decode payload from discovery service", err)
            })?;
            let cell_config =
                CellConfig::read_yaml(get_cell_payload.as_slice()).map_err(|err| {
                    into_js_error("couldn't decode config retrieved from discovery", err)
                })?;

            let mut local_node_config = local_node.config.clone();
            local_node_config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Inline(cell_config)),
                ..Default::default()
            });

            let local_node = LocalNode::from_config(local_node_config)
                .map_err(|err| wrap_js_error("couldn't create local node from config", err))?;
            Ok(local_node.into())
        };

        // wait for discovery to complete OR the struct being dropped
        let drop_receiver = self.drop_receiver.clone();
        let select_fut = async move {
            futures::select! {
                join_res = join_fut.fuse() => {
                    join_res
                },
                _ = drop_receiver.fuse() => {
                    Err("discovery dropped".into())
                },
            }
        };

        wasm_bindgen_futures::future_to_promise(select_fut)
    }
}
