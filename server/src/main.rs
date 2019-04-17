#![deny(bare_trait_objects)]

extern crate exocore_common;
extern crate exocore_data;
extern crate exocore_index;

extern crate log;
extern crate structopt;

#[cfg(test)]
pub mod logging;

use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    data_node: bool,
    index_node: bool,
}

fn main() {
    // TODO: Create tokio runtime

    // TODO: Create engine
    // TODO: Take engine handle for each user

    println!("Hello world");
}
