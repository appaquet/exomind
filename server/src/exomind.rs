use exocore::{
    core::{
        cell::{Cell, LocalNodeConfigExt},
        futures::spawn_future,
        time::Clock,
    },
    protos::core::LocalNodeConfig,
    store::remote::{Client, ClientHandle},
    transport::{Libp2pTransport, ServiceType},
};

use crate::cli;

#[derive(Clone)]
pub struct ExomindClient {
    pub store: ClientHandle,
}

impl ExomindClient {
    pub async fn new(config: &cli::Config) -> anyhow::Result<ExomindClient> {
        let config = LocalNodeConfig::from_yaml_file(&config.node_config)?;
        let (cells, local_node) = Cell::from_local_node_config(config)?;
        let either_cell = cells
            .first()
            .ok_or_else(|| anyhow!("Node doesn't have any cell configured"))?;
        let cell = either_cell.cell();

        let clock = Clock::new();

        let mut transport = Libp2pTransport::new(local_node.clone(), Default::default());
        let store_transport = transport.get_handle(cell.clone(), ServiceType::Store)?;

        spawn_future(async move {
            let res = transport.run().await;
            info!("Transport done: {:?}", res);
        });

        let store_client = Client::new(Default::default(), cell.clone(), clock, store_transport)?;
        let store_handle = store_client.get_handle();

        spawn_future(async move {
            let res = store_client.run().await;
            info!("Remote client done: {:?}", res);
        });

        store_handle.on_start().await;

        Ok(ExomindClient {
            store: store_handle,
        })
    }
}
