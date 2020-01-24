#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

pub mod error;
pub mod store;

pub mod mutation;
pub mod query;
