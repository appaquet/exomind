use futures::Future;

pub use tokio::task::spawn_blocking;
pub use tokio::time::{delay_for, delay_until, interval, interval_at, Interval};

#[cfg(any(test, feature = "tests_utils", feature = "runtime"))]
pub use tokio::runtime::Runtime;

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
