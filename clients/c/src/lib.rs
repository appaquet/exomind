use std::sync::Once;

#[macro_use]
extern crate log;

pub mod client;
pub mod discovery;
pub mod node;
pub mod utils;

/// Initializes exocore library (ex: logging). This method should always be
/// called first in order for the library to correctly initialize.
///
/// This method can be called multiple times without problems as it makes sure
/// its logic is only executed once.
#[no_mangle]
pub extern "C" fn exocore_init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        exocore_core::logging::setup(Some(log::LevelFilter::Info));
    });
}
