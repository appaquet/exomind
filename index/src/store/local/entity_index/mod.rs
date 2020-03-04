use std::collections::HashSet;
use std::hash::Hasher;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use itertools::Itertools;
use prost::Message;

use exocore_core::protos::generated::exocore_index::entity_mutation::Mutation;
use exocore_core::protos::generated::exocore_index::{
    Entity, EntityMutation, EntityQuery, EntityResult, EntityResultSource, EntityResults, Paging,
    Trait,
};
use exocore_core::protos::registry::Registry;
use exocore_data::block::{BlockHeight, BlockOffset};
use exocore_data::engine::Event;
use exocore_data::operation::{Operation, OperationId};
use exocore_data::{chain, pending};
use exocore_data::{EngineHandle, EngineOperationStatus};

use super::mutation_index::{IndexMutation, MutationIndex, MutationMetadataType};
use super::top_results::RescoredTopResultsIterable;
use crate::error::Error;
use crate::query::SortToken;
use exocore_core::protos::prost::ProstDateTimeExt;

mod config;
pub use config::*;
mod entity_mutations;
pub use entity_mutations::*;
#[cfg(test)]
mod test_index;

/// Manages and index entities and their traits stored in the chain and pending
/// store of the data layer. The index accepts mutation from the data layer
/// through its event stream, and manages both indices to be consistent.
///
/// The chain index is persisted on disk, while the pending store index is an
/// in-memory index. Since the persistence in the chain is not definitive until
/// blocks and their operations (traits mutations) are stored at a certain
/// depth, a part of the chain is actually indexed in the in-memory index.
/// Once they reach a certain depth, they are persisted in the chain index.
pub struct EntityIndex<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    config: EntityIndexConfig,
    pending_index: MutationIndex,
    chain_index_dir: PathBuf,
    chain_index: MutationIndex,
    chain_index_last_block: Option<BlockOffset>,
    registry: Arc<Registry>,
    data_handle: EngineHandle<CS, PS>,
}

impl<CS, PS> EntityIndex<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    /// Opens or create an entities index
    pub fn open_or_create(
        data_dir: &Path,
        config: EntityIndexConfig,
        registry: Arc<Registry>,
        data_handle: EngineHandle<CS, PS>,
    ) -> Result<EntityIndex<CS, PS>, Error> {
        let pending_index =
            MutationIndex::create_in_memory(config.pending_index_config, registry.clone())?;

        // make sure directories are created
        let mut chain_index_dir = data_dir.to_path_buf();
        chain_index_dir.push("chain");
        if std::fs::metadata(&chain_index_dir).is_err() {
            std::fs::create_dir_all(&chain_index_dir)?;
        }

        let chain_index = Self::create_chain_index(config, &registry, &chain_index_dir)?;

        let mut index = EntityIndex {
            config,
            pending_index,
            chain_index_dir,
            chain_index,
            chain_index_last_block: None,
            registry,
            data_handle,
        };

        index.reindex_pending()?;

        Ok(index)
    }

    pub fn handle_data_engine_event(&mut self, event: Event) -> Result<(), Error> {
        self.handle_data_engine_events(std::iter::once(event))
    }

    /// Handle events coming from the data layer. These events allow keeping the
    /// index consistent with the data layer, up to the consistency
    /// guarantees that the layer offers.

    /// Since the events stream is buffered, we may receive a discontinuity if
    /// the data layer couldn't send us an event. In that case, we re-index
    /// the pending index since we can't guarantee that we didn't lose an
    /// event.
    pub fn handle_data_engine_events<E>(&mut self, events: E) -> Result<(), Error>
    where
        E: Iterator<Item = Event>,
    {
        let mut pending_operations = Vec::new();
        for event in events {
            // We collect new pending operations so that we can apply them in batch. As soon
            // as we hit another kind of events, we apply collected events at
            // once and then continue.
            if let Event::NewPendingOperation(op_id) = event {
                pending_operations.push(op_id);
                continue;
            } else if !pending_operations.is_empty() {
                let current_operations = std::mem::replace(&mut pending_operations, Vec::new());
                self.handle_data_engine_event_pending_operations(current_operations.into_iter())?;
            }

            match event {
                Event::Started => {
                    info!("Data engine is ready, indexing pending store & chain");
                    self.index_chain_new_blocks()?;
                    self.reindex_pending()?;
                }
                Event::StreamDiscontinuity => {
                    warn!("Got a stream discontinuity. Forcing re-indexation of pending...");
                    self.reindex_pending()?;
                }
                Event::NewChainBlock(block_offset) => {
                    debug!(
                        "Got new block at offset {}, checking if we can index a new block",
                        block_offset
                    );
                    self.index_chain_new_blocks()?;
                }
                Event::ChainDiverged(diverged_block_offset) => {
                    let highest_indexed_block = self.chain_index.highest_indexed_block()?;
                    warn!(
                        "Chain has diverged at offset={}. Highest indexed block at = {:?}",
                        diverged_block_offset, highest_indexed_block
                    );

                    if let Some(last_indexed_offset) = highest_indexed_block {
                        if last_indexed_offset < diverged_block_offset {
                            // since we only index blocks that have a certain depth, and therefor
                            // higher probability of being definitive,
                            // if we have a divergence, we can just re-index
                            // the pending store which should still contain operations that are in
                            // our invalid chain
                            warn!(
                                "Divergence is after last indexed offset, we only re-index pending"
                            );
                            self.reindex_pending()?;
                        } else {
                            // if we are here, we indexed a block from the chain that isn't valid
                            // anymore since we are deleting traits that
                            // got deleted from the actual index, there is no
                            // way to rollback to the diverged offset, and will require a re-index.
                            // this can be prevented by tweaking the
                            // `EntitiesIndexConfig`.`chain_index_min_depth` value
                            return Err(Error::Fatal(format!(
                                "Chain has diverged at an offset={}, which is before last indexed block at offset {}",
                                diverged_block_offset, last_indexed_offset
                            )));
                        }
                    } else {
                        warn!("Diverged with an empty chain index. Re-indexing...");
                        self.reindex_chain()?;
                    }
                }
                Event::NewPendingOperation(_op_id) => unreachable!(),
            }
        }

        if !pending_operations.is_empty() {
            self.handle_data_engine_event_pending_operations(pending_operations.into_iter())?;
        }

        Ok(())
    }

    /// Execute a search query on the indices, and returning all entities
    /// matching the query.
    pub fn search(&self, query: &EntityQuery) -> Result<EntityResults, Error> {
        let current_page = query
            .paging
            .clone()
            .unwrap_or_else(crate::query::default_paging);

        let chain_results = self.chain_index.search_all(query)?;
        let pending_results = self.pending_index.search_all(query)?;

        let total_estimated = chain_results.total_results + pending_results.total_results;
        debug!(
            "Found approximately {} from chain, {} from pending, for total of {}",
            chain_results.total_results, pending_results.total_results, total_estimated
        );

        // create merged iterator, returning results from both underlying in order
        let chain_results = chain_results.map(|res| (res, EntityResultSource::Chain));
        let pending_results = pending_results.map(|res| (res, EntityResultSource::Pending));
        let combined_results = chain_results
            .merge_by(pending_results, |(res1, _src1), (res2, _src2)| {
                res1.score >= res2.score
            });

        let mut hasher = result_hasher();

        // iterate through results and returning the first N entities
        let mut matched_entities = HashSet::new();
        let (mut entities_results, traits_results) = combined_results
            // iterate through results, starting with best scores
            .flat_map(|(trait_meta, source)| {
                // check if we already processed this entity through another trait result that
                // ranked higher
                if matched_entities.contains(&trait_meta.entity_id) {
                    return None;
                }

                let traits_meta = self
                    .fetch_entity_traits_metadata(&trait_meta.entity_id)
                    .map_err(|err| {
                        error!(
                            "Error fetching traits for entity_id={} from indices: {}",
                            trait_meta.entity_id, err
                        );
                        err
                    })
                    .ok()?;
                let operation_is_active = traits_meta
                    .active_operations_id
                    .contains(&trait_meta.operation_id);
                if traits_meta.traits.is_empty() || !operation_is_active {
                    // no traits remaining means that entity is now deleted
                    // if current operation is not active anymore, it means that it got overridden
                    // by another operation and should not be considered at this
                    // point as a result
                    return None;
                }

                matched_entities.insert(trait_meta.entity_id.clone());

                // TODO: Support for negative rescoring https://github.com/appaquet/exocore/issues/143
                let score = trait_meta.score;
                let sort_token = SortToken::from_u64(score);
                if sort_token.is_within_page_bound(&current_page) {
                    Some((trait_meta, traits_meta, source, sort_token))
                } else {
                    None
                }
            })
            // this steps consumes the results up until we reach the best 10 results based on the
            // score of the highest matching trait, but re-scored negatively based on
            // other traits
            .top_negatively_rescored_results(
                current_page.count as usize,
                |(trait_result, _traits, _source, _sort_token)| {
                    (trait_result.score, trait_result.score)
                },
            )
            // accumulate results
            .fold(
                (Vec::new(), Vec::new()),
                |(mut entities_results, mut all_traits_results),
                 (trait_result, traits_results, source, sort_token)| {
                    hasher.write_u64(traits_results.hash);
                    let entity = Entity {
                        id: trait_result.entity_id,
                        traits: Vec::new(),
                    };

                    entities_results.push(EntityResult {
                        entity: Some(entity),
                        source: source.into(),
                        sort_token: sort_token.0,
                    });

                    all_traits_results.push(traits_results);

                    (entities_results, all_traits_results)
                },
            );

        let next_page = if let Some(last_result) = entities_results.last() {
            let new_page = Paging {
                before_token: last_result.sort_token.clone(),
                ..current_page.clone()
            };

            Some(new_page)
        } else {
            None
        };

        // TODO: Support for summary https://github.com/appaquet/exocore/issues/142
        let results_hash = hasher.finish();
        let only_summary = query.summary || results_hash == query.result_hash;
        if !only_summary {
            self.fetch_entities_results_full_traits(&mut entities_results, traits_results);
        }

        Ok(EntityResults {
            entities: entities_results,
            summary: only_summary,
            next_page,
            current_page: Some(current_page),
            estimated_count: total_estimated as u32,
            hash: results_hash,
        })
    }

    /// Create the chain index based on configuration.
    fn create_chain_index(
        config: EntityIndexConfig,
        registry: &Arc<Registry>,
        chain_index_dir: &PathBuf,
    ) -> Result<MutationIndex, Error> {
        if !config.chain_index_in_memory {
            MutationIndex::open_or_create_mmap(
                config.chain_index_config,
                registry.clone(),
                &chain_index_dir,
            )
        } else {
            MutationIndex::create_in_memory(config.chain_index_config, registry.clone())
        }
    }

    /// Re-indexes the pending store completely, along the last few blocks of
    /// the chain (see `EntitiesIndexConfig`.`chain_index_min_depth`) that
    /// are not considered definitive yet.
    fn reindex_pending(&mut self) -> Result<(), Error> {
        self.pending_index = MutationIndex::create_in_memory(
            self.config.pending_index_config,
            self.registry.clone(),
        )?;

        let last_chain_indexed_offset = self
            .last_chain_indexed_block()?
            .map(|(offset, _height)| offset)
            .unwrap_or(0);

        info!(
            "Clearing & reindexing pending index. last_chain_indexed_offset={}",
            last_chain_indexed_offset
        );

        // create an iterator over operations in pending store that aren't in chain
        // index
        let pending_iter = self
            .data_handle
            .get_pending_operations(..)?
            .into_iter()
            .filter(|op| match op.status {
                EngineOperationStatus::Pending => true,
                EngineOperationStatus::Committed(offset, _height) => {
                    offset > last_chain_indexed_offset
                }
            });

        // combine pending and chain operations that aren't deemed committed yet
        let pending_and_chain_iter = {
            // filter pending to exclude operations that are now in the chain index
            let pending_iter =
                pending_iter.filter(move |op| op.status == EngineOperationStatus::Pending);

            // take operations from chain that have not been indexed to the chain index yet
            let chain_iter = self
                .data_handle
                .get_chain_operations(Some(last_chain_indexed_offset))
                .filter(move |op| {
                    if let EngineOperationStatus::Committed(offset, _height) = op.status {
                        offset > last_chain_indexed_offset
                    } else {
                        false
                    }
                });

            Box::new(chain_iter.chain(pending_iter))
        };

        let mutations_iter =
            pending_and_chain_iter.flat_map(IndexMutation::from_pending_engine_operation);
        self.pending_index.apply_mutations(mutations_iter)?;

        Ok(())
    }

    /// Re-indexes the chain index completely
    fn reindex_chain(&mut self) -> Result<(), Error> {
        info!("Clearing & reindexing chain index");

        // create temporary in-memory to wipe directory
        self.chain_index = MutationIndex::create_in_memory(
            self.config.pending_index_config,
            self.registry.clone(),
        )?;

        // remove and re-create data dir
        std::fs::remove_dir_all(&self.chain_index_dir)?;
        std::fs::create_dir_all(&self.chain_index_dir)?;

        // re-create index, and force re-index of chain
        self.chain_index =
            Self::create_chain_index(self.config, &self.registry, &self.chain_index_dir)?;
        self.index_chain_new_blocks()?;

        self.reindex_pending()?;

        Ok(())
    }

    /// Check if we need to index any new block in the chain.
    /// Blocks don't get indexed as soon as they appear in the chain so that we
    /// don't need to revert them from the chain index since their wouldn't
    /// be "easy" way to revert them from the chain index (Tantivy don't
    /// support deletion revert).

    /// The latest blocks that aren't considered definitive are kept in the
    /// pending store, and deletion are actually implemented using tombstone
    /// in the pending store. If a trait gets deleted from the chain, the
    /// tombstone in the in-memory will be used to remove it from
    /// the results.
    fn index_chain_new_blocks(&mut self) -> Result<(), Error> {
        let (_last_chain_block_offset, last_chain_block_height) =
            self.data_handle.get_chain_last_block()?.ok_or_else(|| {
                Error::Other("Tried to index chain, but it had no blocks in it".to_string())
            })?;

        let chain_index_min_depth = self.config.chain_index_min_depth;
        let last_indexed_block = self.last_chain_indexed_block()?;
        let offset_from = last_indexed_block.map(|(offset, _height)| offset);
        if let Some((_last_indexed_offset, last_indexed_height)) = last_indexed_block {
            if last_chain_block_height - last_indexed_height < chain_index_min_depth {
                debug!(
                    "No new blocks to index from chain. last_chain_block_height={} last_indexed_block_height={}",
                    last_chain_block_height, last_indexed_height
                );
                return Ok(());
            }
        }

        let mut pending_index_mutations = Vec::new();
        let mut new_highest_block_offset: Option<BlockOffset> = None;

        let operations = self.data_handle.get_chain_operations(offset_from);
        let chain_index_mutations = operations
            .flat_map(|operation| {
                if let EngineOperationStatus::Committed(offset, height) = operation.status {
                    Some((offset, height, operation))
                } else {
                    None
                }
            })
            .filter(|(offset, height, _engine_operation)| {
                // make sure that this operation belongs to the chain index by making sure its
                // depth is below the configured chain_index_min_depth.
                *offset > offset_from.unwrap_or(0)
                    && last_chain_block_height - *height >= chain_index_min_depth
            })
            .flat_map(|(offset, _height, engine_operation)| {
                // for every mutation we index in the chain index, we delete it from the pending
                // index
                pending_index_mutations.push(IndexMutation::DeleteOperation(
                    engine_operation.operation_id,
                ));

                // take note of the latest block that we indexed in chain
                if Some(offset) > new_highest_block_offset {
                    new_highest_block_offset = Some(offset);
                }

                IndexMutation::from_chain_engine_operation(engine_operation, offset)
            });

        self.chain_index.apply_mutations(chain_index_mutations)?;
        info!(
            "Indexed in chain, and deleted from pending {} operations. New chain index last offset is {:?}.",
            pending_index_mutations.len(),
            new_highest_block_offset
        );
        self.pending_index
            .apply_mutations(pending_index_mutations.into_iter())?;

        if let Some(new_highest_block_offset) = new_highest_block_offset {
            self.chain_index_last_block = Some(new_highest_block_offset);
        }

        Ok(())
    }

    /// Get last block that got indexed in the chain index
    fn last_chain_indexed_block(&self) -> Result<Option<(BlockOffset, BlockHeight)>, Error> {
        let mut last_indexed_offset = self.chain_index_last_block;

        if last_indexed_offset.is_none() {
            last_indexed_offset = self.chain_index.highest_indexed_block()?;
        }

        Ok(last_indexed_offset
            .and_then(|offset| self.data_handle.get_chain_block_info(offset).ok())
            .and_then(|opt| opt))
    }

    /// Handle new pending store operations events from the data layer by
    /// indexing them into the pending index.
    fn handle_data_engine_event_pending_operations<O>(
        &mut self,
        operations_id: O,
    ) -> Result<(), Error>
    where
        O: Iterator<Item = OperationId>,
    {
        let mutations = operations_id
            .flat_map(|op_id| match self.data_handle.get_pending_operation(op_id) {
                Ok(Some(op)) => IndexMutation::from_pending_engine_operation(op),
                Ok(None) => {
                    error!(
                        "An event from data layer contained a pending operation that wasn't found: operation_id={}",
                        op_id
                    );
                    smallvec![]
                }
                Err(err) => {
                    error!(
                        "An event from data layer contained that couldn't be fetched from pending operation: {}",
                        err
                    );
                    smallvec![]
                }
            })
            .collect::<Vec<_>>();

        self.pending_index.apply_mutations(mutations.into_iter())
    }

    /// Fetch an entity and all its traits from indices and the data layer.
    /// Traits returned follow mutations in order of operation id.
    #[cfg(test)]
    fn fetch_entity(&self, entity_id: &str) -> Result<Entity, Error> {
        let traits_metadata = self.fetch_entity_traits_metadata(entity_id)?;
        let traits = self.fetch_entity_traits_data(traits_metadata);

        Ok(Entity {
            id: entity_id.to_string(),
            traits,
        })
    }

    /// Fetch indexed traits metadata from pending and chain indices for this
    /// entity id, and merge them.
    fn fetch_entity_traits_metadata(&self, entity_id: &str) -> Result<EntityMutations, Error> {
        let pending_results = self.pending_index.search_entity_id(entity_id)?;
        let chain_results = self.chain_index.search_entity_id(entity_id)?;
        let ordered_traits_metadata = pending_results
            .results
            .into_iter()
            .chain(chain_results.results.into_iter());

        EntityMutations::new(ordered_traits_metadata)
    }

    /// Populate traits in the EntityResult by fetching each entity's traits
    /// from the data layer.
    fn fetch_entities_results_full_traits(
        &self,
        entities_results: &mut Vec<EntityResult>,
        entities_traits_results: Vec<EntityMutations>,
    ) {
        for (entity_result, traits_results) in entities_results
            .iter_mut()
            .zip(entities_traits_results.into_iter())
        {
            let traits = self.fetch_entity_traits_data(traits_results);
            if let Some(entity) = entity_result.entity.as_mut() {
                entity.traits = traits;
            }
        }
    }

    /// Fetch traits data from data layer.
    fn fetch_entity_traits_data(&self, results: EntityMutations) -> Vec<Trait> {
        results
            .traits
            .values()
            .flat_map(|merged_metadata| {
                let mutation = self.fetch_trait_mutation_operation(
                    merged_metadata.operation_id,
                    merged_metadata.block_offset,
                );
                let mutation = match mutation {
                    Ok(Some(mutation)) => mutation,
                    other => {
                        error!(
                            "Couldn't fetch operation_id={} for entity_id={}: {:?}",
                            merged_metadata.operation_id, merged_metadata.entity_id, other
                        );
                        return None;
                    }
                };

                let mut trait_instance = match mutation.mutation? {
                    Mutation::PutTrait(trait_put) => trait_put.r#trait,
                    Mutation::CompactTrait(trait_cmpt) => trait_cmpt.r#trait,
                    Mutation::DeleteTrait(_)
                    | Mutation::DeleteEntity(_)
                    | Mutation::UpdateTrait(_)
                    | Mutation::Test(_) => return None,
                }?;

                // update the trait with creation & modification date that got merged from
                // metadata
                if let MutationMetadataType::TraitPut(put_mut) = &merged_metadata.mutation_type {
                    trait_instance.creation_date =
                        put_mut.creation_date.map(|d| d.to_proto_timestamp());
                    trait_instance.modification_date =
                        put_mut.modification_date.map(|d| d.to_proto_timestamp());
                }

                Some(trait_instance)
            })
            .collect()
    }

    /// Fetch an operation from the data layer by the given operation id and
    /// optional block offset.
    fn fetch_trait_mutation_operation(
        &self,
        operation_id: OperationId,
        block_offset: Option<BlockOffset>,
    ) -> Result<Option<EntityMutation>, Error> {
        let operation = if let Some(block_offset) = block_offset {
            self.data_handle
                .get_chain_operation(block_offset, operation_id)?
        } else {
            self.data_handle.get_operation(operation_id)?
        };

        let operation = if let Some(operation) = operation {
            operation
        } else {
            return Ok(None);
        };

        if let Ok(data) = operation.as_entry_data() {
            let mutation = EntityMutation::decode(data)?;
            Ok(Some(mutation))
        } else {
            Ok(None)
        }
    }
}

fn result_hasher() -> impl std::hash::Hasher {
    crc::crc64::Digest::new(crc::crc64::ECMA)
}

#[cfg(test)]
mod tests {
    use exocore_core::protos::generated::exocore_test::TestMessage;
    use exocore_core::protos::prost::ProstTimestampExt;

    use crate::mutation::MutationBuilder;
    use crate::query::QueryBuilder;

    use super::*;
    use crate::store::local::mutation_index::MutationResults;

    use test_index::*;

    #[test]
    fn index_full_pending_to_chain() -> Result<(), failure::Error> {
        let config = EntityIndexConfig {
            chain_index_min_depth: 1, // index when block is at depth 1 or more
            ..TestEntityIndex::create_test_config()
        };
        let mut test_index = TestEntityIndex::new_with_config(config)?;
        test_index.handle_engine_events()?;

        // index a few traits, they should now be available from pending index
        let first_ops_id = test_index.put_test_traits(0..=4)?;
        test_index.wait_operations_emitted(&first_ops_id);
        test_index.handle_engine_events()?;
        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        let pending_res = count_results_source(&res, EntityResultSource::Pending);
        let chain_res = count_results_source(&res, EntityResultSource::Chain);
        assert_eq!(pending_res + chain_res, 5);

        // index a few traits, wait for first block ot be committed
        let second_ops_id = test_index.put_test_traits(5..=9)?;
        test_index.wait_operations_emitted(&second_ops_id);
        test_index.wait_operations_committed(&first_ops_id);
        test_index.handle_engine_events()?;
        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        let pending_res = count_results_source(&res, EntityResultSource::Pending);
        let chain_res = count_results_source(&res, EntityResultSource::Chain);
        assert_eq!(pending_res + chain_res, 10);

        // wait for second block to be committed, first operations should now be indexed
        // in chain
        test_index.wait_operations_committed(&second_ops_id);
        test_index.handle_engine_events()?;
        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        let pending_res = count_results_source(&res, EntityResultSource::Pending);
        let chain_res = count_results_source(&res, EntityResultSource::Chain);
        assert!(chain_res >= 5, "was equal to {}", chain_res);
        assert_eq!(pending_res + chain_res, 10);

        Ok(())
    }

    #[test]
    fn reopen_chain_index() -> Result<(), failure::Error> {
        let config = EntityIndexConfig {
            chain_index_min_depth: 0, // index as soon as new block appear
            chain_index_in_memory: false,
            ..TestEntityIndex::create_test_config()
        };

        // index a few traits & make sure it's in the chain index
        let mut test_index = TestEntityIndex::new_with_config(config)?;
        let ops_id = test_index.put_test_traits(0..=9)?;
        test_index.wait_operations_committed(&ops_id);
        test_index.drain_received_events();
        test_index.index.reindex_chain()?;

        // reopen index, make sure data is still in there
        let test_index = test_index.with_restarted_node()?;
        // traits should still be indexed
        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        assert_eq!(res.entities.len(), 10);

        Ok(())
    }

    #[test]
    fn reopen_chain_and_pending_transition() -> Result<(), failure::Error> {
        let config = EntityIndexConfig {
            chain_index_min_depth: 2,
            chain_index_in_memory: false,
            ..TestEntityIndex::create_test_config()
        };

        let mut test_index = TestEntityIndex::new_with_config(config)?;
        let query = QueryBuilder::with_trait("exocore.test.TestMessage")
            .with_count(100)
            .build();

        let mut range_from = 0;
        for i in 1..=3 {
            let range_to = range_from + 9;

            let ops_id = test_index.put_test_traits(range_from..=range_to)?;
            test_index.wait_operations_committed(&ops_id);
            test_index.handle_engine_events()?;

            let res = test_index.index.search(&query)?;
            assert_eq!(res.entities.len(), i * 10);

            // restart node, which will clear pending
            // reopening index should re-index first block in pending
            test_index = test_index.with_restarted_node()?;

            // traits should still be indexed
            let res = test_index.index.search(&query)?;
            assert_eq!(res.entities.len(), i * 10);

            range_from = range_to + 1;
        }

        Ok(())
    }

    #[test]
    fn reindex_pending_on_discontinuity() -> Result<(), failure::Error> {
        let mut test_index = TestEntityIndex::new()?;

        // index traits without indexing them by clearing events
        test_index.put_test_traits(0..=5)?;
        test_index.drain_received_events();

        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        assert_eq!(res.entities.len(), 0);

        // trigger discontinuity, which should force reindex
        test_index
            .index
            .handle_data_engine_event(Event::StreamDiscontinuity)?;

        // pending is indexed
        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        assert_eq!(res.entities.len(), 6);

        Ok(())
    }

    #[test]
    fn chain_divergence() -> Result<(), failure::Error> {
        let config = EntityIndexConfig {
            chain_index_min_depth: 0, // index as soon as new block appear
            ..TestEntityIndex::create_test_config()
        };
        let mut test_index = TestEntityIndex::new_with_config(config)?;

        // create 3 blocks worth of traits
        let ops_id = test_index.put_test_traits(0..=2)?;
        test_index.wait_operations_committed(&ops_id);
        let ops_id = test_index.put_test_traits(3..=5)?;
        test_index.wait_operations_committed(&ops_id);
        let ops_id = test_index.put_test_traits(6..=9)?;
        test_index.wait_operations_committed(&ops_id);
        test_index.drain_received_events();

        // divergence without anything in index will trigger re-indexation
        test_index
            .index
            .handle_data_engine_event(Event::ChainDiverged(0))?;
        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        assert_eq!(res.entities.len(), 10);

        // divergence at an offset not indexed yet will just re-index pending
        let (chain_last_offset, _) = test_index
            .cluster
            .get_handle(0)
            .get_chain_last_block()?
            .unwrap();
        test_index
            .index
            .handle_data_engine_event(Event::ChainDiverged(chain_last_offset + 1))?;
        let res = test_index
            .index
            .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
        assert_eq!(res.entities.len(), 10);

        // divergence at an offset indexed in chain index will fail
        let res = test_index
            .index
            .handle_data_engine_event(Event::ChainDiverged(0));
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn delete_entity_trait() -> Result<(), failure::Error> {
        let config = EntityIndexConfig {
            chain_index_min_depth: 1, // index in chain as soon as another block is after
            ..TestEntityIndex::create_test_config()
        };
        let mut test_index = TestEntityIndex::new_with_config(config)?;

        let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
        let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
        test_index.wait_operations_committed(&[op1, op2]);
        test_index.handle_engine_events()?;

        let entity = test_index.index.fetch_entity("entity1")?;
        assert_eq!(entity.traits.len(), 2);

        // delete trait2, this should delete via a tombstone in pending
        let op_id = test_index.delete_trait("entity1", "trait2")?;
        test_index.wait_operation_committed(op_id);
        test_index.handle_engine_events()?;
        let entity = test_index.index.fetch_entity("entity1")?;
        assert_eq!(entity.traits.len(), 1);

        let pending_res = test_index.index.pending_index.search_entity_id("entity1")?;
        assert!(pending_res.results.iter().any(|r| match &r.mutation_type {
            MutationMetadataType::TraitTombstone(_) => true,
            _ => false,
        }));

        // now bury the deletion under 1 block, which should delete for real the trait
        let op_id = test_index.put_test_trait("entity2", "trait2", "name1")?;
        test_index.wait_operation_committed(op_id);
        test_index.handle_engine_events()?;

        Ok(())
    }

    #[test]
    fn delete_all_entity_traits() -> Result<(), failure::Error> {
        let config = TestEntityIndex::create_test_config();
        let mut test_index = TestEntityIndex::new_with_config(config)?;

        let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
        let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
        test_index.wait_operations_committed(&[op1, op2]);
        test_index.handle_engine_events()?;

        let query = QueryBuilder::with_entity_id("entity1").build();
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), 1);

        let op_id = test_index.delete_trait("entity1", "trait1")?;
        test_index.wait_operation_committed(op_id);
        test_index.handle_engine_events()?;

        let query = QueryBuilder::with_entity_id("entity1").build();
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), 1);

        let op_id = test_index.delete_trait("entity1", "trait2")?;
        test_index.wait_operation_committed(op_id);
        test_index.handle_engine_events()?;

        let query = QueryBuilder::with_entity_id("entity1").build();
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), 0);

        Ok(())
    }

    #[test]
    fn delete_entity() -> Result<(), failure::Error> {
        let config = EntityIndexConfig {
            chain_index_min_depth: 1, // index in chain as soon as another block is after
            ..TestEntityIndex::create_test_config()
        };
        let mut test_index = TestEntityIndex::new_with_config(config)?;

        let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
        let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
        test_index.wait_operations_committed(&[op1, op2]);
        test_index.handle_engine_events()?;

        let query = QueryBuilder::with_entity_id("entity1").build();
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), 1);

        let op_id = test_index.write_mutation(MutationBuilder::delete_entity("entity1"))?;
        test_index.wait_operation_committed(op_id);
        test_index.handle_engine_events()?;
        let query = QueryBuilder::with_entity_id("entity1").build();
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), 0);

        // now bury the deletion under 1 block, which should delete for real the trait
        let op_id = test_index.put_test_trait("entity2", "trait2", "name1")?;
        test_index.wait_operation_committed(op_id);
        test_index.handle_engine_events()?;

        // should still be deleted
        let query = QueryBuilder::with_entity_id("entity1").build();
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), 0);

        Ok(())
    }

    #[test]
    fn traits_compaction() -> Result<(), failure::Error> {
        let config = EntityIndexConfig {
            chain_index_min_depth: 1, // index in chain as soon as another block is after
            ..TestEntityIndex::create_test_config()
        };
        let mut test_index = TestEntityIndex::new_with_config(config)?;

        let op1 = test_index.put_test_trait("entity1", "trait1", "op1")?;
        let op2 = test_index.put_test_trait("entity1", "trait1", "op2")?;
        let op3 = test_index.put_test_trait("entity1", "trait1", "op3")?;
        test_index.wait_operations_committed(&[op1, op2, op3]);
        test_index.handle_engine_events()?;

        // we have 3 mutations on same trait
        let pending_res = test_index.index.pending_index.search_entity_id("entity1")?;
        let ops = extract_indexed_operations_id(pending_res);
        assert_eq!(vec![op1, op2, op3], ops);

        // mut entity has only 1 trait since all ops are on same trait
        let query = QueryBuilder::with_entity_id("entity1").build();
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), 1);
        let traits_msgs = extract_result_messages(&res.entities[0]);
        assert_eq!(traits_msgs.len(), 1);

        // last version of trait should have been ket
        assert_eq!("op3", traits_msgs[0].1.string1);

        assert_eq!(
            op1,
            traits_msgs[0]
                .0
                .creation_date
                .as_ref()
                .unwrap()
                .to_timestamp_nanos()
        );
        assert_eq!(
            op3,
            traits_msgs[0]
                .0
                .modification_date
                .as_ref()
                .unwrap()
                .to_timestamp_nanos()
        );

        let new_trait = TestEntityIndex::new_test_trait("trait1", "op4")?;
        let op_id = test_index.write_mutation(MutationBuilder::compact_traits(
            "entity1",
            new_trait,
            vec![op1, op2, op3],
        ))?;
        test_index.wait_operation_committed(op_id);
        test_index.handle_engine_events()?;

        // dates should still be the same even if we compacted the traits
        assert_eq!(
            op1,
            traits_msgs[0]
                .0
                .creation_date
                .as_ref()
                .unwrap()
                .to_timestamp_nanos()
        );
        assert_eq!(
            op3,
            traits_msgs[0]
                .0
                .modification_date
                .as_ref()
                .unwrap()
                .to_timestamp_nanos()
        );

        Ok(())
    }

    #[test]
    fn query_paging() -> Result<(), failure::Error> {
        let config = TestEntityIndex::create_test_config();
        let mut test_index = TestEntityIndex::new_with_config(config)?;

        // add traits in 3 batch so that we have pending & chain items
        let ops_id = test_index.put_test_traits(0..10)?;
        test_index.wait_operations_emitted(&ops_id);
        test_index.handle_engine_events()?;
        test_index.wait_operations_committed(&ops_id[0..10]);

        let ops_id = test_index.put_test_traits(10..20)?;
        test_index.wait_operations_emitted(&ops_id);
        test_index.handle_engine_events()?;

        let ops_id = test_index.put_test_traits(20..30)?;
        test_index.wait_operations_emitted(&ops_id);
        test_index.handle_engine_events()?;

        // first page
        let query_builder = QueryBuilder::with_trait("exocore.test.TestMessage").with_count(10);
        let res = test_index.index.search(&query_builder.clone().build())?;
        let entities_id = extract_results_entities_id(&res);

        // estimated, since it may be in pending and chain store
        assert!(res.estimated_count >= 30);
        assert!(entities_id.contains(&"entity29"));
        assert!(entities_id.contains(&"entity20"));

        // second page
        let query_builder = query_builder.with_paging(res.next_page.unwrap());
        let res = test_index.index.search(&query_builder.clone().build())?;
        let entities_id = extract_results_entities_id(&res);
        assert!(entities_id.contains(&"entity19"));
        assert!(entities_id.contains(&"entity10"));

        // third page
        let query_builder = query_builder.with_paging(res.next_page.unwrap());
        let res = test_index.index.search(&query_builder.clone().build())?;
        let entities_id = extract_results_entities_id(&res);
        assert!(entities_id.contains(&"entity9"));
        assert!(entities_id.contains(&"entity0"));

        // fourth page (empty)
        let query_builder = query_builder.with_paging(res.next_page.unwrap());
        let res = test_index.index.search(&query_builder.clone().build())?;
        assert_eq!(res.entities.len(), 0);
        assert!(res.next_page.is_none());

        // test explicit after token
        let paging = Paging {
            count: 10,
            after_token: SortToken::from_u64(0).into(),
            ..Default::default()
        };
        let query_builder = query_builder.with_paging(paging);
        let res = test_index.index.search(&query_builder.clone().build())?;
        assert_eq!(res.entities.len(), 10);

        let paging = Paging {
            count: 10,
            after_token: SortToken::from_u64(std::u64::MAX).into(),
            ..Default::default()
        };
        let query_builder = query_builder.with_paging(paging);
        let res = test_index.index.search(&query_builder.clone().build())?;
        assert_eq!(res.entities.len(), 0);

        // test explicit before token
        let paging = Paging {
            count: 10,
            before_token: SortToken::from_u64(0).into(),
            ..Default::default()
        };
        let query_builder = query_builder.with_paging(paging);
        let res = test_index.index.search(&query_builder.clone().build())?;
        assert_eq!(res.entities.len(), 0);

        let paging = Paging {
            count: 10,
            before_token: SortToken::from_u64(std::u64::MAX).into(),
            ..Default::default()
        };
        let query_builder = query_builder.with_paging(paging);
        let res = test_index.index.search(&query_builder.build())?;
        assert_eq!(res.entities.len(), 10);

        Ok(())
    }

    #[test]
    fn summary_query() -> Result<(), failure::Error> {
        let config = TestEntityIndex::create_test_config();
        let mut test_index = TestEntityIndex::new_with_config(config)?;

        let op1 = test_index.put_test_trait("entity1", "trait1", "name")?;
        let op2 = test_index.put_test_trait("entity2", "trait1", "name")?;
        test_index.wait_operations_committed(&[op1, op2]);
        test_index.handle_engine_events()?;

        let query = QueryBuilder::match_text("name").only_summary().build();
        let res = test_index.index.search(&query)?;
        assert!(res.summary);
        assert!(res.entities[0].entity.as_ref().unwrap().traits.is_empty());

        let query = QueryBuilder::match_text("name").build();
        let res = test_index.index.search(&query)?;
        assert!(!res.summary);

        let query = QueryBuilder::match_text("name")
            .only_summary_if_equals(res.hash)
            .build();
        let res = test_index.index.search(&query)?;
        assert!(res.summary);

        Ok(())
    }

    fn count_results_source(results: &EntityResults, source: EntityResultSource) -> usize {
        results
            .entities
            .iter()
            .filter(|r| r.source == i32::from(source))
            .count()
    }

    fn extract_results_entities_id(res: &EntityResults) -> Vec<&str> {
        res.entities
            .iter()
            .map(|res| res.entity.as_ref().unwrap().id.as_str())
            .collect_vec()
    }

    fn extract_result_messages(res: &EntityResult) -> Vec<(Trait, TestMessage)> {
        let traits = res.entity.as_ref().unwrap().traits.clone();
        traits
            .into_iter()
            .map(|trt| {
                let msg =
                    TestMessage::decode(trt.message.as_ref().unwrap().value.as_slice()).unwrap();
                (trt, msg)
            })
            .collect()
    }

    fn extract_indexed_operations_id(res: MutationResults) -> Vec<OperationId> {
        res.results
            .iter()
            .map(|r| r.operation_id)
            .unique()
            .collect()
    }
}
