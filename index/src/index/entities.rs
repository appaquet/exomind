use super::traits::{
    IndexMutation, PutTraitMutation, PutTraitTombstone, TraitResult, TraitsIndex, TraitsIndexConfig,
};
use crate::domain::entity::{Entity, EntityId, EntityIdRef};
use crate::domain::schema;
use crate::domain::schema::Schema;
use crate::error::Error;
use crate::mutation::Mutation;
use crate::query::*;
use crate::results::{EntitiesResults, EntityResult, EntityResultSource};
use exocore_data::block::{BlockHeight, BlockOffset};
use exocore_data::engine::{EngineOperation, Event};
use exocore_data::operation::{Operation, OperationId};
use exocore_data::{chain, pending};
use exocore_data::{EngineHandle, EngineOperationStatus};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

///
/// Configuration of the entities index
///
#[derive(Clone, Copy, Debug)]
pub struct EntitiesIndexConfig {
    /// When should we index a block in the chain so that odds that we aren't going to revert it are high enough.
    /// Related to `CommitManagerConfig`.`operations_cleanup_after_block_depth`
    pub chain_index_min_depth: BlockHeight,

    /// Configuration for the in-memory traits index that are in the pending store
    pub pending_index_config: TraitsIndexConfig,

    /// Configuration for the persisted traits index that are in the chain
    pub chain_index_config: TraitsIndexConfig,
}

impl Default for EntitiesIndexConfig {
    fn default() -> Self {
        EntitiesIndexConfig {
            chain_index_min_depth: 3,
            pending_index_config: TraitsIndexConfig::default(),
            chain_index_config: TraitsIndexConfig::default(),
        }
    }
}

///
/// Manages and index entities and their traits stored in the chain and pending store of the data
/// layer. The index accepts mutation from the data layer through its event stream, and managed
/// both indices to be consistent.
///
/// The chain index is persisted on disk, while the pending store is an in-memory index. Since the
/// persistence in the chain is not definitive until blocks and their operations (traits mutations)
/// are stored at a certain depth, a part of the chain is actually indexed in the in-memory index.
/// Once they reach a certain depth, they are persisted in the chain index.
///
pub struct EntitiesIndex<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    config: EntitiesIndexConfig,
    pending_index: TraitsIndex,
    chain_index_dir: PathBuf,
    chain_index: TraitsIndex,

    schema: Arc<Schema>,

    phantom: std::marker::PhantomData<(CS, PS)>,
}

impl<CS, PS> EntitiesIndex<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    /// Opens or create an entities index
    pub fn open_or_create(
        data_dir: &Path,
        config: EntitiesIndexConfig,
        schema: Arc<schema::Schema>,
    ) -> Result<EntitiesIndex<CS, PS>, Error> {
        let pending_index =
            TraitsIndex::create_in_memory(config.pending_index_config, schema.clone())?;

        // make sure directories are created
        let mut chain_index_dir = data_dir.to_path_buf();
        chain_index_dir.push("chain");
        if std::fs::metadata(&chain_index_dir).is_err() {
            std::fs::create_dir_all(&chain_index_dir)?;
        }

        let chain_index = TraitsIndex::open_or_create_mmap(
            config.chain_index_config,
            schema.clone(),
            &chain_index_dir,
        )?;

        Ok(EntitiesIndex {
            config,
            pending_index,
            chain_index_dir,
            chain_index,
            schema,
            phantom: std::marker::PhantomData,
        })
    }

    /// Writes a trait mutation to the data layer. It's not indexed right away, since we
    /// will wait for the event to bubble back up to preserve consistency.
    pub fn write_mutation(
        &mut self,
        data_handle: &EngineHandle<CS, PS>,
        mutation: Mutation,
    ) -> Result<OperationId, Error> {
        let json_mutation = mutation.to_json(self.schema.clone())?;
        let op_id = data_handle.write_entry_operation(json_mutation.as_bytes())?;
        Ok(op_id)
    }

    /// Handle an event coming from the data layer. These events allow keeping the index
    /// consistent with the data layer, up to the consistency guarantees that the layer offers.
    ///
    /// Since the events stream is buffered, we may receive a discontinuity if the data layer
    /// couldn't send us an event. In that case, we re-index the pending index since we can't
    /// guarantee that we didn't lose an event.
    pub fn handle_engine_event(
        &mut self,
        data_handle: &EngineHandle<CS, PS>,
        event: Event,
    ) -> Result<(), Error> {
        match event {
            Event::Started => {
                info!("Data engine is ready, indexing pending store & chain");
                self.index_chain_new_blocks(data_handle)?;
                self.reindex_pending(data_handle)?;
            }
            Event::StreamDiscontinuity => {
                warn!("Got a stream discontinuity. Forcing re-indexation of pending...");
                self.reindex_pending(data_handle)?;
            }
            Event::NewPendingOperation(op_id) => {
                self.handle_engine_event_new_pending_operation(data_handle, op_id)?;
            }
            Event::NewChainBlock(block_offset) => {
                debug!(
                    "Got new block at offset {}, checking if we can index a new block",
                    block_offset
                );
                self.index_chain_new_blocks(data_handle)?;
            }
            Event::ChainDiverged(diverged_block_offset) => {
                let highest_indexed_block = self.chain_index.highest_indexed_block()?;
                warn!(
                    "Chain has diverged at offset={}. Highest indexed block at = {:?}",
                    diverged_block_offset, highest_indexed_block
                );

                if let Some(last_indexed_offset) = highest_indexed_block {
                    if last_indexed_offset < diverged_block_offset {
                        // since we only index blocks that have a certain depth, and therefor higher
                        // probability of being definitive, if we have a divergence, we can just re-index
                        // the pending store which should still contain operations that are in our invalid
                        // chain
                        warn!("Divergence is after last indexed offset, we only re-index pending");
                        self.reindex_pending(data_handle)?;
                    } else {
                        // if we are here, we indexed a block from the chain that isn't valid anymore
                        // since we are deleting traits that got deleted from the actual index, there is no
                        // way to rollback to the diverged offset, and will require a re-index.
                        // this can be prevented by tweaking the `EntitiesIndexConfig`.`chain_index_min_depth` value
                        return Err(Error::Fatal(
                            format!("Chain has diverged at an offset={}, which is before last indexed block at offset {}",
                                                        diverged_block_offset,last_indexed_offset
                        )));
                    }
                } else {
                    warn!("Diverged with an empty chain index. Re-indexing...");
                    self.reindex_chain(data_handle)?;
                }
            }
        }

        Ok(())
    }

    /// Execute a search query on the indices, and returning all entities matching the query.
    pub fn search(
        &self,
        data_handle: &EngineHandle<CS, PS>,
        query: &Query,
    ) -> Result<EntitiesResults, Error> {
        // TODO: Implement paging, counting, sorting & limit https://github.com/appaquet/exocore/issues/105

        let chain_results = self
            .chain_index
            .search(query, 50)?
            .into_iter()
            .map(|res| (res, EntityResultSource::Chain));
        let pending_results = self
            .pending_index
            .search(query, 50)?
            .into_iter()
            .map(|res| (res, EntityResultSource::Pending));
        debug!(
            "Found {} from chain, {} from pending",
            chain_results.len(),
            pending_results.len()
        );
        let combined_results = chain_results.chain(pending_results).collect_vec();

        // accumulate traits found for each entities
        let mut entities_traits: HashMap<EntityId, Vec<(&TraitResult, EntityResultSource)>> =
            HashMap::new();
        for (trait_result, source) in &combined_results {
            if let Some(traits) = entities_traits.get_mut(&trait_result.entity_id) {
                traits.push((trait_result, *source));
            } else {
                entities_traits.insert(
                    trait_result.entity_id.clone(),
                    vec![(trait_result, *source)],
                );
            }
        }

        // iterate through results and returning the first N entities
        let mut included_entities = HashSet::new();
        let results = combined_results
            .iter()
            .flat_map(|(trait_result, source)| {
                if included_entities.contains(&trait_result.entity_id) {
                    return None;
                }

                let entity = self
                    .fetch_entity(data_handle, &trait_result.entity_id)
                    .ok()?;
                included_entities.insert(trait_result.entity_id.clone());

                Some(EntityResult {
                    entity,
                    source: *source,
                })
            })
            .take(100)
            .collect();

        Ok(EntitiesResults {
            results,
            next_page: None,
            current_page: QueryPaging {
                // TODO: https://github.com/appaquet/exocore/issues/105
                count: 0,
                from_token: None,
                to_token: None,
            },
            total_estimated: combined_results.len() as u32, // TODO: https://github.com/appaquet/exocore/issues/105
        })
    }

    /// Fetch an entity and all its traits from indices and the data layer. Traits returned
    /// follow mutations in order of operation id.
    fn fetch_entity(
        &self,
        data_handle: &EngineHandle<CS, PS>,
        entity_id: &EntityIdRef,
    ) -> Result<Entity, Error> {
        let entity_query = Query::with_entity_id(entity_id);

        let pending_results = self.pending_index.search(&entity_query, 9999)?;
        let chain_results = self.chain_index.search(&entity_query, 9999)?;
        let ordered_traits = pending_results
            .into_iter()
            .chain(chain_results.into_iter())
            .sorted_by_key(|result| result.operation_id);

        // only keep last operation for each trait, and remove trait if it's a tombstone
        let mut traits: HashMap<EntityId, TraitResult> = HashMap::new();
        for trait_result in ordered_traits {
            if trait_result.tombstone {
                traits.remove(&trait_result.trait_id);
            }
            traits.insert(trait_result.trait_id.clone(), trait_result);
        }

        // for remaining traits, fetch operations from data layer
        let final_traits = traits
            .values()
            .flat_map(|trait_result| {
                let mutation = match self.fetch_trait_mutation_operation(
                    data_handle,
                    trait_result.operation_id,
                    trait_result.block_offset,
                ) {
                    Ok(Some(mutation)) => mutation,
                    other => {
                        error!(
                        "Couldn't fetch trait trait_id={} operation_id={} for entity_id={}: {:?}",
                        trait_result.trait_id,
                        trait_result.operation_id,
                        trait_result.entity_id,
                        other
                    );
                        return None;
                    }
                };

                match mutation {
                    Mutation::PutTrait(trait_put) => Some(trait_put.trt),
                    Mutation::DeleteTrait(_) => None,
                }
            })
            .collect();

        Ok(Entity {
            id: entity_id.to_string(),
            traits: final_traits,
        })
    }

    /// Fetch an operation from the data layer by the given operation id and optional block
    /// offset.
    fn fetch_trait_mutation_operation(
        &self,
        data_handle: &EngineHandle<CS, PS>,
        operation_id: OperationId,
        block_offset: Option<BlockOffset>,
    ) -> Result<Option<Mutation>, Error> {
        let operation = if let Some(block_offset) = block_offset {
            data_handle.get_chain_operation(block_offset, operation_id)?
        } else {
            data_handle.get_operation(operation_id)?
        };

        let operation = if let Some(operation) = operation {
            operation
        } else {
            return Ok(None);
        };

        if let Ok(data) = operation.as_entry_data() {
            let mutation = Mutation::from_json_slice(self.schema.clone(), data)?;
            Ok(Some(mutation))
        } else {
            Ok(None)
        }
    }

    /// Reindexes the pending store completely, along the last few blocks of the chain
    /// (see `EntitiesIndexConfig`.`chain_index_min_depth`) that are not considered definitive yet.
    fn reindex_pending(&mut self, data_handle: &EngineHandle<CS, PS>) -> Result<(), Error> {
        info!("Clearing & reindexing pending index");

        self.pending_index =
            TraitsIndex::create_in_memory(self.config.pending_index_config, self.schema.clone())?;

        // create an iterator over operations from chain (if any) and pending store
        let pending_iter = data_handle.get_pending_operations(..)?.into_iter();
        let pending_and_chain_iter: Box<dyn Iterator<Item = EngineOperation>> =
            if let Some((last_indexed_offset, _last_indexed_height)) =
                self.last_chain_indexed_block(data_handle)?
            {
                // filter pending to exclude operations that are now in the chain index
                let pending_iter =
                    pending_iter.filter(move |op| op.status == EngineOperationStatus::Pending);

                // take operations from chain that have not been indexed to the chain index yet
                let chain_iter = data_handle
                    .get_chain_operations(Some(last_indexed_offset))
                    .filter(move |op| {
                        if let EngineOperationStatus::Committed(offset, _height) = op.status {
                            offset > last_indexed_offset
                        } else {
                            false
                        }
                    });

                Box::new(chain_iter.chain(pending_iter))
            } else {
                Box::new(pending_iter)
            };

        let schema = self.schema.clone();
        let mutations_iter = pending_and_chain_iter
            .flat_map(|op| Self::chain_operation_to_index_mutation(schema.clone(), op));
        self.pending_index.apply_mutations(mutations_iter)?;

        Ok(())
    }

    /// Reindexes the chain index completely
    fn reindex_chain(&mut self, data_handle: &EngineHandle<CS, PS>) -> Result<(), Error> {
        info!("Clearing & reindexing chain index");

        // create temporary in-memory to wipe directory
        self.chain_index =
            TraitsIndex::create_in_memory(self.config.pending_index_config, self.schema.clone())?;

        // remove and re-create data dir
        std::fs::remove_dir_all(&self.chain_index_dir)?;
        std::fs::create_dir_all(&self.chain_index_dir)?;

        // re-create index, and force re-index of chain
        self.chain_index = TraitsIndex::open_or_create_mmap(
            self.config.chain_index_config,
            self.schema.clone(),
            &self.chain_index_dir,
        )?;
        self.index_chain_new_blocks(data_handle)?;

        self.reindex_pending(data_handle)?;

        Ok(())
    }

    /// Check if we need to index any new block in the chain.
    /// Blocks don't get indexed as soon as they appear in the chain so that we don't
    /// need to revert them from the chain index since their wouldn't be "easy" way to revert
    /// them from the chain index (Tantivy don't support deletion revert).
    ///
    /// The latest blocks that aren't considered definitive are kept in the pending store, and
    /// deletion are actually implemented using tombstone in the pending store. If a trait gets
    /// deleted from the chain, the tombstone in the in-memory will be used to remove it from
    /// the results.
    fn index_chain_new_blocks(&mut self, data_handle: &EngineHandle<CS, PS>) -> Result<(), Error> {
        let (_last_chain_block_offset, last_chain_block_height) =
            data_handle.get_chain_last_block()?.ok_or_else(|| {
                Error::Other("Tried to index chain, but it had no blocks in it".to_string())
            })?;

        let last_indexed_block = self.last_chain_indexed_block(&data_handle)?;
        if let Some((_last_indexed_offset, last_indexed_height)) = last_indexed_block {
            if last_chain_block_height - last_indexed_height < self.config.chain_index_min_depth {
                debug!("No new blocks to index from chain. last_chain_block_height={} last_indexed_block_height={}",
                       last_chain_block_height, last_indexed_height
                );
                return Ok(());
            }
        }

        let offset_from = last_indexed_block.map(|(offset, _height)| offset);
        let operations = data_handle.get_chain_operations(offset_from);

        let mut pending_index_mutations = Vec::new();

        let chain_index_min_height = self.config.chain_index_min_depth;
        let schema = self.schema.clone();
        let chain_index_mutations = operations
            .flat_map(|op| {
                if let EngineOperationStatus::Committed(offset, height) = op.status {
                    Some((offset, height, op))
                } else {
                    None
                }
            })
            .filter(|(offset, height, _op)| {
                *offset > offset_from.unwrap_or(0)
                    && last_chain_block_height - *height >= chain_index_min_height
            })
            .flat_map(|(offset, height, op)| {
                if let Ok(data) = op.as_entry_data() {
                    let mutation = Mutation::from_json_slice(schema.clone(), data).ok()?;
                    Some((offset, height, op, mutation))
                } else {
                    None
                }
            })
            .map(|(offset, _height, op, mutation)| {
                // for every mutation we index in the chain index, we delete it from the pending index
                pending_index_mutations.push(IndexMutation::DeleteOperation(op.operation_id));

                match mutation {
                    Mutation::PutTrait(trt_mut) => IndexMutation::PutTrait(PutTraitMutation {
                        block_offset: Some(offset),
                        operation_id: op.operation_id,
                        entity_id: trt_mut.entity_id,
                        trt: trt_mut.trt,
                    }),
                    Mutation::DeleteTrait(trt_del) => {
                        IndexMutation::DeleteTrait(trt_del.entity_id, trt_del.trait_id)
                    }
                }
            });

        self.chain_index.apply_mutations(chain_index_mutations)?;
        info!(
            "Indexed {} new operations to chain",
            pending_index_mutations.len()
        );
        self.pending_index
            .apply_mutations(pending_index_mutations.into_iter())?;

        Ok(())
    }

    /// Get last block that got indexed in the chain index
    fn last_chain_indexed_block(
        &self,
        data_handle: &EngineHandle<CS, PS>,
    ) -> Result<Option<(BlockOffset, BlockHeight)>, Error> {
        let last_indexed_offset = self.chain_index.highest_indexed_block()?;

        Ok(last_indexed_offset
            .and_then(|offset| data_handle.get_chain_block_info(offset).ok())
            .and_then(|opt| opt))
    }

    /// Handle a new pending store operation event from the data layer by indexing it
    /// into the pending index
    fn handle_engine_event_new_pending_operation(
        &mut self,
        data_handle: &EngineHandle<CS, PS>,
        operation_id: OperationId,
    ) -> Result<(), Error> {
        let schema = self.schema.clone();
        let operation = data_handle
            .get_pending_operation(operation_id)?
            .expect("Couldn't find operation in data layer for an event we received");
        if let Some(mutation) = Self::chain_operation_to_index_mutation(schema, operation) {
            self.pending_index.apply_mutation(mutation)?;
        }

        Ok(())
    }

    /// Converts a operations from the data layer (chain or pending) into
    /// the trait mutation
    fn chain_operation_to_index_mutation(
        schema: Arc<Schema>,
        operation: EngineOperation,
    ) -> Option<IndexMutation> {
        if let Ok(data) = operation.as_entry_data() {
            let mutation = Mutation::from_json_slice(schema, data).ok()?;
            match mutation {
                Mutation::PutTrait(mutation) => Some(IndexMutation::PutTrait(PutTraitMutation {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: mutation.entity_id,
                    trt: mutation.trt,
                })),
                Mutation::DeleteTrait(mutation) => {
                    Some(IndexMutation::PutTraitTombstone(PutTraitTombstone {
                        block_offset: None,
                        operation_id: operation.operation_id,
                        entity_id: mutation.entity_id,
                        trait_id: mutation.trait_id,
                    }))
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Record, Trait, TraitId};
    use crate::domain::schema::tests::create_test_schema;
    use crate::mutation::{DeleteTraitMutation, PutTraitMutation};
    use exocore_data::tests_utils::DataTestCluster;
    use exocore_data::{DirectoryChainStore, MemoryPendingStore};

    #[test]
    fn index_full_pending_to_chain() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let mut cluster = create_test_cluster()?;

        let config = EntitiesIndexConfig {
            chain_index_min_depth: 1, // index when block is at depth 1 or more
            ..create_test_config()
        };
        let temp = tempdir::TempDir::new("entities_index")?;
        let mut index = EntitiesIndex::open_or_create(temp.path(), config, schema.clone())?;
        handle_engine_events(&mut cluster, &mut index)?;

        // index a few traits, they should now be available from pending index
        put_traits(&schema, &mut cluster, &mut index, 0..=9)?;
        handle_engine_events(&mut cluster, &mut index)?;
        let res = index.search(cluster.get_handle(0), &Query::with_trait("contact"))?;
        let pending_res = count_results_source(&res, EntityResultSource::Pending);
        let chain_res = count_results_source(&res, EntityResultSource::Chain);
        assert_eq!(pending_res, 10);
        assert_eq!(chain_res, 0);

        // index a few traits, wait for them to be in a block
        put_traits(&schema, &mut cluster, &mut index, 10..=19)?;
        handle_engine_events(&mut cluster, &mut index)?;
        let res = index.search(cluster.get_handle(0), &Query::with_trait("contact"))?;
        let pending_res = count_results_source(&res, EntityResultSource::Pending);
        let chain_res = count_results_source(&res, EntityResultSource::Chain);
        assert_eq!(pending_res, 10);
        assert_eq!(chain_res, 10);

        Ok(())
    }

    #[test]
    fn reopen_chain_index() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let mut cluster = create_test_cluster()?;

        let config = EntitiesIndexConfig {
            chain_index_min_depth: 0, // index as soon as new block appear
            ..create_test_config()
        };
        let temp = tempdir::TempDir::new("entities_index")?;

        {
            // index a few traits & make sure it's in the chain index
            let mut index = EntitiesIndex::open_or_create(temp.path(), config, schema.clone())?;
            put_traits(&schema, &mut cluster, &mut index, 0..=9)?;
            cluster.wait_next_block_commit(0);
            cluster.clear_received_events(0);
            index.reindex_chain(cluster.get_handle(0))?;
        }

        {
            // index a few traits & make sure it's in the chain index
            let mut index = EntitiesIndex::open_or_create(temp.path(), config, schema.clone())?;

            // we notify the index that the engine is ready
            index.handle_engine_event(cluster.get_handle(0), Event::Started)?;

            // traits should still be indexed
            let res = index.search(cluster.get_handle(0), &Query::with_trait("contact"))?;
            assert_eq!(res.results.len(), 10);
        }

        Ok(())
    }

    #[test]
    fn reindex_pending_on_discontinuity() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let mut cluster = create_test_cluster()?;

        let config = create_test_config();
        let temp = tempdir::TempDir::new("entities_index")?;
        let mut index = EntitiesIndex::open_or_create(temp.path(), config, schema.clone())?;

        // index traits without indexing them by clearing events
        put_traits(&schema, &mut cluster, &mut index, 0..=9)?;
        cluster.clear_received_events(0);

        let res = index.search(cluster.get_handle(0), &Query::with_trait("contact"))?;
        assert_eq!(res.results.len(), 0);

        // trigger discontinuity, which should force reindex
        index.handle_engine_event(cluster.get_handle(0), Event::StreamDiscontinuity)?;

        // pending is indexed
        let res = index.search(cluster.get_handle(0), &Query::with_trait("contact"))?;
        assert_eq!(res.results.len(), 10);

        Ok(())
    }

    #[test]
    fn chain_divergence() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let mut cluster = create_test_cluster()?;

        let config = EntitiesIndexConfig {
            chain_index_min_depth: 0, // index as soon as new block appear
            ..create_test_config()
        };
        let temp = tempdir::TempDir::new("entities_index")?;
        let mut index = EntitiesIndex::open_or_create(temp.path(), config, schema.clone())?;

        // create 3 blocks worth of traits
        put_traits(&schema, &mut cluster, &mut index, 0..=4)?;
        cluster.wait_next_block_commit(0);
        put_traits(&schema, &mut cluster, &mut index, 5..=9)?;
        cluster.wait_next_block_commit(0);
        put_traits(&schema, &mut cluster, &mut index, 10..=14)?;
        cluster.wait_next_block_commit(0);
        cluster.clear_received_events(0);

        // divergence without anything in index will trigger re-indexation
        index.handle_engine_event(cluster.get_handle(0), Event::ChainDiverged(0))?;
        let res = index.search(cluster.get_handle(0), &Query::with_trait("contact"))?;
        assert_eq!(res.results.len(), 15);

        // divergence at an offset not indexed yet will just re-index pending
        let (chain_last_offset, _) = cluster.get_handle(0).get_chain_last_block()?.unwrap();
        index.handle_engine_event(
            cluster.get_handle(0),
            Event::ChainDiverged(chain_last_offset),
        )?;
        let res = index.search(cluster.get_handle(0), &Query::with_trait("contact"))?;
        assert_eq!(res.results.len(), 15);

        // divergence at an offset indexed in chain index will fail
        let res = index.handle_engine_event(cluster.get_handle(0), Event::ChainDiverged(0));
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn delete_entity_trait() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let mut cluster = create_test_cluster()?;

        let config = EntitiesIndexConfig {
            chain_index_min_depth: 1, // index in chain as soon as another block is after
            ..create_test_config()
        };
        let temp = tempdir::TempDir::new("entities_index")?;
        let mut index = EntitiesIndex::open_or_create(temp.path(), config, schema.clone())?;

        put_trait(
            &schema,
            &mut cluster,
            &mut index,
            "entity1",
            "trait1",
            "name1",
        )?;
        put_trait(
            &schema,
            &mut cluster,
            &mut index,
            "entity1",
            "trait2",
            "name2",
        )?;
        cluster.wait_next_block_commit(0);
        handle_engine_events(&mut cluster, &mut index)?;

        let entity = index.fetch_entity(cluster.get_handle(0), "entity1")?;
        assert_eq!(entity.traits.len(), 2);

        // delete trait2, this should delete via a tombstone in pending
        delete_trait(&mut cluster, &mut index, "entity1", "trait2")?;
        cluster.wait_next_block_commit(0);
        handle_engine_events(&mut cluster, &mut index)?;
        let entity = index.fetch_entity(cluster.get_handle(0), "entity1")?;
        assert_eq!(entity.traits.len(), 1);

        let pending_res = index
            .pending_index
            .search(&Query::with_entity_id("entity1"), 10)?;
        assert!(pending_res.iter().any(|r| r.tombstone));

        // now bury the deletion under 1 block, which should delete for real the trait
        put_trait(
            &schema,
            &mut cluster,
            &mut index,
            "entity2",
            "trait2",
            "name1",
        )?;
        cluster.wait_next_block_commit(0);
        handle_engine_events(&mut cluster, &mut index)?;

        // tombstone should have been deleted, and only 1 trait left in chain index
        let entity = index.fetch_entity(cluster.get_handle(0), "entity1")?;
        assert_eq!(entity.traits.len(), 1);
        let pending_res = index
            .pending_index
            .search(&Query::with_entity_id("entity1"), 10)?;
        assert!(pending_res.is_empty());
        let chain_res = index
            .chain_index
            .search(&Query::with_entity_id("entity1"), 10)?;
        assert_eq!(chain_res.len(), 1);

        Ok(())
    }

    fn create_test_cluster() -> Result<DataTestCluster, failure::Error> {
        let mut cluster = DataTestCluster::new(1)?;

        cluster.create_node(0)?;
        cluster.create_chain_genesis_block(0);
        cluster.start_engine(0);

        // wait for engine to start
        cluster.collect_events_stream(0);
        cluster.wait_started(0);

        Ok(cluster)
    }

    fn create_test_config() -> EntitiesIndexConfig {
        EntitiesIndexConfig {
            pending_index_config: TraitsIndexConfig {
                indexer_num_threads: Some(1),
                ..TraitsIndexConfig::default()
            },
            chain_index_config: TraitsIndexConfig {
                indexer_num_threads: Some(1),
                ..TraitsIndexConfig::default()
            },
            ..EntitiesIndexConfig::default()
        }
    }

    fn put_traits<R: Iterator<Item = i32>>(
        schema: &Arc<Schema>,
        cluster: &mut DataTestCluster,
        index: &mut EntitiesIndex<DirectoryChainStore, MemoryPendingStore>,
        range: R,
    ) -> Result<(), failure::Error> {
        for i in range {
            put_trait(
                schema,
                cluster,
                index,
                format!("entity{}", i),
                format!("trt{}", i),
                format!("name{}", i),
            )?;
        }
        Ok(())
    }

    fn put_trait<E: Into<EntityId>, T: Into<TraitId>, N: Into<String>>(
        schema: &Arc<Schema>,
        cluster: &mut DataTestCluster,
        index: &mut EntitiesIndex<DirectoryChainStore, MemoryPendingStore>,
        entity_id: E,
        trait_id: T,
        name: N,
    ) -> Result<(), failure::Error> {
        let handle = cluster.get_handle(0);
        let mutation = Mutation::PutTrait(PutTraitMutation {
            entity_id: entity_id.into(),
            trt: Trait::new(schema.clone(), "contact")
                .with_id(trait_id.into())
                .with_value_by_name("name", name.into()),
        });
        index.write_mutation(handle, mutation)?;
        Ok(())
    }

    fn delete_trait<E: Into<EntityId>, T: Into<TraitId>>(
        cluster: &mut DataTestCluster,
        index: &mut EntitiesIndex<DirectoryChainStore, MemoryPendingStore>,
        entity_id: E,
        trait_id: T,
    ) -> Result<(), failure::Error> {
        let handle = cluster.get_handle(0);
        let mutation = Mutation::DeleteTrait(DeleteTraitMutation {
            entity_id: entity_id.into(),
            trait_id: trait_id.into(),
        });
        index.write_mutation(handle, mutation)?;
        Ok(())
    }

    fn handle_engine_events(
        cluster: &mut DataTestCluster,
        index: &mut EntitiesIndex<
            chain::directory::DirectoryChainStore,
            pending::memory::MemoryPendingStore,
        >,
    ) -> Result<(), Error> {
        while let Some(event) = cluster.pop_received_event(0) {
            index.handle_engine_event(cluster.get_handle(0), event)?;
        }

        Ok(())
    }

    fn count_results_source(results: &EntitiesResults, source: EntityResultSource) -> usize {
        results
            .results
            .iter()
            .filter(|r| r.source == source)
            .count()
    }
}
