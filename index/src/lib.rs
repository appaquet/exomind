#![deny(bare_trait_objects)]

#[macro_use]
extern crate log;

#[cfg(feature = "local-store")]
#[macro_use]
extern crate smallvec;

pub mod entity;
pub mod error;
pub mod mutation;
pub mod ordering;
pub mod query;

#[cfg(feature = "local-store")]
pub mod local;
pub mod remote;
