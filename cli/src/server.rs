use crate::config::NodeConfig;
use crate::options;
use exocore_common::cell::Cell;
use exocore_common::time::Clock;
use exocore_data::{DirectoryChainStore, DirectoryChainStoreConfig, MemoryPendingStore};
use exocore_transport::transport::TransportHandle;
use exocore_transport::TransportLayer;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::runtime::Runtime;

pub fn start(
    _opts: &options::Options,
    server_opts: &options::ServerOptions,
) -> Result<(), failure::Error> {
    let config = NodeConfig::from_file(&server_opts.config)?;
    let mut rt = Runtime::new()?;

    let local_node = config.create_local_node()?;
    let mut engines_handle = Vec::new();

    // create transport
    let transport_config = exocore_transport::lp2p::Config::default();
    let mut transport =
        exocore_transport::lp2p::Libp2pTransport::new(local_node.clone(), transport_config);

    for cell_config in &config.cells {
        let (_full_cell, cell) = cell_config.create_cell(&local_node)?;
        let clock = Clock::new();


        // make sure data directory exists
        let mut chain_dir = cell_config.data_directory.clone();
        chain_dir.push("chain");
        std::fs::create_dir_all(&chain_dir)?;

        // create chain store
        let chain_store =
            DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;
        let pending_store = MemoryPendingStore::new();

        // create the engine
        let data_transport = transport.get_handle(cell.clone(), TransportLayer::Data)?;
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
        let ws_engine_handle = engine.get_handle();

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

        // start ws server
        if let Some(listen_address) = config.websocket_listen_address {
            start_ws_server(&mut rt, &cell, listen_address, ws_engine_handle)?;
        }
    }

    // start transport
    rt.spawn(transport.map(|_| ()).map_err(|_| ()));

    // wait for runtime to finish all its task
    tokio::run(rt.shutdown_on_idle());

    Ok(())
}

fn start_ws_server(
    rt: &mut Runtime,
    cell: &Cell,
    listen_address: SocketAddr,
    engine_handle: exocore_data::engine::Handle<DirectoryChainStore, MemoryPendingStore>,
) -> Result<(), failure::Error> {
    let config = exocore_transport::ws::Config::default();

    // start transport
    let mut ws_transport = exocore_transport::ws::WebsocketTransport::new(listen_address, config);
    let mut ws_handle = ws_transport.get_handle(cell)?;
    rt.spawn(
        ws_transport
            .map(|_| {
                info!("WebSocket transport has stopped");
            })
            .map_err(|err| {
                error!("WebSocket transport stopped with error: {}", err);
            }),
    );

    // wait for ws transport to start, then schedule stream & handle
    rt.block_on(ws_handle.on_start())?;
    rt.spawn(
        ws_handle
            .get_stream()
            .for_each(move |_in_message| {
                debug!("Got message in WebSocket transport");
                let _ = engine_handle.write_entry_operation(b"hello world");
                Ok(())
            })
            .map_err(|err| error!("Error in stream from transport handle: {}", err)),
    );
    rt.spawn(ws_handle.map(|_| {}).map_err(|_| ()));

    Ok(())
}
