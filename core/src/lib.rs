#![deny(bare_trait_objects)]

#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[cfg(any(test, feature = "tests-utils"))]
#[macro_use]
extern crate anyhow;

pub extern crate capnp;
pub mod protos;
pub(crate) use self::protos::generated::{common_capnp, data_chain_capnp, data_transport_capnp}; // generated capnp protos expect to be at root

pub mod cell;
pub mod crypto;
pub mod framing;
pub mod futures;
#[cfg(feature = "logger")]
pub mod logging;
pub mod simple_store;
#[cfg(any(test, feature = "tests-utils"))]
pub mod tests_utils;
pub mod time;
pub mod utils;
