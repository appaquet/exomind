use std::sync::Arc;

use tempfile::TempDir;

use exocore_chain::tests_utils::TestChainCluster;
use exocore_chain::{DirectoryChainStore, MemoryPendingStore};

use crate::local::mutation_index::MutationIndexConfig;
use crate::local::store::StoreHandle;
use crate::local::EntityIndexConfig;
use crate::mutation::{MutationBuilder, MutationRequestLike};

use super::*;
use chrono::Utc;
use exocore_core::protos::generated::exocore_index::{
    EntityQuery, EntityResults, MutationResult, Trait,
};
use exocore_core::protos::generated::exocore_test::TestMessage;
use exocore_core::protos::prost::{ProstAnyPackMessageExt, ProstDateTimeExt};
use exocore_core::protos::{index::TraitDetails, registry::Registry};

/// Utility to test store
pub struct TestStore {
    pub cluster: TestChainCluster,
    pub registry: Arc<Registry>,

    pub store: Option<Store<DirectoryChainStore, MemoryPendingStore>>,
    pub store_handle: StoreHandle<DirectoryChainStore, MemoryPendingStore>,
    _temp_dir: TempDir,
}

impl TestStore {
    pub fn new() -> Result<TestStore, anyhow::Error> {
        let cluster = TestChainCluster::new_single_and_start()?;

        let temp_dir = tempfile::tempdir()?;
        let registry = Arc::new(Registry::new_with_exocore_types());

        let index_config = EntityIndexConfig {
            pending_index_config: MutationIndexConfig {
                indexer_num_threads: Some(1),
                ..MutationIndexConfig::default()
            },
            chain_index_config: MutationIndexConfig {
                indexer_num_threads: Some(1),
                ..MutationIndexConfig::default()
            },
            ..EntityIndexConfig::default()
        };
        let index = EntityIndex::<DirectoryChainStore, MemoryPendingStore>::open_or_create(
            cluster.cells[0].clone(),
            index_config,
            cluster.get_handle(0).clone(),
        )?;

        let store = Store::new(
            Default::default(),
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

    pub fn start_store(&mut self) -> anyhow::Result<()> {
        let store = self.store.take().unwrap();
        self.cluster.runtime.spawn(async move {
            match store.run().await {
                Ok(_) => {}
                Err(err) => error!("Error running store: {}", err),
            }
        });

        self.cluster.runtime.block_on(self.store_handle.on_start());

        Ok(())
    }

    pub fn mutate<M: Into<MutationRequestLike>>(
        &mut self,
        request: M,
    ) -> Result<MutationResult, anyhow::Error> {
        self.cluster
            .runtime
            .block_on(self.store_handle.mutate(request))
            .map_err(|err| err.into())
    }

    pub fn query(&mut self, query: EntityQuery) -> Result<EntityResults, anyhow::Error> {
        self.cluster
            .runtime
            .block_on(self.store_handle.query(query))
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
