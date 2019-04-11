// TODO: move to new project for tests only

use futures::prelude::*;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct FutureWatch {
    status: Arc<Mutex<FutureStatus>>,
}

impl FutureWatch {
    pub fn new<F, I, E>(
        fut: F,
    ) -> (
        Box<dyn Future<Item = I, Error = E> + 'static + Send>,
        FutureWatch,
    )
    where
        F: Future<Item = I, Error = E> + Send + 'static,
        I: Send + 'static,
        E: Send + 'static,
    {
        let status = Arc::new(Mutex::new(FutureStatus::NotReady));

        let inner_status = Arc::downgrade(&status);
        let wrapped_future = Box::new(fut.then(move |res| {
            if let Some(upgraded) = inner_status.upgrade() {
                if let Ok(mut unlocked) = upgraded.lock() {
                    match res {
                        Ok(_) => *unlocked = FutureStatus::Ok,
                        Err(_) => *unlocked = FutureStatus::Failed,
                    }
                }
            }

            res
        }));

        (wrapped_future, FutureWatch { status })
    }

    pub fn get_status(&self) -> FutureStatus {
        if let Ok(unlocked) = self.status.as_ref().lock() {
            *unlocked
        } else {
            FutureStatus::Failed
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FutureStatus {
    NotReady,
    Ok,
    Failed,
}

pub fn expect_eventually<F>(cb: F)
where
    F: Fn() -> bool,
{
    expect_eventually_within(Duration::from_secs(5), cb)
}

pub fn expect_eventually_within<F>(timeout: Duration, cb: F)
where
    F: Fn() -> bool,
{
    let start_time = Instant::now();
    while start_time.elapsed() < timeout {
        if cb() {
            return;
        } else {
            std::thread::sleep(Duration::from_millis(10))
        }
    }
    panic!(
        "Expected result within {:?}, but waited {:?} without result",
        timeout,
        start_time.elapsed()
    );
}

pub fn expect_result<F, R, E: Debug>(cb: F) -> R
where
    F: Fn() -> Result<R, E>,
{
    expect_result_within(cb, Duration::from_secs(5))
}

pub fn expect_result_within<F, R, E: Debug>(cb: F, time: Duration) -> R
where
    F: Fn() -> Result<R, E>,
{
    let begin = Instant::now();
    loop {
        match cb() {
            Ok(res) => return res,
            Err(err) => {
                if begin.elapsed() >= time {
                    panic!("Couldn't get a result within time. Last error: {:?}", err);
                } else {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
}
