#![deny(bare_trait_objects)]

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[macro_use]
extern crate log;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod js;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod ws;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod client;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod mutation;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod query;
