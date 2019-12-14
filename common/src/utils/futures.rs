use futures::{Future as Future03, FutureExt, TryFutureExt};
use futures01::Future as Future01;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use futures::compat::Future01CompatExt;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn spawn_future_01<F>(f: F) -> tokio::executor::Spawn
where
    F: Future01<Item = (), Error = ()> + 'static + Send,
{
    tokio::executor::spawn(f)
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn spawn_future<F>(f: F) -> tokio::executor::Spawn
where
    F: Future03<Output = Result<(), ()>> + 'static + Send,
{
    tokio::executor::spawn(f.boxed().compat())
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn spawn_future_non_send<F>(_f: F)
where
    F: Future03<Output = Result<(), ()>> + 'static,
{
    unimplemented!()
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn spawn_future_01<F>(f: F)
where
    F: Future01<Item = (), Error = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f.compat().unwrap_or_else(|_| ()));
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn spawn_future<F>(f: F)
where
    F: Future03<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn spawn_future_non_send<F>(f: F)
where
    F: Future03<Output = Result<(), ()>> + 'static,
{
    wasm_bindgen_futures::spawn_local(f.unwrap_or_else(|_| ()));
}
