extern crate exocore_common;
extern crate exocore_data;
extern crate exocore_index;

#[macro_use]
extern crate structopt;

use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    data_node: bool,
    index_node: bool,
}

fn main() {
    println!("Hello world");
}
