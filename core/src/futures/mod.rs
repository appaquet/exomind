#[cfg(not(target_arch = "wasm32"))]
mod spawn_tokio;
#[cfg(not(target_arch = "wasm32"))]
pub use spawn_tokio::*;

#[cfg(target_arch = "wasm32")]
mod spawn_wasm;
pub use futures::executor::block_on;
#[cfg(target_arch = "wasm32")]
pub use spawn_wasm::*;

mod owned_spawn;
pub use owned_spawn::*;

mod batching_stream;
pub use batching_stream::*;
