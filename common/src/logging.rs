extern crate log4rs;

use self::log4rs::append::Append;
use self::log4rs::append::console::ConsoleAppender;
use self::log4rs::config::{Appender, Config, Logger, Root};

use log::LevelFilter;

pub fn setup() {
    let appender = Box::new(ConsoleAppender::builder().build());

    let config = Config::builder()
        .appender(Appender::builder().build("default_output", appender))
        .logger(Logger::builder().build("ampplayer", LevelFilter::Debug))
        .build(Root::builder().appender("default_output").build(LevelFilter::Info))
        .expect("Couldn't configure logger");

    log4rs::init_config(config).expect("Couldn't initialize log4rs");
}