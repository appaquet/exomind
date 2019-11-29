use futures::Future;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn spawn_future<F>(f: F) -> tokio::executor::Spawn
where
    F: Future<Item = (), Error = ()> + 'static + Send,
{
    tokio::executor::spawn(f)
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn spawn_future<F>(f: F)
where
    F: Future<Item = (), Error = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f);
}
