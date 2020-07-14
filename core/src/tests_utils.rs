use std::fmt::Debug;
use std::time::{Duration, Instant};

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
pub fn result_assert_equal<A: PartialEq + Debug>(left: A, right: A) -> anyhow::Result<()> {
    if left != right {
        Err(anyhow!(format!("expected: {:?} got: {:?}", left, right)))
    } else {
        Ok(())
    }
}

#[inline]
pub fn result_assert_true(value: bool) -> anyhow::Result<()> {
    if !value {
        Err(anyhow!("value is not true"))
    } else {
        Ok(())
    }
}

#[inline]
pub fn result_assert_false(value: bool) -> anyhow::Result<()> {
    if value {
        Err(anyhow!("value is not false"))
    } else {
        Ok(())
    }
}

/// Finds the given relative path from root of the project by popping current
/// directory until we find the root directory. This is needed since tests may
/// be executed from root directory, but also from test's file directory.
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
