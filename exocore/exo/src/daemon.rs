use std::pin::Pin;

use exocore_chain::{DirectoryChainStore, Engine, EngineConfig, EngineHandle, MemoryPendingStore};
use exocore_core::{
    cell::{Cell, CellNodeRole, EitherCell, FullCell},
    futures::owned_spawn,
    time::Clock,
};
use exocore_protos::core::LocalNodeConfig;
use exocore_store::{
    local::{EntityIndex, EntityIndexConfig, Store},
    remote::server::Server,
};
use exocore_transport::{
    either::EitherTransportServiceHandle,
    http::{HttpTransportConfig, HttpTransportServer},
    p2p::Libp2pTransportConfig,
    Libp2pTransport, ServiceType, TransportServiceHandle,
};
use futures::{Future, FutureExt};

use crate::Context;

pub async fn cmd_daemon(ctx: &Context) -> anyhow::Result<()> {
    let (local_node, either_cells) = ctx.options.get_node_and_cells();
    let node_config = local_node.config().clone();

    let clock = Clock::new();

    let mut p2p_transport = {
        let p2p_config = Libp2pTransportConfig::default();
        Libp2pTransport::new(local_node.clone(), p2p_config)
    };

    let mut http_transport = {
        let http_config = HttpTransportConfig::default();
        HttpTransportServer::new(local_node, http_config, clock.clone())
    };

    let mut engine_handles = Vec::new();
    let mut services_completion: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();
    let mut http_handles = Vec::new();

    for either_cell in either_cells.iter() {
        let cell = either_cell.cell();
        if cell.local_node_has_role(CellNodeRole::Chain) {
            let cell_name = cell.name().to_string();

            // make sure data directory exists
            let chain_dir = cell
                .chain_directory()
                .as_os_path()
                .expect("Cell is not stored in an OS directory");
            std::fs::create_dir_all(&chain_dir)?;

            // create chain store
            let chain_config = node_config.chain.unwrap_or_default();
            let chain_store = DirectoryChainStore::create_or_open(chain_config.into(), &chain_dir)?;
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
            engine_handles.push(engine_handle);
            let store_engine_handle = engine.get_handle();

            // start the engine
            services_completion.push(
                owned_spawn(async move {
                    let res = engine.run().await;
                    info!("{}: Engine is done: {:?}", cell_name, res);
                })
                .map(|_| ())
                .boxed(),
            );

            // start an local store if needed
            if cell.local_node_has_role(CellNodeRole::Store) {
                let full_cell = match &either_cell {
                    EitherCell::Full(cell) => cell.as_ref().clone(),
                    _ => {
                        return Err(anyhow!("Cannot have store role on cell without keypair",));
                    }
                };

                let entities_index_config: EntityIndexConfig = node_config
                    .store
                    .and_then(|s| s.index)
                    .map(|e| e.into())
                    .unwrap_or_default();

                let entities_index = EntityIndex::open_or_create(
                    full_cell.clone(),
                    entities_index_config,
                    store_engine_handle.clone(),
                    clock.clone(),
                )?;

                // create a combined p2p + http transport for entities store so that it can
                // received mutation / query over http
                let store_p2p_transport =
                    p2p_transport.get_handle(cell.clone(), ServiceType::Store)?;
                let store_http_transport =
                    http_transport.get_handle(cell.clone(), ServiceType::Store)?;
                let store_transport =
                    EitherTransportServiceHandle::new(store_p2p_transport, store_http_transport);

                let (store_handle, store_task) = create_local_store(
                    &node_config,
                    store_transport,
                    store_engine_handle,
                    full_cell,
                    clock.clone(),
                    entities_index,
                )
                .await?;
                services_completion.push(store_task.boxed());

                if cell.local_node_has_role(CellNodeRole::AppHost) {
                    create_app_host(
                        clock.clone(),
                        cell.clone(),
                        store_handle.clone(),
                        &mut services_completion,
                    )
                    .await?;
                }
            } else {
                info!(
                    "{}: Local node doesn't have store role. Not starting local store server.",
                    cell
                );

                http_handles.push(http_transport.get_handle(cell.clone(), ServiceType::None)?);
            }
        } else {
            info!(
                "{}: Local node doesn't have chain role. Not starting chain engine.",
                cell
            );
        }
    }

    // start transports
    let p2p_transport = owned_spawn(async {
        let res = p2p_transport.run().await;
        info!("libp2p transport done: {:?}", res);
    });
    let http_transport = owned_spawn(async {
        let res = http_transport.run().await;
        info!("HTTP transport done: {:?}", res);
    });

    // wait for any services or p2p transport to complete
    futures::select! {
        _ = p2p_transport.fuse() => {},
        _ = http_transport.fuse() => {},
        _ = futures::future::join_all(services_completion).fuse() => {},
    }
    info!("One service is done. Shutting down.");

    Ok(())
}

// Application host can only run on targets supported by wasmtime.
// See https://github.com/bytecodealliance/wasmtime/blob/5c1d728e3ae8ee7aa329e294999a2c3086b21676/docs/stability-platform-support.md
#[cfg(any(
    all(
        target_arch = "x86_64",
        any(target_os = "linux", target_os = "macos", target_os = "windows")
    ),
    all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos"))
))]
async fn create_app_host(
    clock: Clock,
    cell: Cell,
    store_handle: impl exocore_store::store::Store,
    services_completion: &mut Vec<Pin<Box<dyn Future<Output = ()>>>>,
) -> anyhow::Result<()> {
    use exocore_apps_host::{runtime::Applications, Config as ApplicationsConfig};
    use exocore_core::cell::CellNodes;

    // make sure that we are the only node with app host role.
    // TODO: Support for multiple app host nodes https://github.com/appaquet/exocore/issues/619
    {
        let nodes = cell.nodes();
        let app_host_count = nodes.count_with_role(CellNodeRole::AppHost);
        if app_host_count != 1 {
            return Err(anyhow!(
                "{}: Only one node can be an application host",
                cell
            ));
        }
    }

    let apps_config = ApplicationsConfig::default();
    let apps = match Applications::new(apps_config, clock.clone(), cell.clone(), store_handle).await
    {
        Ok(apps) => apps,
        Err(err) => {
            crate::term::print_error(
                "Couldn't start application host. Make sure apps are installed and unpacked.",
            );
            return Err(err.into());
        }
    };

    services_completion.push(
        async move {
            let res = apps.run().await;
            info!(
                "{}: Applications runtime completed with result {:?}",
                cell, res
            );
        }
        .boxed(),
    );

    Ok(())
}

#[cfg(not(any(
    all(
        target_arch = "x86_64",
        any(target_os = "linux", target_os = "macos", target_os = "windows")
    ),
    all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos"))
)))]
async fn create_app_host(
    _clock: Clock,
    _cell: Cell,
    _store_handle: impl exocore_store::store::Store,
    _services_completion: &mut Vec<Pin<Box<dyn Future<Output = ()>>>>,
) -> anyhow::Result<()> {
    Err(anyhow!("Cannot host app on this target."))
}

async fn create_local_store<T: TransportServiceHandle>(
    config: &LocalNodeConfig,
    transport: T,
    chain_handle: EngineHandle<DirectoryChainStore, MemoryPendingStore>,
    full_cell: FullCell,
    clock: Clock,
    entities_index: EntityIndex<DirectoryChainStore, MemoryPendingStore>,
) -> anyhow::Result<(impl exocore_store::store::Store, impl Future<Output = ()>)> {
    let store_config = config.store.map(|c| c.into()).unwrap_or_default();
    let local_store = Store::new(
        store_config,
        full_cell.cell().clone(),
        clock,
        chain_handle,
        entities_index,
    )?;
    let store_handle = local_store.get_handle();

    let local_store_complete = owned_spawn(async move {
        match local_store.run().await {
            Ok(_) => info!("Local store has stopped"),
            Err(err) => error!("Local store has stopped: {}", err),
        }
    });

    store_handle.on_start().await;

    let server_config = Default::default();
    let remote_store_server = Server::new(
        server_config,
        full_cell.cell().clone(),
        store_handle.clone(),
        transport,
    )?;
    let remote_store_complete = owned_spawn(async move {
        match remote_store_server.run().await {
            Ok(_) => info!("Remote store server has stopped"),
            Err(err) => info!("Remote store server has failed: {}", err),
        }
    });

    let store_task = async move {
        futures::select! {
            _ = local_store_complete.fuse() => {},
            _ = remote_store_complete.fuse() => {},
        }
    };

    Ok((store_handle, store_task))
}
