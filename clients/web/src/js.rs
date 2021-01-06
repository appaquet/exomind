use std::fmt::Display;

use wasm_bindgen::JsValue;

pub fn into_js_error<A: Display, E: Display>(action: A, err: E) -> JsValue {
    let js_error = js_sys::Error::new(&format!("Error '{}': {}", action, err));
    JsValue::from(js_error)
}

pub fn wrap_js_error<A: Display>(action: A, err: JsValue) -> JsValue {
    let js_error = js_sys::Error::new(&format!("Error '{}': {:?}", action, err));
    JsValue::from(js_error)
}
