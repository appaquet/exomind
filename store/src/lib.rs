#[cfg(any(feature = "local", feature = "remote"))]
#[macro_use]
extern crate log;

#[cfg(feature = "local")]
#[macro_use]
extern crate smallvec;

pub mod entity;
pub mod error;
pub mod mutation;
pub mod ordering;
pub mod query;

#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "remote")]
pub mod remote;
