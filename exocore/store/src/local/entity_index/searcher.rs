use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    time::Instant,
};

use exocore_protos::{
    prost::ProstDateTimeExt,
    store::{Entity, EntityQuery, EntityResult, EntityResultSource, EntityResults, Projection},
};
use itertools::Itertools;

use super::{gc::GarbageCollector, EntityAggregator};
use crate::{
    entity::EntityId,
    error::Error,
    local::{
        entity_index::result_hasher,
        mutation_index::{MutationIndex, MutationMetadata},
        top_results::ReScoredTopResultsIterable,
    },
    ordering::{OrderingValueExt, OrderingValueWrapper},
};

pub type EntityMetaCache = HashMap<EntityId, Rc<EntityAggregator>>;

pub struct Searcher<'i, M, E>
where
    M: Fn(&mut EntityMetaCache, &str, &[Projection]) -> Option<Rc<EntityAggregator>>,
    E: Fn(&mut Vec<SearchResult>, bool),
{
    chain_index: &'i MutationIndex,
    pending_index: &'i MutationIndex,
    gc: &'i GarbageCollector,
    meta_fetcher: M,
    entity_fetcher: E,
    query: &'i EntityQuery,
}

impl<'i, M, E> Searcher<'i, M, E>
where
    M: Fn(&mut EntityMetaCache, &str, &[Projection]) -> Option<Rc<EntityAggregator>>,
    E: Fn(&mut Vec<SearchResult>, bool),
{
    pub fn new(
        chain_index: &'i MutationIndex,
        pending_index: &'i MutationIndex,
        gc: &'i GarbageCollector,
        meta_fetcher: M,
        entity_fetcher: E,
        query: &'i EntityQuery,
    ) -> Self {
        Self {
            chain_index,
            pending_index,
            gc,
            meta_fetcher,
            entity_fetcher,
            query,
        }
    }

    pub fn search(&self) -> Result<EntityResults, Error> {
        let begin_instant = Instant::now();

        let query_include_deleted = self.query.include_deleted;
        let mut query_page = self
            .query
            .paging
            .clone()
            .unwrap_or_else(crate::query::default_paging);
        crate::query::fill_default_paging(&mut query_page);

        let reference_boost = self
            .query
            .ordering
            .as_ref()
            .map_or(true, |o| !o.no_reference_boost);

        let (chain_hits, pending_hits, combined_results) = self.search_hits()?;

        let after_query_instant = Instant::now();

        let hasher = result_hasher();
        let mut digest = hasher.digest();
        let mut entity_mutations_cache = HashMap::<EntityId, Rc<EntityAggregator>>::new();
        let mut matched_entities = HashSet::new();

        // iterate through results and returning the first N entities
        let mut entity_results = combined_results
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
                let entity_mutations = (self.meta_fetcher)(
                    &mut entity_mutations_cache,
                    &entity_id,
                    &self.query.projections,
                )?;

                let operation_still_present = entity_mutations
                    .active_operations
                    .contains(&matched_mutation.operation_id);
                if entity_mutations.deletion_date.is_some() || !operation_still_present {
                    // we are here if the entity has been deleted (ex: explicitly or no traits
                    // remaining) or if the mutation metadata that was returned
                    // by the mutation index is not active anymore, which means
                    // that it got overridden by a subsequent operation.
                    //
                    // We let the garbage collector know that the entity may need to be garbage
                    // collected. The actual garbage collection will be done
                    // asynchronously at later time.
                    self.gc.maybe_flag_for_collection(&entity_mutations);

                    if !query_include_deleted {
                        return None;
                    }
                }
                matched_entities.insert(matched_mutation.entity_id.clone());

                // Unless disabled, penalizes the entity score if it doesn't have a reference to
                // another object
                let mut ordering_value = matched_mutation.sort_value.clone();
                let original_ordering_value = ordering_value.clone();
                if reference_boost && ordering_value.is_score() && !entity_mutations.has_reference {
                    ordering_value.boost_score(0.3);
                };

                if ordering_value.value.is_within_page_bound(&query_page) {
                    let result = SearchResult {
                        matched_mutation,
                        ordering_value: ordering_value.clone(),
                        original_ordering_value,
                        proto: EntityResult {
                            entity: Some(Entity {
                                id: entity_id,
                                traits: Vec::new(),
                                creation_date: opt_date_to_proto(entity_mutations.creation_date),
                                modification_date: opt_date_to_proto(
                                    entity_mutations.modification_date,
                                ),
                                deletion_date: opt_date_to_proto(entity_mutations.deletion_date),
                                last_operation_id: entity_mutations.last_operation_id,
                            }),
                            source: index_source.into(),
                            ordering_value: Some(ordering_value.value),
                            hash: entity_mutations.hash,
                        },
                        mutations: entity_mutations,
                    };

                    Some(result)
                } else {
                    None
                }
            })
            // this steps consumes the results up until we reach the best 10 results based on the
            // score of the highest matching trait, but re-scored negatively based on
            // other traits
            .top_negatively_rescored_results(query_page.count as usize, |result: &SearchResult| {
                (
                    result.original_ordering_value.clone(),
                    result.ordering_value.clone(),
                )
            })
            // accumulate results
            .fold(
                Vec::new(),
                |mut results: Vec<SearchResult>, result: SearchResult| {
                    digest.update(&result.mutations.hash.to_ne_bytes());
                    results.push(result);
                    results
                },
            );

        let after_aggregate_instant = Instant::now();

        let next_page = self.next_paging(&entity_results, &query_page);

        // if query specifies a `result_hash` and that new results have the same hash,
        // we don't fetch results' data
        let results_hash = digest.finalize();
        let skipped_hash = results_hash == self.query.result_hash;
        if !skipped_hash {
            (self.entity_fetcher)(&mut entity_results, self.query.include_deleted);
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
            query_page,
            next_page,
        );

        let entities = entity_results.into_iter().map(|res| res.proto).collect();
        Ok(EntityResults {
            entities,
            skipped_hash,
            next_page,
            current_page: Some(query_page),
            estimated_count: (chain_hits + pending_hits) as u32,
            hash: results_hash,
        })
    }

    fn search_hits(
        &self,
    ) -> Result<
        (
            usize,
            usize,
            impl Iterator<Item = (MutationMetadata, EntityResultSource)> + '_,
        ),
        Error,
    > {
        // query pending & chain mutation index without original query paging since we
        // need to do our own paging here since we are re-ranking results and
        // that we may have more than one mutation match for each entity.
        let mutations_query = Rc::new(EntityQuery {
            paging: None,
            ..self.query.clone()
        });

        let chain_results = self.chain_index.search_iter(mutations_query.clone())?;
        let chain_hits = chain_results.total_results;

        let pending_results = self.pending_index.search_iter(mutations_query)?;
        let pending_hits = pending_results.total_results;

        let chain_results = chain_results.map(|res| (res, EntityResultSource::Chain));
        let pending_results = pending_results.map(|res| (res, EntityResultSource::Pending));
        let combined_results = chain_results
            .merge_by(pending_results, |(res1, _src1), (res2, _src2)| {
                res1.sort_value >= res2.sort_value
            });

        Ok((chain_hits, pending_hits, combined_results))
    }

    fn next_paging(
        &self,
        entity_results: &[SearchResult],
        query_paging: &exocore_protos::store::Paging,
    ) -> Option<exocore_protos::store::Paging> {
        if let Some(last_result) = entity_results.last() {
            let mut new_paging = query_paging.clone();

            let ascending = self
                .query
                .ordering
                .as_ref()
                .map(|s| s.ascending)
                .unwrap_or(false);
            if !ascending {
                new_paging.before_ordering_value = Some(last_result.ordering_value.value.clone());
            } else {
                new_paging.after_ordering_value = Some(last_result.ordering_value.value.clone());
            }

            Some(new_paging)
        } else {
            None
        }
    }
}

/// Wrapper for entity result with matched mutation from store layer along
/// aggregated traits.
pub struct SearchResult {
    pub matched_mutation: MutationMetadata,
    pub ordering_value: OrderingValueWrapper,
    pub original_ordering_value: OrderingValueWrapper,
    pub proto: EntityResult,
    pub mutations: Rc<EntityAggregator>,
}

fn opt_date_to_proto(
    dt: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<exocore_protos::prost::Timestamp> {
    dt.map(|t| t.to_proto_timestamp())
}
