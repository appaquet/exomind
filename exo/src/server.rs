use exocore_chain::{
    DirectoryChainStore, DirectoryChainStoreConfig, Engine, EngineConfig, EngineHandle,
    MemoryPendingStore,
};
use exocore_core::cell::{Cell, CellNodeRole, EitherCell, FullCell};
use exocore_core::futures::Runtime;
use exocore_core::time::Clock;
use exocore_index::local::{EntityIndex, EntityIndexConfig, Store};
use exocore_index::remote::server::Server;
use exocore_transport::lp2p::Libp2pTransportConfig;
use exocore_transport::{Libp2pTransport, TransportHandle, TransportLayer};

use crate::options;

/// Starts servers based on given command line options
pub fn start(_opts: &options::Options, server_opts: &options::ServerOptions) -> anyhow::Result<()> {
    let config = exocore_core::cell::node_config_from_yaml_file(&server_opts.config)?;
    let (either_cells, local_node) = Cell::new_from_local_node_config(config)?;

    let mut rt = Runtime::new()?;
    let mut engines_handle = Vec::new();

    // create transport
    let transport_config = Libp2pTransportConfig::default();
    let mut transport = Libp2pTransport::new(local_node, transport_config);

    for either_cell in either_cells.iter() {
        let clock = Clock::new();

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
            let chain_transport = transport.get_handle(cell.clone(), TransportLayer::Chain)?;
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
            let index_engine_handle = engine.get_handle();

            // start the engine
            rt.spawn(async move {
                let res = engine.run().await;
                info!("{}: Engine is done: {:?}", cell_name, res);
            });

            // start an local store index if needed
            if cell.local_node_has_role(CellNodeRole::IndexStore) {
                let full_cell = match &either_cell {
                    EitherCell::Full(cell) => cell.as_ref().clone(),
                    _ => {
                        return Err(anyhow!(
                            "Cannot have IndexStore role on cell without keypair",
                        ));
                    }
                };

                let entities_index_config = EntityIndexConfig::default();
                let entities_index = EntityIndex::open_or_create(
                    full_cell.clone(),
                    entities_index_config,
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
                info!(
                    "{}: Local node doesn't have index role. Not starting local store index.",
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
    entities_index: EntityIndex<DirectoryChainStore, MemoryPendingStore>,
) -> anyhow::Result<()> {
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
