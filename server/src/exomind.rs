use exocore::{
    core::{
        cell::{Cell, LocalNodeConfigExt},
        futures::spawn_future,
        time::Clock,
    },
    protos::{
        core::LocalNodeConfig,
        prost::ProstAnyPackMessageExt,
        store::{Entity, Trait},
    },
    store::{
        mutation::MutationBuilder,
        query::QueryBuilder,
        remote::{Client, ClientHandle},
        store::Store,
    },
    transport::{Libp2pTransport, ServiceType},
};
use exomind_protos::base::{Collection, Snoozed};

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

    pub async fn create_base_entities(&self) -> anyhow::Result<()> {
        let inbox_trait = Trait {
            id: "inbox".to_string(),
            message: Some(
                Collection {
                    name: "Inbox".to_string(),
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        };
        let fav_trait = Trait {
            id: "favorites".to_string(),
            message: Some(
                Collection {
                    name: "Favorites".to_string(),
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        };

        let mutations = MutationBuilder::new()
            .put_trait("inbox", inbox_trait)
            .put_trait("favorites", fav_trait);
        let _ = self.store.mutate(mutations).await?;

        Ok(())
    }

    pub async fn get_snoozed(&self) -> anyhow::Result<Vec<Entity>> {
        let query = QueryBuilder::with_trait::<Snoozed>()
            .count(100)
            .order_by_field("until_date", true)
            .build();

        let results = self.store.query(query).await?;
        let entities = results
            .entities
            .into_iter()
            .flat_map(|res| res.entity)
            .collect();

        Ok(entities)
    }
}
