#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate smallvec;

pub mod error;
pub mod store;

pub mod entity;
pub mod mutation;
pub mod query;
