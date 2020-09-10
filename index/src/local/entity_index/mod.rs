use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;
use std::path::PathBuf;
use std::rc::Rc;
use std::{sync::Arc, time::Instant};

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
    Entity, EntityMutation, EntityQuery, EntityResult as EntityResultProto, EntityResultSource,
    EntityResults, Trait,
};
use exocore_core::protos::{prost::ProstDateTimeExt, registry::Registry};

use crate::error::Error;
use crate::{
    entity::EntityId,
    ordering::{OrderingValueExt, OrderingValueWrapper},
};

use super::mutation_index::{IndexOperation, MutationIndex, MutationMetadata};
use super::top_results::RescoredTopResultsIterable;

mod config;
pub use config::*;

mod aggregator;
pub(crate) use aggregator::*;

#[cfg(test)]
mod test_index;
#[cfg(test)]
mod tests;

/// Manages and index entities and their traits stored in the chain and pending
/// store of the chain layer. The index accepts mutations from the chain layer
/// through its event stream, and manages both indices to be consistent.
///
/// The chain index is persisted on disk, while the pending store index is an
/// in-memory index. Since the persistence in the chain is not definitive until
/// blocks and their operations (entity mutations) are stored at a certain
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

        let chain_last_block = index.chain_handle.get_chain_last_block_info()?;
        let last_chain_indexed_block = index.chain_index.highest_indexed_block()?;
        if last_chain_indexed_block.is_none() && chain_last_block.is_some() {
            index.reindex_chain()?;
        }

        index.reindex_pending()?;

        Ok(index)
    }

    pub fn handle_chain_engine_event(
        &mut self,
        event: Event,
    ) -> Result<(Vec<OperationId>, usize), Error> {
        self.handle_chain_engine_events(std::iter::once(event))
    }

    /// Handle events coming from the chain layer. These events allow keeping
    /// the index consistent with the chain layer, up to the consistency
    /// guarantees that the layer offers. Returns operations that have been
    /// involved and the number of index operations applied.
    ///
    /// Since the events stream is buffered, we may receive a discontinuity if
    /// the chain layer couldn't send us an event. In that case, we re-index
    /// the pending index since we can't guarantee that we didn't lose an
    /// event.
    pub fn handle_chain_engine_events<E>(
        &mut self,
        events: E,
    ) -> Result<(Vec<OperationId>, usize), Error>
    where
        E: Iterator<Item = Event>,
    {
        let mut index_operations_count = 0;
        let mut affected_operations = Vec::new();

        let mut batched_operations = Vec::new();
        for event in events {
            // We collect new pending operations so that we can apply them in batch. As soon
            // as we hit another kind of events, we apply collected events at
            // once and then continue.
            if let Event::NewPendingOperation(op_id) = event {
                batched_operations.push(op_id);
                affected_operations.push(op_id);
                continue;
            } else if !batched_operations.is_empty() {
                let current_operations = std::mem::replace(&mut batched_operations, Vec::new());
                index_operations_count +=
                    self.handle_chain_pending_operations(current_operations.into_iter())?;
            }

            match event {
                Event::Started => {
                    info!("Chain engine is ready, indexing pending store & chain");
                    self.index_chain_new_blocks(Some(&mut affected_operations))?;
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
                    index_operations_count +=
                        self.index_chain_new_blocks(Some(&mut affected_operations))?;
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

        if !batched_operations.is_empty() {
            index_operations_count +=
                self.handle_chain_pending_operations(batched_operations.into_iter())?;
        }

        Ok((affected_operations, index_operations_count))
    }

    /// Execute a search query on the indices, and returning all entities
    /// matching the query.
    pub fn search<Q: Borrow<EntityQuery>>(&self, query: Q) -> Result<EntityResults, Error> {
        let begin_instant = Instant::now();

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
        let chain_hits = chain_results.total_results;
        let pending_results = self.pending_index.search_iter(&mutations_query)?;
        let pending_hits = pending_results.total_results;
        let after_query_instant = Instant::now();

        // create merged iterator, returning results from both underlying in order
        let chain_results = chain_results.map(|res| (res, EntityResultSource::Chain));
        let pending_results = pending_results.map(|res| (res, EntityResultSource::Pending));
        let combined_results = chain_results
            .merge_by(pending_results, |(res1, _src1), (res2, _src2)| {
                res1.sort_value >= res2.sort_value
            });

        let mut hasher = result_hasher();
        let mut entity_mutations_cache = HashMap::<EntityId, Rc<EntityAggregator>>::new();
        let mut matched_entities = HashSet::new();
        let mut got_results = false;
        let early_exit = std::cell::Cell::new(false);

        // iterate through results and returning the first N entities
        let mut entity_results = combined_results
            .take_while(|(_matched_mutation, _index_source)| !early_exit.get())
            // iterate through results, starting with best scores
            .flat_map(|(matched_mutation, index_source)| {
                let entity_id = matched_mutation.entity_id.clone();

                // check if we already processed this entity through another trait result that
                // ranked higher
                if matched_entities.contains(&entity_id) {
                    return None;
                }

                // fetch all entity mutations and cache them since we may have multiple hits for
                // same entity for traits that may have been removed since
                let entity_mutations = if let Some(mutations) =
                    entity_mutations_cache.get(&entity_id)
                {
                    mutations.clone()
                } else {
                    let mut entity_mutations = self
                        .fetch_entity_mutations_metadata(&entity_id)
                        .map_err(|err| {
                            error!(
                                "Error fetching mutations metadata for entity_id={} from indices: {}",
                                entity_id, err
                            );
                            err
                        })
                        .ok()?;

                    if !query.projections.is_empty() {
                        entity_mutations.annotate_projections(query.projections.as_slice());
                    }

                    let entity_mutations = Rc::new(entity_mutations);

                    entity_mutations_cache
                        .insert(entity_id.clone(), entity_mutations.clone());

                    entity_mutations
                };

                let operation_still_present = entity_mutations
                    .active_operations
                    .contains(&matched_mutation.operation_id);
                if (entity_mutations.deletion_date.is_some() || !operation_still_present)
                    && !query_include_deleted
                {
                    // we are here if the entity has been deleted (ex: explicitely or no traits remaining)
                    // or if the mutation metadata that was returned by the mutation index is not active anymore,
                    // which means that it got overriden by a subsequent operation.
                    return None;
                }

                matched_entities.insert(matched_mutation.entity_id.clone());

                // TODO: Support for negative rescoring https://github.com/appaquet/exocore/issues/143
                let ordering_value = matched_mutation.sort_value.clone();
                if ordering_value.value.is_within_page_bound(&current_page) {
                    got_results = true;

                    let creation_date = entity_mutations.creation_date.map(|t| t.to_proto_timestamp());
                    let modification_date = entity_mutations.modification_date.map(|t| t.to_proto_timestamp());
                    let deletion_date = entity_mutations.deletion_date.map(|t| t.to_proto_timestamp());

                    let result = EntityResult {
                        matched_mutation,
                        ordering_value: ordering_value.clone(),
                        proto: EntityResultProto {
                            entity: Some(Entity {
                                id: entity_id,
                                traits: Vec::new(),
                                creation_date,
                                modification_date,
                                deletion_date,
                                last_operation_id: entity_mutations.last_operatin_id,
                            }),
                            source: index_source.into(),
                            ordering_value: Some(ordering_value.value),
                            hash: entity_mutations.hash,
                        },
                        mutations: entity_mutations,
                    };

                    Some(result)
                } else {
                    if got_results {
                        // If we are here, it means that we found results within the page we were looking for, and then suddenly a new
                        // result doesn't fit in the page. This means that we are passed the page, and we can early exit since we won't find
                        // any new results passed this.
                        early_exit.set(true);
                    }

                    None
                }
            })
            // this steps consumes the results up until we reach the best 10 results based on the
            // score of the highest matching trait, but re-scored negatively based on
            // other traits
            .top_negatively_rescored_results(
                current_page.count as usize,
                |result| {
                    (result.ordering_value.clone(), result.ordering_value.clone())
                },
            )
            // accumulate results
            .fold(
                Vec::new(),
                |mut results, result| {
                    hasher.write_u64(result.mutations.hash);
                    results.push(result);
                    results
                },
            );

        let after_aggregate_instant = Instant::now();

        let next_page = if let Some(last_result) = entity_results.last() {
            let mut new_page = current_page.clone();

            let ascending = query
                .ordering
                .as_ref()
                .map(|s| s.ascending)
                .unwrap_or(false);
            if !ascending {
                new_page.before_ordering_value =
                    Some(last_result.matched_mutation.sort_value.value.clone());
            } else {
                new_page.after_ordering_value =
                    Some(last_result.matched_mutation.sort_value.value.clone());
            }

            Some(new_page)
        } else {
            None
        };

        // if query had a previous result hash and that new results have the same hash
        // we don't fetch results' data
        let results_hash = hasher.finish();
        let skipped_hash = results_hash == query.result_hash;
        if !skipped_hash {
            self.populate_results_traits(&mut entity_results, query.include_deleted);
        }

        let end_instant = Instant::now();
        debug!(
            "Query done chain_hits={} pending_hits={} aggr_fetch={} query={:?} aggr={:?} fetch={:?} total={:?} page={:?} next_page={:?}",
            chain_hits,
            pending_hits,
            entity_mutations_cache.len(),
            after_query_instant - begin_instant,
            after_aggregate_instant - after_query_instant,
            end_instant - after_aggregate_instant,
            end_instant - begin_instant,
            current_page,
            next_page,
        );

        let entities = entity_results.into_iter().map(|res| res.proto).collect();
        Ok(EntityResults {
            entities,
            skipped_hash,
            next_page,
            current_page: Some(current_page),
            estimated_count: (chain_hits + pending_hits) as u32,
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
    ) -> Result<usize, Error> {
        let (_last_chain_block_offset, last_chain_block_height) = self
            .chain_handle
            .get_chain_last_block_info()?
            .ok_or_else(|| {
                Error::Other("Tried to index chain, but it had no blocks in it".to_string())
            })?;

        let chain_index_min_depth = self.config.chain_index_min_depth;
        let chain_index_depth_leeway = self.config.chain_index_depth_leeway;
        let last_indexed_block = self.last_chain_indexed_block()?;
        let offset_from = last_indexed_block.map(|(offset, _height)| offset);
        if let Some((_last_indexed_offset, last_indexed_height)) = last_indexed_block {
            let depth = last_chain_block_height - last_indexed_height;
            if depth < chain_index_min_depth || depth < chain_index_depth_leeway {
                debug!(
                    "No need to index new blocks to chain index. last_chain_block_height={} last_indexed_block_height={} depth={} min_depth={} leeway={}",
                    last_chain_block_height, last_indexed_height, depth, chain_index_min_depth, chain_index_depth_leeway,
                );
                return Ok(0);
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
                    && last_chain_block_height.saturating_sub(*height) >= chain_index_min_depth
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

        let index_operations_count = pending_index_mutations.len();
        if index_operations_count > 0 {
            info!(
                "Indexed in chain, and deleted from pending {} operations. New chain index last offset is {:?}.",
                index_operations_count,
                new_highest_block_offset
            );

            self.pending_index
                .apply_operations(pending_index_mutations.into_iter())?;

            if let Some(new_highest_block_offset) = new_highest_block_offset {
                self.chain_index_last_block = Some(new_highest_block_offset);
            }
        }

        Ok(index_operations_count)
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
    ///
    /// Returns number of operations applied on the mutations index.
    fn handle_chain_pending_operations<O>(&mut self, operations_id: O) -> Result<usize, Error>
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
                Err(err) => { error!(
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
        let mutations = self.fetch_entity_mutations_metadata(entity_id)?;
        let traits = self.fetch_entity_traits(&mutations, false);

        Ok(Entity {
            id: entity_id.to_string(),
            traits,
            creation_date: mutations.creation_date.map(|t| t.to_proto_timestamp()),
            modification_date: mutations.modification_date.map(|t| t.to_proto_timestamp()),
            deletion_date: mutations.deletion_date.map(|t| t.to_proto_timestamp()),
            last_operation_id: mutations.last_operatin_id,
        })
    }

    /// Fetch indexed mutations metadata from pending and chain indices for this
    /// entity id, and merge them.
    fn fetch_entity_mutations_metadata(&self, entity_id: &str) -> Result<EntityAggregator, Error> {
        let pending_results = self.pending_index.fetch_entity_mutations(entity_id)?;
        let chain_results = self.chain_index.fetch_entity_mutations(entity_id)?;
        let mutations_metadata = pending_results
            .mutations
            .iter()
            .chain(chain_results.mutations.iter())
            .cloned();

        EntityAggregator::new(mutations_metadata)
    }

    /// Populate traits in the EntityResult by fetching each entity's traits
    /// from the chain layer.
    fn populate_results_traits(
        &self,
        entity_results: &mut Vec<EntityResult>,
        include_deleted: bool,
    ) {
        for entity_result in entity_results {
            let traits = self.fetch_entity_traits(&entity_result.mutations, include_deleted);
            if let Some(entity) = entity_result.proto.entity.as_mut() {
                entity.traits = traits;
            }
        }
    }

    /// Fetch traits data from chain layer.
    fn fetch_entity_traits(
        &self,
        entity_mutations: &EntityAggregator,
        include_deleted: bool,
    ) -> Vec<Trait> {
        entity_mutations
            .traits
            .iter()
            .flat_map(|(trait_id, agg)| {
                if let Some(projection) = &agg.projection {
                    if projection.skip {
                        return None;
                    }
                }

                if agg.deletion_date.is_some() && !include_deleted {
                    return None;
                }

                let (mut_metadata, _put_mut_metadata) = agg.last_put_mutation()?;

                let mutation = self.fetch_chain_mutation_operation(
                    mut_metadata.operation_id,
                    mut_metadata.block_offset,
                );
                let mutation = match mutation {
                    Ok(Some(mutation)) => mutation,
                    other => {
                        error!(
                            "Couldn't fetch operation_id={} for entity_id={}: {:?}",
                            mut_metadata.operation_id, mut_metadata.entity_id, other
                        );
                        return None;
                    }
                };

                let mut trt = match mutation.mutation? {
                    Mutation::PutTrait(trait_put) => trait_put.r#trait,
                    Mutation::CompactTrait(trait_cmpt) => trait_cmpt.r#trait,
                    Mutation::DeleteTrait(_)
                    | Mutation::DeleteEntity(_)
                    | Mutation::UpdateTrait(_)
                    | Mutation::Test(_) => return None,
                }?;

                if let Some(projection) = &agg.projection {
                    let res =
                        project_trait(self.cell.schemas().as_ref(), &mut trt, projection.as_ref());

                    if let Err(err) = res {
                        error!(
                            "Couldn't run projection on trait_id={} of entity_id={}: {:?}",
                            trait_id, mut_metadata.entity_id, err,
                        );
                    }
                }

                // update the trait's dates that got merged from metadata
                trt.creation_date = agg.creation_date.map(|d| d.to_proto_timestamp());
                trt.modification_date = agg.modification_date.map(|d| d.to_proto_timestamp());
                trt.deletion_date = agg.deletion_date.map(|d| d.to_proto_timestamp());
                trt.last_operation_id = agg.last_operation_id.unwrap_or_default();

                Some(trt)
            })
            .collect()
    }

    /// Fetch an operation from the chain layer by the given operation id and
    /// optional block offset.
    fn fetch_chain_mutation_operation(
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

/// Wrapper for entity result with matched mutation from index layer along
/// aggregated traits.
pub struct EntityResult {
    pub matched_mutation: MutationMetadata,
    pub ordering_value: OrderingValueWrapper,
    pub proto: EntityResultProto,
    pub mutations: Rc<EntityAggregator>,
}
