use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use exocore_chain::{
    block::{BlockHeight, BlockOffset},
    chain,
    engine::Event,
    operation::{Operation, OperationId},
    pending, EngineHandle, EngineOperationStatus,
};
use exocore_core::{
    cell::FullCell,
    time::{Clock, Instant},
};
use exocore_protos::{
    generated::exocore_store::{
        entity_mutation::Mutation, EntityMutation, EntityQuery, EntityResults, Trait,
    },
    prost::{Message, ProstDateTimeExt},
    registry::Registry,
    store::Projection,
};
use gc::GarbageCollector;
use itertools::Itertools;

use super::mutation_index::{IndexOperation, MutationIndex, MutationMetadata};
use crate::error::Error;

mod config;
pub use config::*;

mod aggregator;
pub use aggregator::*;

pub mod gc;

mod searcher;

pub mod iterator;

#[cfg(test)]
pub(crate) mod test_index;
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
    full_cell: FullCell,
    chain_handle: EngineHandle<CS, PS>,
    gc: GarbageCollector,
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
        clock: Clock,
    ) -> Result<EntityIndex<CS, PS>, Error> {
        let mut pending_index = MutationIndex::create_in_memory(
            config.pending_index_config,
            cell.cell().schemas().clone(),
        )?;
        pending_index.set_full_text_boost(config.pending_index_boost);

        // make sure directories are created
        let chain_index_dir = cell
            .cell()
            .store_directory()
            .as_os_path()
            .expect("Expected cell to be in an OS directory")
            .join("chain");
        if std::fs::metadata(&chain_index_dir).is_err() {
            std::fs::create_dir_all(&chain_index_dir)?;
        }

        let chain_index =
            Self::create_chain_index(config, cell.cell().schemas(), &chain_index_dir)?;
        let mut index = EntityIndex {
            config,
            pending_index,
            chain_index_dir,
            chain_index,
            chain_index_last_block: None,
            full_cell: cell,
            chain_handle,
            gc: GarbageCollector::new(config.garbage_collector, clock),
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

    /// Handles events coming from the chain layer. These events allow keeping
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
                let current_operations = std::mem::take(&mut batched_operations);
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
                            return Err(Error::Fatal(anyhow!(
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

    pub fn maybe_index_chain_blocks(&mut self) -> Result<Vec<OperationId>, Error> {
        let mut affected_operations = Vec::new();
        self.index_chain_new_blocks(Some(&mut affected_operations))?;
        Ok(affected_operations)
    }

    pub fn search<Q: Borrow<EntityQuery>>(&self, query: Q) -> Result<EntityResults, Error> {
        let searcher = searcher::Searcher::new(
            &self.chain_index,
            &self.pending_index,
            &self.gc,
            |cache, entity_id, projections| {
                self.fetch_and_cache_entity_mutations_metadata(cache, entity_id, projections)
            },
            |entity_results, include_deleted| {
                self.populate_results_traits(entity_results, include_deleted)
            },
            query.borrow(),
        );

        searcher.search()
    }

    /// Calls the garbage collector to run a pass on entities that got flagged
    /// and generates deletion mutations.
    pub fn run_garbage_collector(&self) -> Result<Vec<EntityMutation>, Error> {
        let last_chain_indexed_block = self
            .last_chain_indexed_block()
            .map_err(|err| anyhow!("Couldn't get last chain indexed block: {}", err))?;
        if last_chain_indexed_block.is_none() {
            // we can only run GC from chain index
            return Ok(Vec::new());
        };

        let deletions = self
            .gc
            .run(|entity_id| self.fetch_entity_mutations_metadata(entity_id));

        Ok(deletions)
    }

    /// Creates the chain index based on configuration.
    fn create_chain_index<P: AsRef<Path>>(
        config: EntityIndexConfig,
        schemas: &Arc<Registry>,
        chain_index_dir: P,
    ) -> Result<MutationIndex, Error> {
        if !config.chain_index_in_memory {
            MutationIndex::open_or_create_mmap(
                config.chain_index_config,
                schemas.clone(),
                chain_index_dir.as_ref(),
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
            self.full_cell.cell().schemas().clone(),
        )?;
        self.pending_index
            .set_full_text_boost(self.config.pending_index_boost);

        let last_chain_indexed_offset = self
            .last_chain_indexed_block()?
            .map(|(offset, _height)| offset)
            .unwrap_or(0);

        info!(
            "Clearing & re-indexing pending index. last_chain_indexed_offset={}",
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
            self.full_cell.cell().schemas().clone(),
        )?;

        // remove and re-create data dir
        std::fs::remove_dir_all(&self.chain_index_dir)?;
        std::fs::create_dir_all(&self.chain_index_dir)?;

        // re-create index, and force re-index of chain
        self.chain_index = Self::create_chain_index(
            self.config,
            self.full_cell.cell().schemas(),
            &self.chain_index_dir,
        )?;
        self.index_chain_new_blocks(None)?;

        self.reindex_pending()?;

        Ok(())
    }

    /// Checks if we need to index any new block in the chain.
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
            .ok_or_else(|| anyhow!("Tried to index chain, but it had no blocks in it"))?;

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

        let pending_index_empty = self.pending_index.highest_indexed_block()?.is_none();

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
                let operation_id = engine_operation.operation_id;
                let (index_ops, entity_id) =
                    IndexOperation::from_chain_engine_operation(engine_operation, offset);

                if !pending_index_empty {
                    // delete from pending index if it's not already empty
                    pending_index_mutations.push(IndexOperation::DeleteEntityOperation(
                        entity_id,
                        operation_id,
                    ));
                }

                // take note of the latest block that we indexed in chain
                if Some(offset) > new_highest_block_offset {
                    new_highest_block_offset = Some(offset);
                }

                index_ops
            });

        let before_apply = Instant::now();
        self.chain_index.apply_operations(chain_index_mutations)?;

        let index_operations_count = pending_index_mutations.len();
        if index_operations_count > 0 {
            self.pending_index
                .apply_operations(pending_index_mutations.into_iter())?;

            if let Some(new_highest_block_offset) = new_highest_block_offset {
                self.chain_index_last_block = Some(new_highest_block_offset);
            }

            info!(
                "Indexed in chain, and deleted from pending {} operations (from offset {:?}) in {:?}. New chain index last offset is {:?}.",
                index_operations_count,
                offset_from,
                before_apply.elapsed(),
                new_highest_block_offset
            );
        }

        Ok(index_operations_count)
    }

    /// Gets last block that got indexed in the chain index
    fn last_chain_indexed_block(&self) -> Result<Option<(BlockOffset, BlockHeight)>, Error> {
        let mut last_indexed_offset = self.chain_index_last_block;

        if last_indexed_offset.is_none() {
            last_indexed_offset = self.chain_index.highest_indexed_block()?;
        }

        match last_indexed_offset {
            Some(offset) => {
                let block_info = self.chain_handle.get_chain_block_info(offset)?;
                Ok(block_info)
            }
            None => Ok(None),
        }
    }

    /// Handles new pending store operations events from the chain layer by
    /// indexing them into the pending index.
    ///
    /// Returns number of operations applied on the mutation index.
    fn handle_chain_pending_operations<O>(&mut self, operations_id: O) -> Result<usize, Error>
    where
        O: Iterator<Item = OperationId>,
    {
        #![allow(clippy::needless_collect)] // see https://github.com/rust-lang/rust-clippy/issues/6066
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

    /// Fetches an entity and all its traits from indices and the chain layer.
    /// Traits returned follow mutations in order of operation id.
    #[cfg(test)]
    fn fetch_entity(
        &self,
        entity_id: &str,
    ) -> Result<exocore_protos::generated::exocore_store::Entity, Error> {
        let aggr = self.fetch_aggregated_entity_mutations(entity_id)?;
        let traits = self.fetch_entity_traits(&aggr, false);

        Ok(exocore_protos::generated::exocore_store::Entity {
            id: entity_id.to_string(),
            traits,
            creation_date: aggr.creation_date.map(|t| t.to_proto_timestamp()),
            modification_date: aggr.modification_date.map(|t| t.to_proto_timestamp()),
            deletion_date: aggr.deletion_date.map(|t| t.to_proto_timestamp()),
            last_operation_id: aggr.last_operation_id,
        })
    }

    /// Fetches indexed mutations metadata from pending and chain indices for
    /// this entity id and aggregate them.
    pub(super) fn fetch_aggregated_entity_mutations(
        &self,
        entity_id: &str,
    ) -> Result<EntityAggregator, Error> {
        let mutations_metadata = self.fetch_entity_mutations_metadata(entity_id)?;
        Ok(EntityAggregator::new(mutations_metadata))
    }

    /// Fetches indexed mutations metadata from pending and chain indices for
    /// this entity id.
    pub(super) fn fetch_entity_mutations_metadata(
        &self,
        entity_id: &str,
    ) -> Result<impl Iterator<Item = MutationMetadata>, Error> {
        let pending_results = self.pending_index.fetch_entity_mutations(entity_id)?;
        let chain_results = self.chain_index.fetch_entity_mutations(entity_id)?;
        Ok(pending_results
            .mutations
            .into_iter()
            .chain(chain_results.mutations)
            .sorted_by_key(|result| {
                // sorts mutations in order they got committed (block offset/pending, then
                // operation id)
                let block_offset = result.block_offset.unwrap_or(std::u64::MAX);
                (block_offset, result.operation_id)
            })
            .dedup_by(|a, b| {
                // make sure we don't have duplicate across pending & chain (may happen
                // temporarily)
                a.operation_id == b.operation_id
            }))
    }

    /// Fetches and cache indexed mutations metadata.
    fn fetch_and_cache_entity_mutations_metadata(
        &self,
        cache: &mut searcher::EntityMetaCache,
        entity_id: &str,
        projections: &[Projection],
    ) -> Option<Rc<EntityAggregator>> {
        let entity_mutations = if let Some(mutations) = cache.get(entity_id) {
            mutations.clone()
        } else {
            let mut entity_mutations = self
                .fetch_aggregated_entity_mutations(entity_id)
                .map_err(|err| {
                    error!(
                        "Error fetching mutations metadata for entity_id={} from indices: {}",
                        entity_id, err
                    );
                    err
                })
                .ok()?;

            if !projections.is_empty() {
                entity_mutations.annotate_projections(projections);
            }

            let entity_mutations = Rc::new(entity_mutations);

            cache.insert(entity_id.to_string(), entity_mutations.clone());

            entity_mutations
        };
        Some(entity_mutations)
    }

    /// Populates traits in the EntityResult by fetching each entity's traits
    /// from the chain layer.
    fn populate_results_traits(
        &self,
        entity_results: &mut Vec<searcher::SearchResult>,
        include_deleted: bool,
    ) {
        for entity_result in entity_results {
            if entity_result.mutations.should_collect() {
                self.gc.maybe_flag_for_collection(&entity_result.mutations);
            }

            let traits = self.fetch_entity_traits(&entity_result.mutations, include_deleted);
            if let Some(entity) = entity_result.proto.entity.as_mut() {
                entity.traits = traits;
            }
        }
    }

    /// Fetches traits data from chain layer.
    fn fetch_entity_traits(
        &self,
        entity_mutations: &EntityAggregator,
        include_deleted: bool,
    ) -> Vec<Trait> {
        entity_mutations
            .traits
            .iter()
            .filter_map(|(trait_id, agg)| {
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

                if mutation.entity_id != entity_mutations.entity_id {
                    error!(
                        "Fetched from chain operation {} that didn't belong to entity {}, but entity {}",
                        mut_metadata.operation_id, entity_mutations.entity_id, mutation.entity_id
                    );
                    return None;
                }

                let mut trt = match mutation.mutation? {
                    Mutation::PutTrait(put_mut) => put_mut.r#trait,
                    Mutation::DeleteTrait(_)
                    | Mutation::DeleteEntity(_)
                    | Mutation::DeleteOperations(_)
                    | Mutation::Test(_) => return None,
                }?;

                if let Some(projection) = &agg.projection {
                    let res = project_trait_fields(
                        self.full_cell.cell().schemas().as_ref(),
                        &mut trt,
                        projection.as_ref(),
                    );

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

    /// Fetches an operation from the chain layer by the given operation id and
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

        let Some(operation) = operation else {
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
