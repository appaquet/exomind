use self::log4rs::config::Logger;
use failure::err_msg;
use std::fmt::Debug;
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
        .logger(Logger::builder().build("tantivy", LevelFilter::Error))
        .logger(Logger::builder().build("exocore_chain", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

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

/// Finds the given relative path from root of the project by popping current directory until
/// we find the root directory. This is needed since tests may be executed from root directory, but
/// also from test's file directory.
pub fn root_test_fixtures_path(relative_path: &str) -> std::path::PathBuf {
    let cur_dir = std::env::current_dir().expect("Couldn't get current directory");
    for level in 0..5 {
        let mut abs_path = cur_dir.clone();
        for _ in 0..level {
            abs_path.pop();
        }
        abs_path.push(relative_path);

        if abs_path.exists() {
            return abs_path;
        }
    }

    panic!(
        "Couldn't find test fixtures file from root: {:?} (cur_dir={:?})",
        relative_path, cur_dir
    )
}
