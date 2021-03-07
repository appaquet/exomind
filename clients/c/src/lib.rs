use std::{ffi::CStr, sync::Once};

use libc::c_char;

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
pub extern "C" fn exocore_init(log_level: usize, log_file: *const c_char) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let log_level = match log_level {
            1 => Some(log::LevelFilter::Error),
            2 => Some(log::LevelFilter::Warn),
            3 => Some(log::LevelFilter::Info),
            4 => Some(log::LevelFilter::Debug),
            5 => Some(log::LevelFilter::Trace),
            _ => None,
        };
        let file = if !log_file.is_null() {
            unsafe { Some(CStr::from_ptr(log_file).to_string_lossy().to_string()) }
        } else {
            None
        };

        exocore_core::logging::setup(log_level, file);
    });
}
