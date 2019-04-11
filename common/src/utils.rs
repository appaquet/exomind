#[cfg(any(test, feature = "tests_utils"))]
extern crate log4rs;

#[cfg(any(test, feature = "tests_utils"))]
pub fn setup_logging() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Root};

    let stdout = ConsoleAppender::builder().build();

    // see https://docs.rs/log4rs/*/log4rs/
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
