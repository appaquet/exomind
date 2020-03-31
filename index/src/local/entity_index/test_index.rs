use crate::entity::{EntityId, TraitId};
use crate::error::Error;
use crate::local::mutation_index::MutationIndexConfig;
use crate::local::{EntityIndex, EntityIndexConfig};
use crate::mutation::MutationBuilder;
use exocore_core::protos::generated::exocore_index::{EntityMutation, Trait};
use exocore_core::protos::generated::exocore_test::TestMessage;
use exocore_core::protos::prost::{ProstAnyPackMessageExt, ProstMessageExt};
use exocore_data::engine::Event;
use exocore_data::operation::OperationId;
use exocore_data::tests_utils::DataTestCluster;
use exocore_data::{DirectoryChainStore, MemoryPendingStore};

/// Utility to test entities index
pub struct TestEntityIndex {
    pub config: EntityIndexConfig,
    pub cluster: DataTestCluster,
    pub index: EntityIndex<DirectoryChainStore, MemoryPendingStore>,
}

impl TestEntityIndex {
    pub fn new() -> Result<TestEntityIndex, failure::Error> {
        Self::new_with_config(Self::create_test_config())
    }

    pub fn new_with_config(config: EntityIndexConfig) -> Result<TestEntityIndex, failure::Error> {
        let cluster = DataTestCluster::new_single_and_start()?;

        let data_handle = cluster.get_handle(0).clone();
        let index = EntityIndex::open_or_create(cluster.cells[0].clone(), config, data_handle)?;

        Ok(TestEntityIndex {
            cluster,
            config,
            index,
        })
    }

    pub fn with_restarted_node(self) -> Result<TestEntityIndex, failure::Error> {
        // deconstruct so that we can drop index and close the index properly before
        // reopening
        let TestEntityIndex {
            mut cluster,
            config,
            index,
        } = self;
        drop(index);

        cluster.restart_node(0)?;

        let index = EntityIndex::<DirectoryChainStore, MemoryPendingStore>::open_or_create(
            cluster.cells[0].clone(),
            config,
            cluster.get_handle(0).clone(),
        )?;

        Ok(TestEntityIndex {
            cluster,
            config,
            index,
        })
    }

    pub fn create_test_config() -> EntityIndexConfig {
        EntityIndexConfig {
            chain_index_in_memory: true,
            pending_index_config: MutationIndexConfig {
                indexer_num_threads: Some(1),
                ..MutationIndexConfig::default()
            },
            chain_index_config: MutationIndexConfig {
                indexer_num_threads: Some(1),
                ..MutationIndexConfig::default()
            },
            ..EntityIndexConfig::default()
        }
    }

    pub fn handle_engine_events(&mut self) -> Result<(), Error> {
        let events = self.cluster.drain_received_events(0);
        if !events.is_empty() {
            self.index.handle_data_engine_events(events.into_iter())?;
        }

        Ok(())
    }

    pub fn wait_operations_emitted(&mut self, operations_id: &[OperationId]) {
        self.cluster.wait_operations_emitted(0, operations_id);
    }

    pub fn wait_operation_committed(&mut self, operation_id: OperationId) {
        self.cluster.wait_operation_committed(0, operation_id);
    }

    pub fn wait_operations_committed(&mut self, operations_id: &[OperationId]) {
        self.cluster.wait_operations_committed(0, operations_id);
    }

    pub fn drain_received_events(&mut self) -> Vec<Event> {
        self.cluster.drain_received_events(0)
    }

    pub fn put_test_traits<R: Iterator<Item = i32>>(
        &mut self,
        range: R,
    ) -> Result<Vec<OperationId>, failure::Error> {
        let mut ops_id = Vec::new();
        for i in range {
            let op_id = self.put_test_trait(
                format!("entity{}", i),
                format!("trt{}", i),
                format!("name{} common", i),
            )?;
            ops_id.push(op_id)
        }
        Ok(ops_id)
    }

    pub fn put_test_trait<E: Into<EntityId>, T: Into<TraitId>, N: Into<String>>(
        &mut self,
        entity_id: E,
        trait_id: T,
        name: N,
    ) -> Result<OperationId, failure::Error> {
        let trt = Self::new_test_trait(trait_id, name)?;
        let mutation = MutationBuilder::put_trait(entity_id.into(), trt);
        self.write_mutation(mutation)
    }

    pub fn delete_trait<E: Into<EntityId>, T: Into<TraitId>>(
        &mut self,
        entity_id: E,
        trait_id: T,
    ) -> Result<OperationId, failure::Error> {
        let mutation = MutationBuilder::delete_trait(entity_id.into(), trait_id.into());
        self.write_mutation(mutation)
    }

    pub fn write_mutation(
        &mut self,
        mutation: EntityMutation,
    ) -> Result<OperationId, failure::Error> {
        let buf = mutation.encode_to_vec()?;
        let op_id = self.cluster.get_handle(0).write_entry_operation(&buf)?;
        Ok(op_id)
    }

    pub fn new_test_trait<T: Into<TraitId>, N: Into<String>>(
        trait_id: T,
        name: N,
    ) -> Result<Trait, Error> {
        let trt = Trait {
            id: trait_id.into(),
            message: Some(
                TestMessage {
                    string1: name.into(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        };

        Ok(trt)
    }
}
