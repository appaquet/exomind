#![allow(clippy::unused_unit)] // TODO: Remove me when wasm-bindgen is bumped

#[macro_use]
extern crate log;

pub mod client;
pub mod discovery;
pub mod node;
pub mod watched_query;

mod js;

use std::sync::Once;

use exocore_protos::prost::Message;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        info!("exocore build: {}", exocore_core::build::build_info_str());
    });
}

#[wasm_bindgen]
pub fn generate_id(prefix: Option<String>) -> String {
    match prefix.as_deref() {
        Some("") => exocore_core::utils::id::generate_id(),
        Some(prefix) => exocore_core::utils::id::generate_prefixed_id(prefix),
        None => exocore_core::utils::id::generate_id(),
    }
}

#[wasm_bindgen]
pub fn build_info() -> js_sys::Uint8Array {
    let info_data = exocore_core::build::build_info().encode_to_vec();
    js_sys::Uint8Array::from(info_data.as_ref())
}
