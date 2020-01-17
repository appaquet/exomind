use crate::config::NodeConfig;
use crate::options;
use exocore_common::cell::{Cell, FullCell};
use exocore_common::futures::{Future01, Runtime};
use exocore_common::time::Clock;
use exocore_data::{
    DirectoryChainStore, DirectoryChainStoreConfig, Engine, EngineConfig, EngineHandle,
    MemoryPendingStore,
};
use exocore_index::store::local::{EntitiesIndex, EntitiesIndexConfig, Store};
use exocore_index::store::remote::server::Server;
use exocore_schema::schema::Schema;
use exocore_transport::either::EitherTransportHandle;
use exocore_transport::lp2p::Libp2pTransportConfig;
use exocore_transport::ws::{
    WebSocketTransportConfig, WebsocketTransport, WebsocketTransportHandle,
};
use exocore_transport::{Libp2pTransport, TransportHandle, TransportLayer};
use failure::err_msg;
use std::net::SocketAddr;
use std::sync::Arc;

///
/// Starts servers based on given command line options
///
pub fn start(
    _opts: &options::Options,
    server_opts: &options::ServerOptions,
) -> Result<(), failure::Error> {
    let config = NodeConfig::from_file(&server_opts.config)?;
    let mut rt = Runtime::new()?;

    let local_node = config.create_local_node()?;
    let mut engines_handle = Vec::new();

    // create transport
    let transport_config = Libp2pTransportConfig::default();
    let mut transport = Libp2pTransport::new(local_node.clone(), transport_config);

    for cell_config in &config.cells {
        let (opt_full_cell, cell) = cell_config.create_cell(&local_node)?;
        let clock = Clock::new();

        // make sure data directory exists
        let mut chain_dir = cell_config.data_directory.clone();
        chain_dir.push("chain");
        std::fs::create_dir_all(&chain_dir)?;

        // create chain store
        let chain_config = DirectoryChainStoreConfig::default();
        let chain_store = DirectoryChainStore::create_or_open(chain_config, &chain_dir)?;
        let pending_store = MemoryPendingStore::new();

        // create the engine
        let data_transport = transport.get_handle(cell.clone(), TransportLayer::Data)?;
        let engine_config = EngineConfig::default();
        let mut engine = Engine::new(
            engine_config,
            clock.clone(),
            data_transport,
            chain_store,
            pending_store,
            cell.clone(),
        );

        // we keep a handle of the engine, otherwise the engine will not start since it will get dropped
        let engine_handle = engine.get_handle();
        engines_handle.push(engine_handle);
        let index_engine_handle = engine.get_handle();

        // start the engine
        rt.spawn_std(async {
            let res = engine.run().await;
            info!("Engine is done: {:?}", res);
        });

        // start WebSocket server if needed
        let ws_transport_handle = config.websocket_listen_address.map(|listen_address| {
            info!("Starting a WebSocket transport server");
            start_ws_server(&mut rt, &cell, listen_address)
        });

        // start an local store index if needed
        if server_opts.index_node {
            let full_cell = opt_full_cell.ok_or_else(|| {
                err_msg("Tried to start a local index, but node doesn't have full cell access (not private key)")
            })?;
            let schema = exocore_schema::test_schema::create();

            let mut index_dir = cell_config.data_directory.clone();
            index_dir.push("index");
            std::fs::create_dir_all(&index_dir)?;

            let entities_index_config = EntitiesIndexConfig::default();
            let entities_index = EntitiesIndex::open_or_create(
                &index_dir,
                entities_index_config,
                schema.clone(),
                index_engine_handle.clone(),
            )?;

            // if we have a WebSocket handle, we create a combined transport
            if let Some(ws_transport) = ws_transport_handle {
                let libp2p_handle = transport.get_handle(cell.clone(), TransportLayer::Index)?;
                let combined_transport = EitherTransportHandle::new(libp2p_handle, ws_transport?);
                create_local_store(
                    &mut rt,
                    combined_transport,
                    index_engine_handle,
                    full_cell,
                    clock,
                    schema,
                    entities_index,
                )?;
            } else {
                let transport_handle = transport.get_handle(cell.clone(), TransportLayer::Index)?;
                create_local_store(
                    &mut rt,
                    transport_handle,
                    index_engine_handle,
                    full_cell,
                    clock,
                    schema,
                    entities_index,
                )?;
            };
        } else {
            info!("Local node is not an index node. Not starting local store index.")
        }
    }

    // start transport
    rt.spawn_std(async {
        let res = transport.run().await;
        info!("Libp2p transport done: {:?}", res);
    });

    // wait for runtime to finish all its task
    rt.shutdown_on_idle().wait().unwrap();

    Ok(())
}

fn start_ws_server(
    rt: &mut Runtime,
    cell: &Cell,
    listen_address: SocketAddr,
) -> Result<WebsocketTransportHandle, failure::Error> {
    // start transport
    let config = WebSocketTransportConfig::default();
    let mut transport = WebsocketTransport::new(listen_address, config);
    let handle = transport.get_handle(cell)?;
    rt.spawn_std(async {
        let res = transport.run().await;
        info!("Websocket transport done: {:?}", res);
    });

    Ok(handle)
}

fn create_local_store<T: TransportHandle>(
    rt: &mut Runtime,
    transport: T,
    index_engine_handle: EngineHandle<DirectoryChainStore, MemoryPendingStore>,
    full_cell: FullCell,
    clock: Clock,
    schema: Arc<Schema>,
    entities_index: EntitiesIndex<DirectoryChainStore, MemoryPendingStore>,
) -> Result<(), failure::Error> {
    let store_config = Default::default();
    let local_store = Store::new(
        store_config,
        full_cell.cell().clone(),
        clock,
        schema.clone(),
        index_engine_handle,
        entities_index,
    )?;
    let store_handle = local_store.get_handle();

    rt.spawn_std(async move {
        match local_store.run().await {
            Ok(_) => info!("Local index has stopped"),
            Err(err) => error!("Local index has stopped: {}", err),
        }
    });
    rt.block_on_std(store_handle.on_start());

    let server_config = Default::default();
    let remote_store_server = Server::new(
        server_config,
        full_cell.cell().clone(),
        schema,
        store_handle,
        transport,
    )?;
    rt.spawn_std(async move {
        match remote_store_server.run().await {
            Ok(_) => info!("Remote store server has stopped"),
            Err(err) => info!("Remote store server has failed: {}", err),
        }
    });

    Ok(())
}
