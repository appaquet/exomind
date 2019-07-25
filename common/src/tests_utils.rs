// TODO: move to new project for tests only

use self::log4rs::config::Logger;
use failure::err_msg;
use futures::prelude::*;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

extern crate log4rs;

pub fn setup_logging() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Root};

    let stdout = ConsoleAppender::builder().build();

    // see https://docs.rs/log4rs/*/log4rs/
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("tokio_reactor", LevelFilter::Info))
        .logger(Logger::builder().build("tantivy", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

///
/// Allows peeking into a future and watch its status, while exposing another future that
/// makes the inner future progress.
///
pub struct FuturePeek {
    status: Arc<Mutex<FutureStatus>>,
}

impl FuturePeek {
    pub fn new<F, I, E>(
        fut: F,
    ) -> (
        Box<dyn Future<Item = I, Error = E> + 'static + Send>,
        FuturePeek,
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

        (wrapped_future, FuturePeek { status })
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

///
/// Testing utils
///
pub fn expect_eventually<F>(cb: F)
where
    F: FnMut() -> bool,
{
    expect_eventually_within(Duration::from_secs(5), cb)
}

pub fn expect_eventually_within<F>(timeout: Duration, mut cb: F)
where
    F: FnMut() -> bool,
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
    F: FnMut() -> Result<R, E>,
{
    expect_result_within(cb, Duration::from_secs(5))
}

pub fn expect_result_within<F, R, E: Debug>(mut cb: F, time: Duration) -> R
where
    F: FnMut() -> Result<R, E>,
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

#[inline]
pub fn result_assert_equal<A: PartialEq + Debug>(left: A, right: A) -> Result<(), failure::Error> {
    if left != right {
        Err(err_msg(format!("expected: {:?} got: {:?}", left, right)))
    } else {
        Ok(())
    }
}

#[inline]
pub fn result_assert_true(value: bool) -> Result<(), failure::Error> {
    if !value {
        Err(err_msg("value is not true"))
    } else {
        Ok(())
    }
}

#[inline]
pub fn result_assert_false(value: bool) -> Result<(), failure::Error> {
    if value {
        Err(err_msg("value is not false"))
    } else {
        Ok(())
    }
}
