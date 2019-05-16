#![deny(bare_trait_objects)]

mod config;
mod logging;
mod options;

#[macro_use]
extern crate log;

use config::NodeConfig;
use exocore_common::crypto::keys::Keypair;
use exocore_common::time::Clock;
use exocore_data::chain::ChainStore;
use exocore_data::{DirectoryChainStore, DirectoryChainStoreConfig, MemoryPendingStore};
use exocore_transport::TransportLayer;
use futures::prelude::*;
use log::LevelFilter;
use std::str::FromStr;
use structopt::StructOpt;

fn main() -> Result<(), failure::Error> {
    let opt: options::Options = options::Options::from_args();
    logging::setup(Some(LevelFilter::from_str(&opt.logging_level)?));

    use options::{CellCommand, KeysCommand, ServerCommand, SubCommand};
    let result = match &opt.subcommand {
        SubCommand::server(server_opts) => match server_opts.command {
            ServerCommand::start => server_start(&opt, server_opts),
        },
        SubCommand::keys(keys_opts) => match keys_opts.command {
            KeysCommand::generate => keys_generate(&opt, keys_opts),
        },
        SubCommand::cell(cell_opts) => match cell_opts.command {
            CellCommand::create_genesis_block => cell_genesis_block(&opt, cell_opts),
        },
    };

    if let Err(err) = result {
        println!("Error: {}", err);
        println!("Backtrace: {}", err.backtrace());
    }

    Ok(())
}

fn server_start(
    _opts: &options::Options,
    server_opts: &options::ServerOptions,
) -> Result<(), failure::Error> {
    let config = NodeConfig::from_file(&server_opts.config)?;
    let mut rt = tokio::runtime::Runtime::new()?;

    let local_node = config.create_local_node()?;
    let mut engines_handle = Vec::new();

    for cell_config in &config.cells {
        let (_full_cell, cell) = cell_config.create_cell(&local_node)?;
        let clock = Clock::new();

        // create transport
        let transport_config = exocore_transport::lp2p::Config::default();
        let mut transport =
            exocore_transport::lp2p::Libp2pTransport::new(local_node.clone(), transport_config);
        let data_transport = transport.get_handle(cell.clone(), TransportLayer::Data)?;

        // make sure data directory exists
        let mut chain_dir = cell_config.data_directory.clone();
        chain_dir.push("chain");
        std::fs::create_dir_all(&chain_dir)?;

        // create chain store
        let chain_store =
            DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;
        let pending_store = MemoryPendingStore::new();

        // create the engine
        let engine_config = exocore_data::EngineConfig::default();
        let mut engine = exocore_data::Engine::new(
            engine_config,
            clock,
            data_transport,
            chain_store,
            pending_store,
            cell.clone(),
        );

        // we keep a handle of the engine, otherwise the engine will not start since it will get dropped
        let engine_handle = engine.get_handle();
        engines_handle.push(engine_handle);

        // wait for transport to start
        rt.block_on(transport)?;

        // start the engine
        let cell_id1 = cell.id().clone();
        let cell_id2 = cell.id().clone();
        rt.spawn(
            engine
                .map(move |_| {
                    info!("Engine for cell {:?} is done", cell_id1);
                })
                .map_err(move |err| {
                    error!("Engine for cell {} has failed: {}", cell_id2, err);
                }),
        );
    }

    tokio::run(rt.shutdown_on_idle());

    Ok(())
}

fn keys_generate(
    _opt: &options::Options,
    keys_opts: &options::KeysOptions,
) -> Result<(), failure::Error> {
    let keypair = match keys_opts.algorithm {
        options::KeyAlgorithm::Ed25519 => {
            println!("Type: ED25519");
            Keypair::generate_ed25519()
        }
        options::KeyAlgorithm::Rsa => unimplemented!(),
    };

    println!("Keypair: {}", keypair.encode_base58_string());
    println!("Public key: {}", keypair.public().encode_base58_string());

    Ok(())
}

fn cell_genesis_block(
    _opt: &options::Options,
    cell_opts: &options::CellOptions,
) -> Result<(), failure::Error> {
    let config = NodeConfig::from_file(&cell_opts.config)?;
    let local_node = config.create_local_node()?;

    let cell_config = config
        .cells
        .iter()
        .find(|config| config.public_key == cell_opts.public_key)
        .expect("Couldn't find cell with given public key");

    let (full_cell, _cell) = cell_config.create_cell(&local_node)?;
    let full_cell = full_cell.expect("Cannot create genesis block on a non-full cell");

    let mut chain_dir = cell_config.data_directory.clone();
    chain_dir.push("chain");
    std::fs::create_dir_all(&chain_dir)?;

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;
    if chain_store.get_last_block()?.is_some() {
        panic!("Chain is already initialized");
    }

    let genesis_block = exocore_data::block::BlockOwned::new_genesis(&full_cell)?;
    chain_store.write_block(&genesis_block)?;

    Ok(())
}
