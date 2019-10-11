use std::fmt::Display;

use futures::Future;
use wasm_bindgen::JsValue;

pub fn into_js_error<E: Display>(err: E) -> JsValue {
    let js_error = js_sys::Error::new(&format!("Error executing query: {}", err));
    JsValue::from(js_error)
}

// TODO: To be moved https://github.com/appaquet/exocore/issues/123
pub fn js_future_spawner(future: Box<dyn Future<Item = (), Error = ()> + Send>) {
    wasm_bindgen_futures::spawn_local(future);
}
