use std::path::Path;

use anyhow::anyhow;
use log::info;

use crate::{
    core::{
        cell::{Cell, LocalNodeConfigExt},
        futures::spawn_future,
        time::Clock,
    },
    protos::core::LocalNodeConfig,
    store::remote::{Client as StoreClient, ClientHandle as StoreHandle},
    transport::{Libp2pTransport, ServiceType},
};

#[derive(Clone)]
pub struct Client {
    pub store: StoreHandle,
}

impl Client {
    pub async fn from_node_config_file<P: AsRef<Path>>(node_config: P) -> anyhow::Result<Self> {
        let config = LocalNodeConfig::from_yaml_file(node_config.as_ref())?;
        Ok(Self::new(config).await?)
    }

    pub async fn new(config: LocalNodeConfig) -> anyhow::Result<Self> {
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

        let store_client =
            StoreClient::new(Default::default(), cell.clone(), clock, store_transport)?;
        let store_handle = store_client.get_handle();

        spawn_future(async move {
            let res = store_client.run().await;
            info!("Remote client done: {:?}", res);
        });

        store_handle.on_start().await;

        Ok(Client {
            store: store_handle,
        })
    }
}
