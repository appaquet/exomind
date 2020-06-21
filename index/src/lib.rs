#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[cfg(feature = "local_store")]
#[macro_use]
extern crate smallvec;

pub mod entity;
pub mod error;
pub mod mutation;
pub mod ordering;
pub mod query;

#[cfg(feature = "local_store")]
pub mod local;
pub mod remote;
