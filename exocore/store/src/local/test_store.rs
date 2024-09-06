use std::sync::Arc;

use chrono::Utc;
use exocore_chain::{tests_utils::TestChainCluster, DirectoryChainStore, MemoryPendingStore};
use exocore_protos::{
    generated::{
        exocore_store::{EntityQuery, EntityResults, MutationResult, Trait},
        exocore_test::TestMessage,
    },
    prost::{ProstAnyPackMessageExt, ProstDateTimeExt},
    registry::Registry,
    store::TraitDetails,
};
use tempfile::TempDir;

use super::{entity_index::test_index::TestEntityIndex, *};
use crate::{
    local::{mutation_index::MutationIndexConfig, store::StoreHandle, EntityIndexConfig},
    mutation::{MutationBuilder, MutationRequestLike},
    store::Store as StoreTrait,
};

/// Utility to test store
pub struct TestStore {
    pub cluster: TestChainCluster,
    pub registry: Arc<Registry>,

    pub store: Option<Store<DirectoryChainStore, MemoryPendingStore>>,
    pub store_handle: StoreHandle<DirectoryChainStore, MemoryPendingStore>,
    _temp_dir: TempDir,
}

impl TestStore {
    pub async fn new() -> Result<TestStore, anyhow::Error> {
        let store_config = StoreConfig::default();
        let index_config = Self::test_index_config();

        Self::new_with_config(store_config, index_config).await
    }

    pub async fn new_with_config(
        store_config: StoreConfig,
        index_config: EntityIndexConfig,
    ) -> Result<TestStore, anyhow::Error> {
        let cluster = TestChainCluster::new_single_and_start().await?;

        let temp_dir = tempfile::tempdir()?;
        let registry = Arc::new(Registry::new_with_exocore_types());

        let index = EntityIndex::<DirectoryChainStore, MemoryPendingStore>::open_or_create(
            cluster.cells[0].clone(),
            index_config,
            cluster.get_handle(0).clone(),
            cluster.clocks[0].clone(),
        )?;

        let store = Store::new(
            store_config,
            cluster.cells[0].cell().clone(),
            cluster.clocks[0].clone(),
            cluster.get_new_handle(0),
            index,
        )?;
        let store_handle = store.get_handle();

        Ok(TestStore {
            cluster,
            registry,
            store: Some(store),
            store_handle,
            _temp_dir: temp_dir,
        })
    }

    pub fn test_index_config() -> EntityIndexConfig {
        EntityIndexConfig {
            pending_index_config: MutationIndexConfig {
                indexer_num_threads: Some(1),
                ..MutationIndexConfig::default()
            },
            chain_index_config: MutationIndexConfig {
                indexer_num_threads: Some(1),
                ..MutationIndexConfig::default()
            },
            ..TestEntityIndex::test_config()
        }
    }

    pub async fn start_store(&mut self) -> anyhow::Result<()> {
        let store = self.store.take().unwrap();
        tokio::spawn(async move {
            match store.run().await {
                Ok(_) => {}
                Err(err) => error!("Error running store: {}", err),
            }
        });

        self.store_handle.on_start().await;

        Ok(())
    }

    pub async fn mutate<M: Into<MutationRequestLike> + Send>(
        &mut self,
        request: M,
    ) -> Result<MutationResult, anyhow::Error> {
        self.store_handle
            .mutate(request)
            .await
            .map_err(|err| err.into())
    }

    pub async fn query(&mut self, query: EntityQuery) -> Result<EntityResults, anyhow::Error> {
        self.store_handle
            .query(query)
            .await
            .map_err(|err| err.into())
    }

    pub fn create_put_contact_mutation<E: Into<String>, T: Into<String>, N: Into<String>>(
        &self,
        entity_id: E,
        trait_id: T,
        name: N,
    ) -> MutationBuilder {
        MutationBuilder::new().put_trait(
            entity_id,
            Trait {
                id: trait_id.into(),
                message: Some(
                    TestMessage {
                        string1: name.into(),
                        ..Default::default()
                    }
                    .pack_to_any()
                    .unwrap(),
                ),
                creation_date: Some(Utc::now().to_proto_timestamp()),
                modification_date: Some(Utc::now().to_proto_timestamp()),
                deletion_date: None,
                last_operation_id: 10,
                details: TraitDetails::Full.into(),
            },
        )
    }
}
