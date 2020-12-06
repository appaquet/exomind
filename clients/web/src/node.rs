use crate::js::into_js_error;
use exocore_core::{
    cell::{LocalNode as CoreLocalNode, LocalNodeConfigExt},
    protos::core::LocalNodeConfig,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LocalNode {
    _node: CoreLocalNode,
    pub(crate) config: LocalNodeConfig,
}

#[wasm_bindgen]
impl LocalNode {
    pub fn generate() -> LocalNode {
        let node = CoreLocalNode::generate();
        LocalNode {
            _node: node.clone(),
            config: LocalNodeConfig {
                keypair: node.keypair().encode_base58_string(),
                public_key: node.public_key().encode_base58_string(),
                name: format!("web-{}", node.name()),
                id: node.id().to_string(),
                ..Default::default()
            },
        }
    }

    pub(crate) fn from_config(config: LocalNodeConfig) -> Result<LocalNode, JsValue> {
        let node = CoreLocalNode::new_from_config(config.clone())
            .map_err(|err| into_js_error("couldn't create node from config", err))?;

        Ok(LocalNode {
            _node: node,
            config,
        })
    }

    pub fn from_yaml(yaml: String) -> Result<LocalNode, JsValue> {
        let config = LocalNodeConfig::from_yaml_reader(yaml.as_bytes())
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
            .config
            .inlined()
            .map_err(|err| into_js_error("couldn't inline config", err))?;
        let config_json = config
            .to_yaml()
            .map_err(|err| into_js_error("couldn't convert to yaml config", err))?;
        storage.set("node_config", config_json.as_str())?;
        Ok(())
    }

    pub fn to_yaml(&self) -> Result<String, JsValue> {
        self.config
            .to_yaml()
            .map_err(|err| into_js_error("couldn't convert to yaml config", err))
    }

    pub fn has_configured_cell(&self) -> bool {
        !self.config.cells.is_empty()
    }
}
