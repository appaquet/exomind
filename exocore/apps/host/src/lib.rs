#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod config;
mod error;

#[cfg(any(
    all(
        target_arch = "x86_64",
        any(target_os = "linux", target_os = "macos", target_os = "windows")
    ),
    all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos"))
))]
pub mod runtime;

pub use config::Config;
pub use error::Error;
