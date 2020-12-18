use futures::{Future, TryFutureExt};
use wasm_timer::{Delay, Interval};

pub fn spawn_future<F>(f: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f);
}

pub fn spawn_future_non_send<F>(f: F)
where
    F: Future<Output = Result<(), ()>> + 'static,
{
    wasm_bindgen_futures::spawn_local(f.unwrap_or_else(|_| ()));
}

pub fn interval(interval: std::time::Duration) -> Interval {
    Interval::new(interval)
}

pub fn sleep(duration: std::time::Duration) -> Delay {
    Delay::new(duration)
}
