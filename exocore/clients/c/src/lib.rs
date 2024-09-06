use std::{ffi::CStr, sync::Once};

use exocore_protos::prost::Message;
use libc::c_char;
use utils::BytesVec;

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
///
/// # Safety
/// * `log_file` should be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn exocore_init(log_level: usize, log_file: *const c_char) {
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

        std::panic::set_hook(Box::new(|info| {
            error!("Panic occurred: {}", info);
        }));

        exocore_core::logging::setup(log_level, file);
        info!("exocore build: {}", exocore_core::build::build_info_str());
    });
}

/// Returns build information in protobuf encoded bytes of
/// `exocore.core.BuildInfo` message.
///
/// # Safety
/// * Returned bytes should be freed using `exocore_bytes_free`.
#[no_mangle]
pub extern "C" fn exocore_build_info() -> BytesVec {
    let info = exocore_core::build::build_info();
    let bytes = info.encode_to_vec();
    BytesVec::from_vec(bytes)
}
