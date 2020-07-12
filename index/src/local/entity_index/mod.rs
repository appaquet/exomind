use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use itertools::Itertools;
use prost::Message;

use exocore_chain::block::{BlockHeight, BlockOffset};
use exocore_chain::engine::Event;
use exocore_chain::operation::{Operation, OperationId};
use exocore_chain::{chain, pending};
use exocore_chain::{EngineHandle, EngineOperationStatus};
use exocore_core::cell::FullCell;
use exocore_core::protos::generated::exocore_index::entity_mutation::Mutation;
use exocore_core::protos::generated::exocore_index::{
    Entity, EntityMutation, EntityQuery, EntityResult, EntityResultSource, EntityResults, Trait,
};
use exocore_core::protos::prost::ProstDateTimeExt;
use exocore_core::protos::registry::Registry;

use crate::error::Error;
use crate::ordering::OrderingValueExt;

use super::mutation_index::{IndexOperation, MutationIndex, MutationType};
use super::top_results::RescoredTopResultsIterable;

mod config;
pub use config::*;

mod mutation_aggregator;
pub use mutation_aggregator::*;

#[cfg(test)]
mod test_index;
#[cfg(test)]
mod tests;

/// Manages and index entities and their traits stored in the chain and pending
/// store of the chain layer. The index accepts mutation from the chain layer
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
    cell: FullCell,
    chain_handle: EngineHandle<CS, PS>,
}

impl<CS, PS> EntityIndex<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    /// Opens or create an entities index
    pub fn open_or_create(
        cell: FullCell,
        config: EntityIndexConfig,
        chain_handle: EngineHandle<CS, PS>,
    ) -> Result<EntityIndex<CS, PS>, Error> {
        let pending_index =
            MutationIndex::create_in_memory(config.pending_index_config, cell.schemas().clone())?;

        // make sure directories are created
        let mut chain_index_dir = cell
            .index_directory()
            .ok_or_else(|| Error::Other("Cell doesn't have an path configured".to_string()))?;
        chain_index_dir.push("chain");
        if std::fs::metadata(&chain_index_dir).is_err() {
            std::fs::create_dir_all(&chain_index_dir)?;
        }

        let chain_index = Self::create_chain_index(config, cell.schemas(), &chain_index_dir)?;
        let mut index = EntityIndex {
            config,
            pending_index,
            chain_index_dir,
            chain_index,
            chain_index_last_block: None,
            cell,
            chain_handle,
        };

        index.reindex_pending()?;

        Ok(index)
    }

    pub fn handle_chain_engine_event(&mut self, event: Event) -> Result<Vec<OperationId>, Error> {
        self.handle_chain_engine_events(std::iter::once(event))
    }

    /// Handle events coming from the chain layer. These events allow keeping
    /// the index consistent with the chain layer, up to the consistency
    /// guarantees that the layer offers.

    /// Since the events stream is buffered, we may receive a discontinuity if
    /// the chain layer couldn't send us an event. In that case, we re-index
    /// the pending index since we can't guarantee that we didn't lose an
    /// event.
    pub fn handle_chain_engine_events<E>(&mut self, events: E) -> Result<Vec<OperationId>, Error>
    where
        E: Iterator<Item = Event>,
    {
        let mut indexed_operations = Vec::new();

        let mut pending_operations = Vec::new();
        for event in events {
            // We collect new pending operations so that we can apply them in batch. As soon
            // as we hit another kind of events, we apply collected events at
            // once and then continue.
            if let Event::NewPendingOperation(op_id) = event {
                pending_operations.push(op_id);
                indexed_operations.push(op_id);
                continue;
            } else if !pending_operations.is_empty() {
                let current_operations = std::mem::replace(&mut pending_operations, Vec::new());
                self.handle_chain_engine_event_pending_operations(current_operations.into_iter())?;
            }

            match event {
                Event::Started => {
                    info!("Chain engine is ready, indexing pending store & chain");
                    self.index_chain_new_blocks(Some(&mut indexed_operations))?;
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
                    self.index_chain_new_blocks(Some(&mut indexed_operations))?;
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
            self.handle_chain_engine_event_pending_operations(pending_operations.into_iter())?;
        }

        Ok(indexed_operations)
    }

    /// Execute a search query on the indices, and returning all entities
    /// matching the query.
    pub fn search<Q: Borrow<EntityQuery>>(&self, query: Q) -> Result<EntityResults, Error> {
        let query = query.borrow();
        let query_include_deleted = query.include_deleted;
        let mut current_page = query
            .paging
            .clone()
            .unwrap_or_else(crate::query::default_paging);
        crate::query::validate_paging(&mut current_page);

        // query pending & chain mutations index without original query paging since we
        // need to do our own paging here since we are re-ranking results and
        // that we may have more than one mutation match for each entity.
        let mutations_query = EntityQuery {
            paging: None,
            ..query.clone()
        };
        let chain_results = self.chain_index.search_iter(&mutations_query)?;
        let pending_results = self.pending_index.search_iter(&mutations_query)?;

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
                res1.sort_value >= res2.sort_value
            });

        // iterate through results and returning the first N entities
        let mut hasher = result_hasher();
        let mut mutations_metadata_cache = HashMap::<String, Rc<MutationAggregator>>::new();
        let mut matched_entities = HashSet::new();
        let (mut entities_results, traits_results) = combined_results
            // iterate through results, starting with best scores
            .flat_map(|(trait_meta, source)| {
                // check if we already processed this entity through another trait result that
                // ranked higher
                if matched_entities.contains(&trait_meta.entity_id) {
                    return None;
                }

                // fetch all entity mutations and cache them since we may have multiple hits for
                // same entity for traits that may have been removed since
                let mutations_metadata = if let Some(mutations_metadata) =
                    mutations_metadata_cache.get(&trait_meta.entity_id)
                {
                    mutations_metadata.clone()
                } else {
                    let mutations_metadata = self
                        .fetch_entity_mutations_metadata(&trait_meta.entity_id)
                        .map_err(|err| {
                            error!(
                                "Error fetching mutations for entity_id={} from indices: {}",
                                trait_meta.entity_id, err
                            );
                            err
                        })
                        .ok()?;

                    mutations_metadata_cache
                        .insert(trait_meta.entity_id.clone(), mutations_metadata.clone());

                    mutations_metadata
                };

                let operation_still_present = mutations_metadata
                    .active_operations
                    .contains(&trait_meta.operation_id);
                if (mutations_metadata.traits.is_empty() || !operation_still_present)
                    && !query_include_deleted
                {
                    // no traits remaining means that entity is now deleted
                    // if current operation is not active anymore, it means that it got overridden
                    // by another operation and should not be considered at this
                    // point as a result
                    return None;
                }

                matched_entities.insert(trait_meta.entity_id.clone());

                // TODO: Support for negative rescoring https://github.com/appaquet/exocore/issues/143
                let sort_value = trait_meta.sort_value.clone();
                if sort_value.value.is_within_page_bound(&current_page) {
                    Some((trait_meta, mutations_metadata, source, sort_value))
                } else {
                    None
                }
            })
            // this steps consumes the results up until we reach the best 10 results based on the
            // score of the highest matching trait, but re-scored negatively based on
            // other traits
            .top_negatively_rescored_results(
                current_page.count as usize,
                |(_trait_result, _traits, _source, sort_value)| {
                    (sort_value.clone(), sort_value.clone())
                },
            )
            // accumulate results
            .fold(
                (Vec::new(), Vec::new()),
                |(mut entities_results, mut all_traits_results),
                 (trait_result, traits_results, source, sort_value)| {
                    hasher.write_u64(traits_results.hash);
                    let entity = Entity {
                        id: trait_result.entity_id,
                        traits: Vec::new(),
                    };

                    entities_results.push(EntityResult {
                        entity: Some(entity),
                        source: source.into(),
                        ordering_value: Some(sort_value.value.clone()),
                    });

                    all_traits_results.push(traits_results);

                    (entities_results, all_traits_results)
                },
            );

        let next_page = if let Some(last_result) = entities_results.last() {
            let mut new_page = current_page.clone();

            let ascending = query
                .ordering
                .as_ref()
                .map(|s| s.ascending)
                .unwrap_or(false);
            if !ascending {
                new_page.before_ordering_value = last_result.ordering_value.clone();
            } else {
                new_page.after_ordering_value = last_result.ordering_value.clone();
            }

            Some(new_page)
        } else {
            None
        };

        // TODO: Support for summary https://github.com/appaquet/exocore/issues/142
        let results_hash = hasher.finish();
        let only_summary = query.summary || results_hash == query.result_hash;
        if !only_summary {
            self.fetch_mutations_results_traits(&mut entities_results, traits_results);
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
        schemas: &Arc<Registry>,
        chain_index_dir: &PathBuf,
    ) -> Result<MutationIndex, Error> {
        if !config.chain_index_in_memory {
            MutationIndex::open_or_create_mmap(
                config.chain_index_config,
                schemas.clone(),
                &chain_index_dir,
            )
        } else {
            MutationIndex::create_in_memory(config.chain_index_config, schemas.clone())
        }
    }

    /// Re-indexes the pending store completely, along the last few blocks of
    /// the chain (see `EntitiesIndexConfig`.`chain_index_min_depth`) that
    /// are not considered definitive yet.
    fn reindex_pending(&mut self) -> Result<(), Error> {
        self.pending_index = MutationIndex::create_in_memory(
            self.config.pending_index_config,
            self.cell.schemas().clone(),
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
            .chain_handle
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
                .chain_handle
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
            pending_and_chain_iter.flat_map(IndexOperation::from_pending_engine_operation);
        self.pending_index.apply_operations(mutations_iter)?;

        Ok(())
    }

    /// Re-indexes the chain index completely
    fn reindex_chain(&mut self) -> Result<(), Error> {
        info!("Clearing & reindexing chain index");

        // create temporary in-memory to wipe directory
        self.chain_index = MutationIndex::create_in_memory(
            self.config.pending_index_config,
            self.cell.schemas().clone(),
        )?;

        // remove and re-create data dir
        std::fs::remove_dir_all(&self.chain_index_dir)?;
        std::fs::create_dir_all(&self.chain_index_dir)?;

        // re-create index, and force re-index of chain
        self.chain_index =
            Self::create_chain_index(self.config, self.cell.schemas(), &self.chain_index_dir)?;
        self.index_chain_new_blocks(None)?;

        self.reindex_pending()?;

        Ok(())
    }

    /// Check if we need to index any new block in the chain.
    /// Blocks don't get indexed as soon as they appear in the chain so that we
    /// don't need to revert them from the chain index since their wouldn't
    /// be "easy" way to revert them from the chain index (Tantivy don't
    /// support deletion revert).
    ///
    /// The latest blocks that aren't considered definitive are kept in the
    /// pending store, and deletion are actually implemented using tombstone
    /// in the pending store. If a trait gets deleted from the chain, the
    /// tombstone in the in-memory will be used to remove it from
    /// the results.
    fn index_chain_new_blocks(
        &mut self,
        affected_operations: Option<&mut Vec<OperationId>>,
    ) -> Result<(), Error> {
        let (_last_chain_block_offset, last_chain_block_height) = self
            .chain_handle
            .get_chain_last_block_info()?
            .ok_or_else(|| {
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
        let mut affected_operations_ref = affected_operations;

        let operations = self.chain_handle.get_chain_operations(offset_from);
        let chain_index_mutations = operations
            .flat_map(|operation| {
                if let Some(affected_operations) = affected_operations_ref.as_mut() {
                    affected_operations.push(operation.operation_id);
                }

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
                pending_index_mutations.push(IndexOperation::DeleteOperation(
                    engine_operation.operation_id,
                ));

                // take note of the latest block that we indexed in chain
                if Some(offset) > new_highest_block_offset {
                    new_highest_block_offset = Some(offset);
                }

                IndexOperation::from_chain_engine_operation(engine_operation, offset)
            });

        self.chain_index.apply_operations(chain_index_mutations)?;

        if !pending_index_mutations.is_empty() {
            info!(
                "Indexed in chain, and deleted from pending {} operations. New chain index last offset is {:?}.",
                pending_index_mutations.len(),
                new_highest_block_offset
            );

            self.pending_index
                .apply_operations(pending_index_mutations.into_iter())?;

            if let Some(new_highest_block_offset) = new_highest_block_offset {
                self.chain_index_last_block = Some(new_highest_block_offset);
            }
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
            .and_then(|offset| self.chain_handle.get_chain_block_info(offset).ok())
            .and_then(|opt| opt))
    }

    /// Handle new pending store operations events from the chain layer by
    /// indexing them into the pending index.
    fn handle_chain_engine_event_pending_operations<O>(
        &mut self,
        operations_id: O,
    ) -> Result<(), Error>
    where
        O: Iterator<Item = OperationId>,
    {
        let mutations = operations_id
            .flat_map(|op_id| match self.chain_handle.get_pending_operation(op_id) {
                Ok(Some(op)) => IndexOperation::from_pending_engine_operation(op),
                Ok(None) => {
                    error!(
                        "An event from chain layer contained a pending operation that wasn't found: operation_id={}",
                        op_id
                    );
                    smallvec![]
                }
                Err(err) => {
                    error!(
                        "An event from chain layer contained that couldn't be fetched from pending operation: {}",
                        err
                    );
                    smallvec![]
                }
            })
            .collect::<Vec<_>>();

        self.pending_index.apply_operations(mutations.into_iter())
    }

    /// Fetch an entity and all its traits from indices and the chain layer.
    /// Traits returned follow mutations in order of operation id.
    #[cfg(test)]
    fn fetch_entity(&self, entity_id: &str) -> Result<Entity, Error> {
        let traits_metadata = self.fetch_entity_mutations_metadata(entity_id)?;
        let traits = self.fetch_entity_traits(traits_metadata);

        Ok(Entity {
            id: entity_id.to_string(),
            traits,
        })
    }

    /// Fetch indexed mutations metadata from pending and chain indices for this
    /// entity id, and merge them.
    fn fetch_entity_mutations_metadata(
        &self,
        entity_id: &str,
    ) -> Result<Rc<MutationAggregator>, Error> {
        let pending_results = self.pending_index.fetch_entity_mutations(entity_id)?;
        let chain_results = self.chain_index.fetch_entity_mutations(entity_id)?;
        let ordered_traits_metadata = pending_results
            .mutations
            .into_iter()
            .chain(chain_results.mutations.into_iter());

        MutationAggregator::new(ordered_traits_metadata).map(Rc::new)
    }

    /// Populate traits in the EntityResult by fetching each entity's traits
    /// from the chain layer.
    fn fetch_mutations_results_traits(
        &self,
        entities_results: &mut Vec<EntityResult>,
        entities_traits_results: Vec<Rc<MutationAggregator>>,
    ) {
        for (entity_result, traits_results) in entities_results
            .iter_mut()
            .zip(entities_traits_results.into_iter())
        {
            let traits = self.fetch_entity_traits(traits_results);
            if let Some(entity) = entity_result.entity.as_mut() {
                entity.traits = traits;
            }
        }
    }

    /// Fetch traits data from chain layer.
    fn fetch_entity_traits(&self, results: Rc<MutationAggregator>) -> Vec<Trait> {
        results
            .traits
            .values()
            .flat_map(|merged_metadata| {
                let mutation = self.fetch_mutation_operation(
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
                if let MutationType::TraitPut(put_mut) = &merged_metadata.mutation_type {
                    trait_instance.creation_date =
                        put_mut.creation_date.map(|d| d.to_proto_timestamp());
                    trait_instance.modification_date =
                        put_mut.modification_date.map(|d| d.to_proto_timestamp());
                }

                Some(trait_instance)
            })
            .collect()
    }

    /// Fetch an operation from the chain layer by the given operation id and
    /// optional block offset.
    fn fetch_mutation_operation(
        &self,
        operation_id: OperationId,
        block_offset: Option<BlockOffset>,
    ) -> Result<Option<EntityMutation>, Error> {
        let operation = if let Some(block_offset) = block_offset {
            self.chain_handle
                .get_chain_operation(block_offset, operation_id)?
        } else {
            self.chain_handle.get_operation(operation_id)?
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
