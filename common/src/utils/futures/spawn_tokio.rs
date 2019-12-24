use futures::{Future as Future03, FutureExt, TryFutureExt};
use futures01::Future as Future01;

#[cfg(any(test, feature = "tests_utils"))]
pub use tokio_compat::runtime::Runtime;

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

pub trait AsyncRuntimeExt {
    fn spawn_async<F>(&mut self, f: F)
    where
        F: Future03<Output = ()> + Send + 'static;
}

impl AsyncRuntimeExt for tokio::runtime::Runtime {
    fn spawn_async<F>(&mut self, f: F)
    where
        F: Future03<Output = ()> + Send + 'static,
    {
        self.spawn(f.unit_error().boxed().compat());
    }
}
