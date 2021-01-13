use std::task::{Context, Poll};

use futures::prelude::*;
use wasm_timer::{Delay, Interval as WasmInterval};

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
    Interval(WasmInterval::new(interval))
}

pub fn sleep(duration: std::time::Duration) -> Delay {
    Delay::new(duration)
}

// We wrap wasm_timer interval to be compatible with Tokio 1
pub struct Interval(WasmInterval);

impl Interval {
    pub async fn tick(&mut self) {
        futures::future::poll_fn(|cx| self.poll_tick(cx)).await;
    }

    pub fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Option<()>> {
        self.0.poll_next_unpin(cx)
    }
}
