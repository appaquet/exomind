#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

#[allow(unused_imports)]
#[macro_use]
extern crate maplit;

pub mod domain;
pub mod error;
pub mod index;
pub mod mutation;
pub mod query;
pub mod results;
