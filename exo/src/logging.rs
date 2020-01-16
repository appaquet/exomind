extern crate log4rs;

use self::log4rs::append::console::ConsoleAppender;
use self::log4rs::config::{Appender, Config, Logger, Root};
use log::LevelFilter;

pub fn setup(level: Option<LevelFilter>) {
    let appender = Box::new(ConsoleAppender::builder().build());
    let level = level.unwrap_or(LevelFilter::Info);

    let config = Config::builder()
        .appender(Appender::builder().build("default_output", appender))
        .logger(Logger::builder().build("tokio_reactor", LevelFilter::Info))
        .logger(Logger::builder().build("tokio_threadpool", LevelFilter::Info))
        .logger(Logger::builder().build("yamux", LevelFilter::Info))
        .build(Root::builder().appender("default_output").build(level))
        .expect("Couldn't configure logger");

    log4rs::init_config(config).expect("Couldn't initialize log4rs");
}
