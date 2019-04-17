#[cfg(any(test, feature = "tests_utils"))]
use stderrlog::Timestamp;

#[cfg(any(test, feature = "tests_utils"))]
pub fn setup_logging() {
    use stderrlog;
    stderrlog::new()
        .timestamp(Timestamp::Millisecond)
        .verbosity(3)
        .init()
        .unwrap();
}
