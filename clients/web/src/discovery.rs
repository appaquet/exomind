use crate::{
    js::{into_js_error, wrap_js_error},
    node::LocalNode,
};
use exocore_core::{
    cell::{CellConfigExt, LocalNodeConfigExt},
    protos::core::{node_cell_config, CellConfig, NodeCellConfig},
};
use exocore_discovery::{Client, DEFAULT_DISCO_SERVER};
use std::{rc::Rc, time::Duration};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Discovery {
    client: Rc<Client>,
}

#[wasm_bindgen]
impl Discovery {
    pub fn new(disco_url: Option<String>) -> Discovery {
        let disco_url = disco_url.as_deref().unwrap_or(DEFAULT_DISCO_SERVER);
        let client = Client::new(disco_url).expect("couldn't create discovery client");

        Discovery {
            client: Rc::new(client),
        }
    }

    pub fn push_node_config(
        &self,
        local_node: LocalNode,
        pin_callback: js_sys::Function,
    ) -> js_sys::Promise {
        let client = self.client.clone();

        let fut = async move {
            let local_node_yml = local_node.to_yaml()?;
            let create_resp = client
                .create(local_node_yml.as_bytes(), true)
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

            let get_cell_payload = get_cell_resp
                .decode_payload()
                .map_err(|_| "couldn't decode payload from discovery service")?;
            let cell_config =
                CellConfig::from_yaml(get_cell_payload.as_slice()).map_err(|err| {
                    into_js_error("couldn't decode config retrieved from discovery", err)
                })?;

            let mut local_node_config = local_node.config.clone();
            local_node_config.add_cell(NodeCellConfig {
                location: Some(node_cell_config::Location::Inline(cell_config)),
            });

            let local_node = LocalNode::from_config(local_node_config)
                .map_err(|err| wrap_js_error("couldn't create local node from config", err))?;
            Ok(local_node.into())
        };

        wasm_bindgen_futures::future_to_promise(fut)
    }
}
