#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(feature = "local")]
#[macro_use]
extern crate smallvec;

#[allow(unused_imports)]
#[macro_use]
extern crate anyhow;

pub mod entity;
pub mod error;
pub mod mutation;
pub mod ordering;
pub mod query;
pub mod store;

#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "remote")]
pub mod remote;
