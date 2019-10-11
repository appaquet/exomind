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

pub mod entity;
pub mod error;
pub mod schema;
pub mod serialization;

pub mod test_schema;
