use futures::Future;
#[cfg(any(test, feature = "tests-utils", feature = "runtime"))]
pub use tokio::runtime::Builder;
#[cfg(any(test, feature = "tests-utils", feature = "runtime"))]
pub use tokio::runtime::Runtime;
#[cfg(any(test, feature = "tests-utils", feature = "runtime"))]
pub use tokio::time::{interval, interval_at, sleep, sleep_until, Interval};
pub use tokio::{self, task::spawn_blocking};

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
