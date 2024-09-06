use exocore_core::{
    cell::{LocalNode as CoreLocalNode, LocalNodeConfigExt},
    dir::ram::RamDirectory,
};
use exocore_protos::core::LocalNodeConfig;
use wasm_bindgen::prelude::*;

use crate::js::into_js_error;

#[wasm_bindgen]
#[derive(Clone)]
pub struct LocalNode {
    pub(crate) node: CoreLocalNode,
    pub(crate) config: LocalNodeConfig,
}

#[wasm_bindgen]
impl LocalNode {
    pub fn generate() -> LocalNode {
        let node = CoreLocalNode::generate();
        let config = LocalNodeConfig {
            name: format!("web-{}", node.name()),
            ..node.config().clone()
        };
        LocalNode { node, config }
    }

    pub(crate) fn from_config(config: LocalNodeConfig) -> Result<LocalNode, JsValue> {
        let node = CoreLocalNode::from_config(RamDirectory::new(), config.clone())
            .map_err(|err| into_js_error("couldn't create node from config", err))?;

        Ok(LocalNode { node, config })
    }

    pub fn from_yaml(yaml: String) -> Result<LocalNode, JsValue> {
        let config = LocalNodeConfig::read_yaml(yaml.as_bytes())
            .map_err(|err| into_js_error("couldn't create node config from yaml", err))?;
        Self::from_config(config)
    }

    pub fn from_storage(storage: web_sys::Storage) -> Result<LocalNode, JsValue> {
        let config_str: Option<String> = storage.get("node_config")?;
        let config = config_str.ok_or("couldn't find `node_config` in storage")?;
        Self::from_yaml(config)
    }

    pub fn save_to_storage(&self, storage: web_sys::Storage) -> Result<(), JsValue> {
        let config = self
            .node
            .inlined_config()
            .map_err(|err| into_js_error("couldn't inline config", err))?;
        let config_yaml = config
            .to_yaml_string()
            .map_err(|err| into_js_error("couldn't convert to yaml config", err))?;
        storage.set("node_config", config_yaml.as_str())?;
        Ok(())
    }

    pub fn to_yaml(&self) -> Result<String, JsValue> {
        self.config
            .to_yaml_string()
            .map_err(|err| into_js_error("couldn't convert to yaml config", err))
    }

    #[wasm_bindgen(getter)]
    pub fn has_configured_cell(&self) -> bool {
        !self.config.cells.is_empty()
    }
}
