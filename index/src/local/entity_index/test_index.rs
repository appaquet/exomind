use crate::entity::{EntityId, TraitId};
use crate::error::Error;
use crate::local::mutation_index::MutationIndexConfig;
use crate::local::{EntityIndex, EntityIndexConfig};
use crate::mutation::{MutationBuilder, MutationRequestLike};
use exocore_chain::engine::Event;
use exocore_chain::operation::OperationId;
use exocore_chain::tests_utils::TestChainCluster;
use exocore_chain::{DirectoryChainStore, MemoryPendingStore};
use exocore_core::protos::generated::exocore_index::Trait;
use exocore_core::protos::generated::exocore_test::TestMessage;
use exocore_core::protos::prost::{ProstAnyPackMessageExt, ProstMessageExt};

/// Utility to test entities index
pub struct TestEntityIndex {
    pub config: EntityIndexConfig,
    pub cluster: TestChainCluster,
    pub index: EntityIndex<DirectoryChainStore, MemoryPendingStore>,
}

impl TestEntityIndex {
    pub fn new() -> Result<TestEntityIndex, anyhow::Error> {
        Self::new_with_config(Self::create_test_config())
    }

    pub fn new_with_config(config: EntityIndexConfig) -> Result<TestEntityIndex, anyhow::Error> {
        let cluster = TestChainCluster::new_single_and_start()?;

        let data_handle = cluster.get_handle(0).clone();
        let index = EntityIndex::open_or_create(cluster.cells[0].clone(), config, data_handle)?;

        Ok(TestEntityIndex {
            cluster,
            config,
            index,
        })
    }

    pub fn with_restarted_node(self) -> Result<TestEntityIndex, anyhow::Error> {
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
            chain_index_depth_leeway: 0, // for tests, we want to index as soon as possible
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
            self.index.handle_chain_engine_events(events.into_iter())?;
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
    ) -> Result<Vec<OperationId>, anyhow::Error> {
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
    ) -> Result<OperationId, anyhow::Error> {
        let trt = Self::new_test_trait(trait_id, name)?;
        let mutation = MutationBuilder::new().put_trait(entity_id.into(), trt);
        self.write_mutation(mutation)
    }

    pub fn delete_trait<E: Into<EntityId>, T: Into<TraitId>>(
        &mut self,
        entity_id: E,
        trait_id: T,
    ) -> Result<OperationId, anyhow::Error> {
        let mutation = MutationBuilder::new().delete_trait(entity_id.into(), trait_id.into());
        self.write_mutation(mutation)
    }

    pub fn write_mutation<M: Into<MutationRequestLike>>(
        &mut self,
        mutation: M,
    ) -> Result<OperationId, anyhow::Error> {
        let request = mutation.into();
        let buf = request.mutations[0].encode_to_vec()?;
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
