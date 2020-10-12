use exocore_chain::{
    DirectoryChainStore, DirectoryChainStoreConfig, Engine, EngineConfig, EngineHandle,
    MemoryPendingStore,
};
use exocore_core::cell::{Cell, CellNodeRole, EitherCell, FullCell};
use exocore_core::futures::Runtime;
use exocore_core::time::Clock;
use exocore_store::local::{EntityIndex, EntityIndexConfig, Store};
use exocore_store::remote::server::Server;
use exocore_transport::{
    either::EitherTransportServiceHandle, http::HTTPTransportConfig, http::HTTPTransportServer,
    p2p::Libp2pTransportConfig,
};
use exocore_transport::{Libp2pTransport, ServiceType, TransportServiceHandle};

use crate::options;

/// Starts servers based on given command line options
pub fn start(
    _opts: &options::ExoOptions,
    server_opts: &options::ServerOptions,
) -> anyhow::Result<()> {
    let config = exocore_core::cell::node_config_from_yaml_file(&server_opts.config)?;
    let (either_cells, local_node) = Cell::new_from_local_node_config(config)?;

    let mut rt = Runtime::new()?;
    let clock = Clock::new();

    let mut p2p_transport = {
        let p2p_config = Libp2pTransportConfig::default();
        Libp2pTransport::new(local_node.clone(), p2p_config)
    };

    let mut http_transport = {
        let http_config = HTTPTransportConfig::default();
        HTTPTransportServer::new(local_node, http_config, clock.clone())
    };

    let mut engines_handle = Vec::new();
    for either_cell in either_cells.iter() {
        let cell = either_cell.cell();
        if cell.local_node_has_role(CellNodeRole::Chain) {
            let cell_name = cell.name().to_string();

            // make sure data directory exists
            let chain_dir = cell.chain_directory().ok_or_else(|| {
                anyhow!("{}: Cell doesn't have a directory configured", cell_name)
            })?;
            std::fs::create_dir_all(&chain_dir)?;

            // create chain store
            let chain_config = DirectoryChainStoreConfig::default();
            let chain_store = DirectoryChainStore::create_or_open(chain_config, &chain_dir)?;
            let pending_store = MemoryPendingStore::new();

            // create the engine
            let chain_transport = p2p_transport.get_handle(cell.clone(), ServiceType::Chain)?;
            let engine_config = EngineConfig::default();
            let mut engine = Engine::new(
                engine_config,
                clock.clone(),
                chain_transport,
                chain_store,
                pending_store,
                cell.clone(),
            );

            // we keep a handle of the engine, otherwise the engine will not start since it
            // will get dropped
            let engine_handle = engine.get_handle();
            engines_handle.push(engine_handle);
            let store_engine_handle = engine.get_handle();

            // start the engine
            rt.spawn(async move {
                let res = engine.run().await;
                info!("{}: Engine is done: {:?}", cell_name, res);
            });

            // start an local store if needed
            if cell.local_node_has_role(CellNodeRole::Store) {
                let full_cell = match &either_cell {
                    EitherCell::Full(cell) => cell.as_ref().clone(),
                    _ => {
                        return Err(anyhow!("Cannot have store role on cell without keypair",));
                    }
                };

                let entities_index_config = EntityIndexConfig::default();
                let entities_index = EntityIndex::open_or_create(
                    full_cell.clone(),
                    entities_index_config,
                    store_engine_handle.clone(),
                )?;

                // create a combined p2p + http transport for entities store so that it can received mutation / query over http
                let entities_p2p_transport =
                    p2p_transport.get_handle(cell.clone(), ServiceType::Store)?;
                let entities_http_transport =
                    http_transport.get_handle(cell.clone(), ServiceType::Store)?;
                let entities_transport = EitherTransportServiceHandle::new(
                    entities_p2p_transport,
                    entities_http_transport,
                );

                create_local_store(
                    &mut rt,
                    entities_transport,
                    store_engine_handle,
                    full_cell,
                    clock.clone(),
                    entities_index,
                )?;
            } else {
                info!(
                    "{}: Local node doesn't have store role. Not starting local store server.",
                    cell
                )
            }
        } else {
            info!(
                "{}: Local node doesn't have chain role. Not starting chain engine.",
                cell
            )
        }
    }

    // start transports
    rt.spawn(async {
        let res = p2p_transport.run().await;
        info!("libp2p transport done: {:?}", res);
    });

    rt.spawn(async {
        let res = http_transport.run().await;
        info!("HTTP transport done: {:?}", res);
    });

    std::thread::park();

    Ok(())
}

fn create_local_store<T: TransportServiceHandle>(
    rt: &mut Runtime,
    transport: T,
    chain_handle: EngineHandle<DirectoryChainStore, MemoryPendingStore>,
    full_cell: FullCell,
    clock: Clock,
    entities_index: EntityIndex<DirectoryChainStore, MemoryPendingStore>,
) -> anyhow::Result<()> {
    let store_config = Default::default();
    let local_store = Store::new(
        store_config,
        full_cell.cell().clone(),
        clock,
        chain_handle,
        entities_index,
    )?;
    let store_handle = local_store.get_handle();

    rt.spawn(async move {
        match local_store.run().await {
            Ok(_) => info!("Local store has stopped"),
            Err(err) => error!("Local store has stopped: {}", err),
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
