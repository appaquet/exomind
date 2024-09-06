#[cfg(feature = "exocore-chain")]
pub extern crate exocore_chain as chain;
#[cfg(feature = "exocore-core")]
pub extern crate exocore_core as core;
#[cfg(feature = "exocore-protos")]
pub extern crate exocore_protos as protos;
#[cfg(feature = "exocore-store")]
pub extern crate exocore_store as store;
#[cfg(feature = "exocore-transport")]
pub extern crate exocore_transport as transport;

#[cfg(feature = "exocore-apps-sdk")]
pub mod apps {
    pub extern crate exocore_apps_sdk as sdk;
}

#[cfg(feature = "client")]
pub mod client;
