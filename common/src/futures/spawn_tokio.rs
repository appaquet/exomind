use futures::compat::Future01CompatExt;
use futures::Future as Future03;

pub use futures01::Future as Future01;

pub use tokio::task::spawn_blocking;
pub use tokio::time::{delay_for, delay_until, interval, interval_at, Interval};
pub use tokio01;
#[cfg(any(test, feature = "tests_utils", feature = "runtime"))]
pub use tokio_compat::runtime::Runtime;

pub fn spawn_future_01<F>(f: F)
where
    F: Future01<Item = (), Error = ()> + 'static + Send,
{
    tokio::spawn(f.compat());
}

pub fn spawn_future<F>(f: F)
where
    F: Future03<Output = ()> + 'static + Send,
{
    tokio::spawn(f);
}

pub fn spawn_future_non_send<F>(_f: F)
where
    F: Future03<Output = Result<(), ()>> + 'static,
{
    unimplemented!()
}
