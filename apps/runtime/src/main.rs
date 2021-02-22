#[macro_use]
extern crate log;

use std::sync::Arc;

use exocore_protos::apps::{InMessage, OutMessage};
use runtime::{AppRuntime, HostEnvironment};

mod runtime;

fn main() -> anyhow::Result<()> {
    struct MyEnv;
    impl HostEnvironment for MyEnv {
        fn handle_message(&self, out: OutMessage) {
            info!("Got out message: {:?}", out);
        }

        fn handle_log(&self, level: log::Level, msg: &str) {
            log!(level, "WASM APP: {}", msg);
        }
    }

    let app_runtime = AppRuntime::from_file("fixtures/example.wasm", Arc::new(MyEnv))?;
    app_runtime.send_message(InMessage::default())?;
    app_runtime.run()?;
    Ok(())
}
