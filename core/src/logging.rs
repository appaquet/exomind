extern crate log4rs;

use self::log4rs::append::console::ConsoleAppender;
use self::log4rs::config::{Appender, Config, Logger, Root};
use log::LevelFilter;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup(level: Option<LevelFilter>) {
    INIT.call_once(|| {
        let appender = Box::new(ConsoleAppender::builder().build());
        let level = level.unwrap_or(LevelFilter::Info);

        let mut config =
            Config::builder().appender(Appender::builder().build("default_output", appender));

        if level < LevelFilter::Debug {
            config = config
                .logger(Logger::builder().build("tantivy::directory", LevelFilter::Warn))
                .logger(Logger::builder().build("tantivy::indexer", LevelFilter::Warn));
        }

        let config = config
            .build(Root::builder().appender("default_output").build(level))
            .expect("Couldn't configure logger");

        log4rs::init_config(config).expect("Couldn't initialize log4rs");
    });
}
