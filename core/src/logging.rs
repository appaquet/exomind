extern crate log4rs;

use std::{path::Path, sync::Once};

use log::LevelFilter;
use log4rs::append::file::FileAppender;

use self::log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};

static INIT: Once = Once::new();

pub fn setup<P: AsRef<Path>>(level: Option<LevelFilter>, file: Option<P>) {
    INIT.call_once(|| {
        let level = level.unwrap_or(LevelFilter::Info);
        let encoder = Box::new(PatternEncoder::new("{d} - {h({l})} - {t} - {m}{n}"));

        let mut appenders = vec!["console_output"];
        let console_appender = ConsoleAppender::builder().encoder(encoder.clone()).build();
        let mut config = Config::builder()
            .appender(Appender::builder().build("console_output", Box::new(console_appender)));

        if let Some(file) = file {
            let file_appender = FileAppender::builder()
                .encoder(encoder)
                .append(false)
                .build(file)
                .expect("Couldn't create file appender");

            appenders.push("file_output");

            config =
                config.appender(Appender::builder().build("file_output", Box::new(file_appender)));
        }

        if level < LevelFilter::Debug {
            config = config
                .logger(Logger::builder().build("tantivy::directory", LevelFilter::Warn))
                .logger(Logger::builder().build("tantivy::indexer", LevelFilter::Warn));
        }

        let config = config
            .build(Root::builder().appenders(appenders).build(level))
            .expect("Couldn't configure logger");

        log4rs::init_config(config).expect("Couldn't initialize log4rs");
    });
}
