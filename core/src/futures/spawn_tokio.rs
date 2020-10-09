use futures::Future;

pub use tokio;
pub use tokio::task::{block_in_place, spawn_blocking};

#[cfg(any(test, feature = "tests-utils", feature = "runtime"))]
pub use tokio::time::{delay_for, delay_until, interval, interval_at, Interval};

#[cfg(any(test, feature = "tests-utils", feature = "runtime"))]
pub use tokio::runtime::Runtime;

#[cfg(any(test, feature = "tests-utils", feature = "runtime"))]
pub use tokio::runtime::Builder;

pub fn spawn_future<F>(f: F)
where
    F: Future<Output = ()> + 'static + Send,
{
    tokio::spawn(f);
}

pub fn spawn_future_non_send<F>(_f: F)
where
    F: Future<Output = Result<(), ()>> + 'static,
{
    unimplemented!()
}
