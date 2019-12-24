#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod spawn_tokio;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub use spawn_tokio::*;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod spawn_wasm;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use spawn_wasm::*;

mod owned_spawn;
pub use futures::executor::block_on;
pub use owned_spawn::*;
