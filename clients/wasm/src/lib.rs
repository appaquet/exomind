#![deny(bare_trait_objects)]

#[cfg(target = "wasm32-unknown-unknown")]
pub mod client;
