use std::fmt::Display;

use wasm_bindgen::JsValue;

pub fn into_js_error<E: Display>(err: E) -> JsValue {
    let js_error = js_sys::Error::new(&format!("Error executing query: {}", err));
    JsValue::from(js_error)
}
