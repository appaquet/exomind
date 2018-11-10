#[cfg(test)]
pub fn setup_logging() {
    use stderrlog;
    stderrlog::new().verbosity(4).init().unwrap();
}
