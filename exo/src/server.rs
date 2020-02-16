use std::sync::Arc;

use failure::err_msg;

use exocore_core::cell::FullCell;
use exocore_core::futures::Runtime;
use exocore_core::protos::registry::Registry;
use exocore_core::time::Clock;
use exocore_data::{
    DirectoryChainStore, DirectoryChainStoreConfig, Engine, EngineConfig, EngineHandle,
    MemoryPendingStore,
};
use exocore_index::store::local::{EntitiesIndex, EntitiesIndexConfig, Store};
use exocore_index::store::remote::server::Server;
use exocore_transport::lp2p::Libp2pTransportConfig;
use exocore_transport::{Libp2pTransport, TransportHandle, TransportLayer};

use crate::config::NodeConfig;
use crate::options;

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
        rt.spawn(async {
            let res = engine.run().await;
            info!("Engine is done: {:?}", res);
        });

        // start an local store index if needed
        if server_opts.index_node {
            let full_cell = opt_full_cell.ok_or_else(|| {
                err_msg("Tried to start a local index, but node doesn't have full cell access (not private key)")
            })?;
            let registry = Arc::new(Registry::new_with_exocore_types());

            let mut index_dir = cell_config.data_directory.clone();
            index_dir.push("index");
            std::fs::create_dir_all(&index_dir)?;

            let entities_index_config = EntitiesIndexConfig::default();
            let entities_index = EntitiesIndex::open_or_create(
                &index_dir,
                entities_index_config,
                registry.clone(),
                index_engine_handle.clone(),
            )?;

            // if we have a WebSocket handle, we create a combined transport
            let transport_handle = transport.get_handle(cell.clone(), TransportLayer::Index)?;
            create_local_store(
                &mut rt,
                transport_handle,
                index_engine_handle,
                full_cell,
                clock,
                entities_index,
            )?;
        } else {
            info!("Local node is not an index node. Not starting local store index.")
        }
    }

    // start transport
    rt.spawn(async {
        let res = transport.run().await;
        info!("Libp2p transport done: {:?}", res);
    });

    std::thread::park();

    Ok(())
}

fn create_local_store<T: TransportHandle>(
    rt: &mut Runtime,
    transport: T,
    index_engine_handle: EngineHandle<DirectoryChainStore, MemoryPendingStore>,
    full_cell: FullCell,
    clock: Clock,
    entities_index: EntitiesIndex<DirectoryChainStore, MemoryPendingStore>,
) -> Result<(), failure::Error> {
    let store_config = Default::default();
    let local_store = Store::new(
        store_config,
        full_cell.cell().clone(),
        clock,
        index_engine_handle,
        entities_index,
    )?;
    let store_handle = local_store.get_handle();

    rt.spawn(async move {
        match local_store.run().await {
            Ok(_) => info!("Local index has stopped"),
            Err(err) => error!("Local index has stopped: {}", err),
        }
    });
    rt.block_on(store_handle.on_start());

    let server_config = Default::default();
    let remote_store_server = Server::new(
        server_config,
        full_cell.cell().clone(),
        store_handle,
        transport,
    )?;
    rt.spawn(async move {
        match remote_store_server.run().await {
            Ok(_) => info!("Remote store server has stopped"),
            Err(err) => info!("Remote store server has failed: {}", err),
        }
    });

    Ok(())
}
