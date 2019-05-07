#![deny(bare_trait_objects)]

extern crate exocore_common;
extern crate exocore_data;
extern crate exocore_index;
extern crate exocore_transport;

extern crate log;
extern crate structopt;
extern crate tokio;

#[cfg(test)]
pub mod logging;

use exocore_common::cell::{Cell, CellID};
use exocore_common::node::LocalNode;
use exocore_transport::lp2p::Config;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    data_node: bool,
    index_node: bool,
    port: u16,
}

fn main() {
    // TODO: Create tokio runtime

    // TODO: Create engine
    // TODO: Take engine handle for each user

    let rt = tokio::runtime::Runtime::new();

    let cell_id = CellID::from_string("cell1");
    let cell = Cell::new(cell_id);

    let transport_config = exocore_transport::lp2p::Config::default();
    let transport = exocore_transport::lp2p::NodeTransport::new(cell.clone(), transport_config);



    println!("Hello world");
}
