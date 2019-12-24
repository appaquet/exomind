use futures::compat::Future01CompatExt;
use futures::{Future as Future03, TryFutureExt};
use futures01::Future as Future01;

pub fn spawn_future_01<F>(f: F)
where
    F: Future01<Item = (), Error = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f.compat().unwrap_or_else(|_| ()));
}

pub fn spawn_future<F>(f: F)
where
    F: Future03<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f);
}

pub fn spawn_future_non_send<F>(f: F)
where
    F: Future03<Output = Result<(), ()>> + 'static,
{
    wasm_bindgen_futures::spawn_local(f.unwrap_or_else(|_| ()));
}
