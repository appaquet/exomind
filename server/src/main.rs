#![deny(bare_trait_objects)]

extern crate exocore_common;
extern crate exocore_data;
extern crate exocore_index;
extern crate exocore_transport;

extern crate log;

#[cfg(test)]
pub mod logging;

use exocore_common::cell::{Cell, CellID};
use exocore_common::node::LocalNode;
use exocore_common::time::Clock;
use exocore_data::{DirectoryChainStore, DirectoryChainStoreConfig, MemoryPendingStore};
use exocore_transport::TransportLayer;
use futures::prelude::*;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long = "data_node")]
    data_node: bool,
    #[structopt(long = "index_node")]
    index_node: bool,
    port: u16,
}

fn main() -> Result<(), failure::Error> {
    setup_logging();

    let opt = Opt::from_args();

    let mut rt = tokio::runtime::Runtime::new()?;

    let local_node = LocalNode::generate();
    local_node.add_address(format!("/ip4/127.0.0.1/tcp/{}", opt.port).parse().unwrap());
    let cell_id = CellID::from_string("cell1");
    let cell = Cell::new(local_node.clone(), cell_id);

    let transport_config = exocore_transport::lp2p::Config::default();
    let mut transport =
        exocore_transport::lp2p::Libp2pTransport::new(local_node.clone(), transport_config);

    let data_transport = transport.get_handle(cell.clone(), TransportLayer::Data)?;
    let clock = Clock::new();

    let mut chain_dir = std::env::current_dir()?;
    chain_dir.push("target/data/chain");
    let chain_store =
        DirectoryChainStore::create(DirectoryChainStoreConfig::default(), &chain_dir)?;
    let pending_store = MemoryPendingStore::new();

    let engine_config = exocore_data::EngineConfig::default();
    let mut engine = exocore_data::Engine::new(
        engine_config,
        local_node.id().clone(),
        clock,
        data_transport,
        chain_store,
        pending_store,
        cell.nodes().clone(),
    );

    let _engine_handle = engine.get_handle();

    rt.spawn(transport);
    rt.block_on(engine.map_err(|_| ())).unwrap();

    Ok(())
}

extern crate log4rs;

pub fn setup_logging() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Root};

    let stdout = ConsoleAppender::builder().build();

    // see https://docs.rs/log4rs/*/log4rs/
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
