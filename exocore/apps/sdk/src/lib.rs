#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub(crate) mod binding;
pub(crate) mod logging;

pub mod app;
pub mod client;
pub mod executor;
pub mod store;
pub mod time;

pub use exocore_apps_macros::exocore_app;

pub mod prelude {
    pub use super::{
        app::{App, AppError},
        client::Exocore,
        executor::spawn,
        exocore_app,
        store::{Store, StoreError},
        time::{now, sleep, Timestamp},
    };
}
