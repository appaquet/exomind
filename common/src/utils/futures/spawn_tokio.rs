use futures::{Future as Future03, FutureExt, TryFutureExt};

pub use futures01::Future as Future01;

#[cfg(any(test, feature = "tests_utils", feature = "runtime"))]
pub use tokio_compat::runtime::Runtime;

pub use tokio02::task::spawn_blocking;
pub use tokio02::time::{delay_for, delay_until, interval, interval_at, Interval};

pub fn spawn_future_01<F>(f: F) -> tokio::executor::Spawn
where
    F: Future01<Item = (), Error = ()> + 'static + Send,
{
    tokio::executor::spawn(f)
}

pub fn spawn_future<F>(f: F) -> tokio::executor::Spawn
where
    F: Future03<Output = ()> + 'static + Send,
{
    tokio::executor::spawn(f.boxed().unit_error().compat())
}

pub fn spawn_future_non_send<F>(_f: F)
where
    F: Future03<Output = Result<(), ()>> + 'static,
{
    unimplemented!()
}
