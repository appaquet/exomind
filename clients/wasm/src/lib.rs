#![deny(bare_trait_objects)]

#[macro_use]
extern crate log;

mod js;
mod ws;

pub mod client;
pub mod mutation;
pub mod query;
