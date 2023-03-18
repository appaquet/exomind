use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
    time::Duration,
};

use chrono::{DateTime, Utc};
use exocore_chain::operation::OperationId;
use exocore_core::time::{Clock, ConsistentTimestamp};
use exocore_protos::{core::EntityGarbageCollectorConfig, store::EntityMutation};

use super::EntityAggregator;
use crate::{
    entity::{EntityId, EntityIdRef, TraitId},
    error::Error,
    local::mutation_index::MutationMetadata,
    mutation::MutationBuilder,
};

const DAY_SECS: u64 = 86_400;

/// The entity garbage collector generates operation deletion mutations on
/// entities that need to be cleaned up from the mutation index.
///
/// Since the chain contains an immutable list of entity mutations, the
/// mutation index contains every mutation that an entity got. When deleting an
/// entity or a trait, we simply create a tombstone to support versioning and to
/// make sure that all nodes have a consistent index. This is also required to
/// make sure that a new node that bootstraps doesn't need to garbage collector
/// the index by itself. By including the deletions in the chain, this new node
/// can delete mutations by operation ids as it index them.
///
/// When searching, if the mutation that got returned by the mutation index
/// isn't valid anymore because it got overridden by another mutation or
/// because the last mutation was a deletion (tombstone on entity or trait), the
/// search result is discarded (but could be returned by a later valid
/// mutations). This eventually creates a problem where a lot of mutations get
/// returned by the mutation index to be then discarded by the entity index, up
/// to eventually creating the issue where we hit the maximum number of pages
/// and don't return valid entities.
///
/// At interval, the entity store calls the entity index to do a garbage
/// collection run.
///
/// There is 3 kind of garbage collections:
/// * Deleted entity: effectively deletes all traits of an entity.
/// * Deleted trait: effectively deletes all versions of a trait of an entity.
/// * Trait: deletes old versions of a trait.
pub struct GarbageCollector {
    config: GarbageCollectorConfig,
    max_entity_deleted_duration: chrono::Duration,
    max_trait_deleted_duration: chrono::Duration,
    trait_versions_min_age: chrono::Duration,
    clock: Clock,
    inner: Arc<RwLock<Inner>>,
}

impl GarbageCollector {
    pub fn new(config: GarbageCollectorConfig, clock: Clock) -> Self {
        let max_entity_deleted_duration =
            chrono::Duration::from_std(config.deleted_entity_collection)
                .expect("Couldn't convert `deleted_entity_collection` to chrono::Duration");

        let max_trait_deleted_duration =
            chrono::Duration::from_std(config.deleted_trait_collection)
                .expect("Couldn't convert `deleted_trait_collection` to chrono::Duration");

        let trait_versions_min_age = chrono::Duration::from_std(config.min_operation_age)
            .expect("Couldn't convert `trait_versions_min_age` to chrono::Duration");

        let inner = Arc::new(RwLock::new(Inner {
            config,
            entity_ids: HashSet::new(),
        }));

        GarbageCollector {
            config,
            max_entity_deleted_duration,
            max_trait_deleted_duration,
            trait_versions_min_age,
            clock,
            inner,
        }
    }

    /// Checks if an entity for which we collected its mutation metadata from
    /// the index should be added to the garbage collection queue.
    pub fn maybe_flag_for_collection(&self, aggregator: &EntityAggregator) {
        let ops = self.gen_entity_collections(aggregator);
        if !ops.is_empty() {
            let mut inner = self.inner.write().expect("Fail to acquire inner lock");
            inner.maybe_enqueue(&aggregator.entity_id);
        }
    }

    /// Garbage collect entities currently in queue.
    pub fn run<F, I>(&self, entity_fetcher: F) -> Vec<EntityMutation>
    where
        F: Fn(EntityIdRef) -> Result<I, Error>,
        I: Iterator<Item = MutationMetadata>,
    {
        let entity_ids = {
            let mut inner = self.inner.write().expect("Fail to acquire inner lock");
            std::mem::take(&mut inner.entity_ids)
        };

        if entity_ids.is_empty() {
            return Vec::new();
        }

        debug!(
            "Starting a garbage collection pass for {} entities...",
            entity_ids.len(),
        );
        let min_op_time = self.clock.now_chrono() - self.trait_versions_min_age;
        let mut deletions = Vec::new();
        for entity_id in &entity_ids {
            let Ok(sorted_mutations) = entity_fetcher(entity_id) else {
                error!("couldn't fetch mutations for entity {}", entity_id);
                continue;
            };

            let sorted_mutations: Vec<MutationMetadata> = sorted_mutations.collect();
            let aggregator = EntityAggregator::new(sorted_mutations.iter().cloned());

            let gc_ops = self.gen_entity_collections(&aggregator);
            if gc_ops.is_empty() {
                // nothing to collect anymore or entity has pending mutations
                continue;
            }

            for gc_op in gc_ops {
                let opt_deletion = match gc_op {
                    GCOperation::DeleteEntity => {
                        collect_delete_entity(&aggregator, &sorted_mutations)
                    }
                    GCOperation::DeleteTrait(trait_id) => {
                        collect_delete_trait(&aggregator, &trait_id, &sorted_mutations)
                    }
                    GCOperation::CompactTrait(trait_id) => collect_trait_versions(
                        &aggregator,
                        &trait_id,
                        min_op_time,
                        self.config.trait_versions_max,
                        &sorted_mutations,
                    ),
                };
                if let Some(deletion) = opt_deletion {
                    deletions.push(deletion);
                }
            }
        }

        if !deletions.is_empty() {
            info!(
                "Garbage collection generated {} deletion operations for {} entities",
                deletions.len(),
                entity_ids.len(),
            );
        }

        deletions
    }

    /// Generates garbage collection operations that need to be applied for an
    /// entity.
    fn gen_entity_collections(&self, aggregator: &EntityAggregator) -> Vec<GCOperation> {
        // we don't collect if any of the operations are still in pending store or if
        // a deletion is pending for the entity
        if aggregator.in_pending || aggregator.pending_deletion {
            return Vec::new();
        }

        let now = self.clock.now_chrono();

        if let Some(deletion_date) = &aggregator.deletion_date {
            let elapsed = now.signed_duration_since(*deletion_date);
            if elapsed >= self.max_entity_deleted_duration {
                trace!(
                    "Collecting entity {} since got deleted since",
                    aggregator.entity_id
                );
                return vec![GCOperation::DeleteEntity];
            }
        }

        let mut ops = Vec::new();

        for (trait_id, trait_aggr) in &aggregator.traits {
            if let Some(deletion_date) = &trait_aggr.deletion_date {
                let elapsed = now.signed_duration_since(*deletion_date);
                if elapsed >= self.max_trait_deleted_duration {
                    trace!(
                        "Collecting entity {} trait {} since it got deleted",
                        aggregator.entity_id,
                        trait_id
                    );
                    ops.push(GCOperation::DeleteTrait(trait_id.clone()));
                    continue;
                }
            }

            if trait_aggr.mutation_count > self.config.trait_versions_leeway {
                trace!(
                    "Collecting entity {} trait {} since it has too many versions ({})",
                    aggregator.entity_id,
                    trait_id,
                    trait_aggr.mutation_count,
                );
                ops.push(GCOperation::CompactTrait(trait_id.clone()));
            }
        }

        ops
    }
}

/// Configuration of the entity garbage collector.
#[derive(Debug, Clone, Copy)]
pub struct GarbageCollectorConfig {
    /// After how long do we collect a fully deleted entity.
    pub deleted_entity_collection: Duration,

    /// After how long do we collect a fully deleted trait.
    pub deleted_trait_collection: Duration,

    /// Maximum versions to keep for a trait when it is compacted.
    pub trait_versions_max: usize,

    /// After how many versions for a trait a compaction is triggered.
    pub trait_versions_leeway: usize,

    /// Minimum age an operation needs to have in order to be collected.
    pub min_operation_age: Duration,

    /// Size of the queue of entities to be collected.
    pub queue_size: usize,
}

impl Default for GarbageCollectorConfig {
    fn default() -> Self {
        GarbageCollectorConfig {
            deleted_entity_collection: Duration::from_secs(14 * DAY_SECS),
            deleted_trait_collection: Duration::from_secs(14 * DAY_SECS),
            trait_versions_max: 5,
            trait_versions_leeway: 7,
            min_operation_age: Duration::from_secs(7 * DAY_SECS),
            queue_size: 500,
        }
    }
}

impl From<EntityGarbageCollectorConfig> for GarbageCollectorConfig {
    fn from(proto: EntityGarbageCollectorConfig) -> Self {
        let mut config = GarbageCollectorConfig::default();

        if let Some(v) = proto.queue_size {
            config.queue_size = v as usize;
        }

        config
    }
}

/// Creates a deletion mutation for all operations of the entity.
fn collect_delete_entity(
    aggr: &EntityAggregator,
    mutations: &[MutationMetadata],
) -> Option<EntityMutation> {
    assert!(aggr.deletion_date.is_some());

    let operation_ids: Vec<OperationId> = mutations
        .iter()
        .map(|mutation| mutation.operation_id)
        .collect();
    if operation_ids.is_empty() {
        return None;
    }

    debug!(
        "Creating delete operation to garbage collect deleted entity {} with operations {:?}",
        aggr.entity_id, operation_ids,
    );
    MutationBuilder::new()
        .delete_operations(&aggr.entity_id, operation_ids)
        .build()
        .mutations
        .into_iter()
        .next()
}

/// Creates a deletion mutation for all operations of a trait of an entity.
fn collect_delete_trait(
    aggr: &EntityAggregator,
    trait_id: &str,
    mutations: &[MutationMetadata],
) -> Option<EntityMutation> {
    assert!(aggr.traits[trait_id].deletion_date.is_some());

    let trait_mutations = filter_trait_mutations(mutations.iter(), trait_id);
    let operation_ids: Vec<OperationId> = trait_mutations
        .map(|mutation| mutation.operation_id)
        .collect();
    if operation_ids.is_empty() {
        return None;
    }

    debug!(
        "Creating delete operation to garbage collect deleted trait {} of entity {} with operations {:?}",
        trait_id, aggr.entity_id, operation_ids,
    );
    MutationBuilder::new()
        .delete_operations(&aggr.entity_id, operation_ids)
        .build()
        .mutations
        .into_iter()
        .next()
}

/// Creates a deletion mutation for the N oldest operations of a trait so that
/// we only keep the most recent ones.
fn collect_trait_versions(
    aggr: &EntityAggregator,
    trait_id: &str,
    min_op_time: DateTime<Utc>,
    max_versions: usize,
    mutations: &[MutationMetadata],
) -> Option<EntityMutation> {
    let trait_operations: Vec<OperationId> = filter_trait_mutations(mutations.iter(), trait_id)
        .filter(|mutation| {
            let op_time = ConsistentTimestamp::from(mutation.operation_id).to_datetime();
            op_time < min_op_time
        })
        .map(|mutation| mutation.operation_id)
        .collect();

    if trait_operations.len() <= max_versions {
        return None;
    }

    let to_delete_count = trait_operations.len() - max_versions;
    let operation_ids: Vec<OperationId> =
        trait_operations.into_iter().take(to_delete_count).collect();

    assert_inactive_operations(aggr, &operation_ids);

    debug!(
        "Creating delete operation to garbage collect operations {:?} of trait {} of entity {}",
        operation_ids, trait_id, aggr.entity_id
    );
    MutationBuilder::new()
        .delete_operations(&aggr.entity_id, operation_ids)
        .build()
        .mutations
        .into_iter()
        .next()
}

/// Filters mutations to only return those that are for a specified trait.
fn filter_trait_mutations<'i, I>(
    mutations: I,
    trait_id: &'i str,
) -> impl Iterator<Item = &'i MutationMetadata> + 'i
where
    I: Iterator<Item = &'i MutationMetadata> + 'i,
{
    use crate::local::mutation_index::MutationType;
    mutations.filter(move |mutation| match &mutation.mutation_type {
        MutationType::TraitPut(put_mut) if put_mut.trait_id == trait_id => true,
        MutationType::TraitTombstone(tomb_trait_id) if tomb_trait_id == trait_id => true,
        _ => false,
    })
}

fn assert_inactive_operations(aggr: &EntityAggregator, to_delete_ops: &[OperationId]) {
    for op_id in to_delete_ops {
        if aggr.active_operations.contains(op_id) {
            panic!("Tried to garbage collect operation {} for entity {} but operation was still active", op_id, aggr.entity_id);
        }
    }
}

struct Inner {
    config: GarbageCollectorConfig,
    entity_ids: HashSet<EntityId>,
}

impl Inner {
    fn maybe_enqueue(&mut self, entity_id: EntityIdRef) {
        if self.is_full() {
            return;
        }

        if self.entity_ids.contains(entity_id) {
            return;
        }

        self.entity_ids.insert(entity_id.to_string());
    }

    fn is_full(&self) -> bool {
        self.entity_ids.len() >= self.config.queue_size
    }
}

enum GCOperation {
    DeleteEntity,
    DeleteTrait(TraitId),
    CompactTrait(TraitId),
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use exocore_chain::block::BlockOffset;
    use exocore_core::cell::LocalNode;
    use exocore_protos::store::{entity_mutation, OrderingValue};

    use super::{
        super::aggregator::tests::{mock_delete_entity, mock_delete_trait, mock_put_trait},
        *,
    };
    use crate::{
        local::{
            entity_index::aggregator::tests::mock_pending_delete,
            mutation_index::{MutationType, PutTraitMetadata},
        },
        ordering::OrderingValueWrapper,
    };

    #[test]
    fn collect_old_deleted_entity() {
        let node = LocalNode::generate();
        let now = Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
        let clock = Clock::new_fixed_mocked(now);

        let config = GarbageCollectorConfig {
            deleted_entity_collection: Duration::from_secs(1),
            min_operation_age: Duration::from_secs(1),
            ..Default::default()
        };
        let gc = GarbageCollector::new(config, clock.clone());

        let put_op: u64 = clock.consistent_time(node.node()).into();
        let del_op: u64 = put_op + 1;

        {
            // entity cannot be collected since deletion is not old enough
            let mutations = vec![
                mock_put_trait("trt1", "typ", Some(9), put_op, None, None),
                mock_delete_entity(Some(10), del_op),
            ];
            let aggregator = EntityAggregator::new(mutations.into_iter());
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 0);
        }

        // move time a second later
        clock.add_fixed_instant_duration(Duration::from_secs(2));

        {
            // delete operation is old enough, can be collected
            let mutations = vec![
                mock_put_trait("trt1", "typ", Some(9), put_op, None, None),
                mock_delete_entity(Some(10), del_op),
            ];

            let aggregator = EntityAggregator::new(mutations.iter().cloned());
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 1);

            let deletions = gc.run(|_id| Ok(mutations.iter().cloned()));
            assert_eq!(deletions.len(), 1);
            assert_eq!(extract_ops(deletions), vec![put_op, del_op]);
        }
    }

    #[test]
    fn collect_old_deleted_trait() {
        let node = LocalNode::generate();
        let now = Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
        let clock = Clock::new_fixed_mocked(now);

        let config = GarbageCollectorConfig {
            deleted_trait_collection: Duration::from_secs(1),
            min_operation_age: Duration::from_secs(1),
            ..Default::default()
        };
        let gc = GarbageCollector::new(config, clock.clone());

        let put1_op: u64 = clock.consistent_time(node.node()).into();
        let put2_op: u64 = put1_op + 1;
        let del_op: u64 = put2_op + 1;

        {
            // entity cannot be collected since deletion date is not 1 sec later
            let mutations = vec![
                mock_put_trait("trt1", "typ", Some(9), put1_op, None, None),
                mock_put_trait("trt2", "typ", Some(9), put2_op, None, None),
                mock_delete_trait("trt1", Some(10), del_op),
            ];
            let aggregator = EntityAggregator::new(mutations.into_iter());
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 0);
        }

        // move time a second later
        clock.add_fixed_instant_duration(Duration::from_secs(2));

        {
            // delete operation is old enough, can be collected
            let mutations = vec![
                mock_put_trait("trt1", "typ", Some(9), put1_op, None, None),
                mock_put_trait("trt2", "typ", Some(9), put2_op, None, None),
                mock_delete_trait("trt1", Some(10), del_op),
            ];
            let aggregator = EntityAggregator::new(mutations.iter().cloned());
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 1);

            let deletions = gc.run(|_entity_id| Ok(mutations.iter().cloned()));
            assert_eq!(deletions.len(), 1);
            assert_eq!(extract_ops(deletions), vec![put1_op, del_op]);
        }
    }

    #[test]
    fn collect_old_trait_versions() {
        let node = LocalNode::generate();
        let now = Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
        let clock = Clock::new_fixed_mocked(now);

        let put1_op: u64 = clock.consistent_time(node.node()).into();
        let put2_op: u64 = put1_op + 1;
        let put3_op: u64 = put2_op + 1;
        let put4_op: u64 = put3_op + 1;
        let put5_op: u64 = put4_op + 1;
        let mutations = vec![
            mock_put_trait("trt1", "typ", Some(2), put1_op, None, None),
            mock_put_trait("trt1", "typ", Some(3), put2_op, None, None),
            mock_put_trait("trt1", "typ", Some(4), put3_op, None, None),
            mock_put_trait("trt1", "typ", Some(5), put4_op, None, None),
            mock_put_trait("trt1", "typ", Some(6), put5_op, None, None),
        ];
        let aggregator = EntityAggregator::new(mutations.into_iter());

        // move time a second later
        clock.add_fixed_instant_duration(Duration::from_secs(2));

        // not enough versions to be collected
        let gc = GarbageCollector::new(
            GarbageCollectorConfig {
                trait_versions_max: 3,
                trait_versions_leeway: 6,
                min_operation_age: Duration::from_secs(1),
                ..Default::default()
            },
            clock.clone(),
        );
        gc.maybe_flag_for_collection(&aggregator);
        assert_queue_len(&gc, 0);

        // should now have enough versions to be collected
        let gc = GarbageCollector::new(
            GarbageCollectorConfig {
                trait_versions_max: 3,
                trait_versions_leeway: 4,
                min_operation_age: Duration::from_secs(1),
                ..Default::default()
            },
            clock,
        );
        gc.maybe_flag_for_collection(&aggregator);
        assert_queue_len(&gc, 1);

        // two first trait should get deleted
        let deletions = gc.run(|_entity_id| {
            Ok(vec![
                mock_metadata(Some(2), put1_op, "trt1"),
                mock_metadata(Some(3), put2_op, "trt1"),
                mock_metadata(Some(4), put3_op, "trt1"),
                mock_metadata(Some(5), put4_op, "trt1"),
                mock_metadata(Some(6), put5_op, "trt1"),
            ]
            .into_iter())
        });
        assert_eq!(deletions.len(), 1);
        assert_eq!(extract_ops(deletions), vec![put1_op, put2_op]);
    }

    #[test]
    fn cannot_collect_if_pending_deletion() {
        let node = LocalNode::generate();
        let now = Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
        let clock = Clock::new_fixed_mocked(now);

        let put1_op: u64 = clock.consistent_time(node.node()).into();
        let put2_op: u64 = put1_op + 1;
        let put3_op: u64 = put2_op + 1;
        let put4_op: u64 = put3_op + 1;
        let put5_op: u64 = put4_op + 1;
        let mutations = vec![
            mock_put_trait("trt1", "typ", Some(2), put1_op, None, None),
            mock_put_trait("trt1", "typ", Some(3), put2_op, None, None),
            mock_put_trait("trt1", "typ", Some(4), put3_op, None, None),
            mock_put_trait("trt1", "typ", Some(5), put4_op, None, None),
            mock_pending_delete(Some(6), put5_op),
        ];
        let aggregator = EntityAggregator::new(mutations.into_iter());

        // should be collecting, but one pending deletion, so won't collect
        let gc = GarbageCollector::new(
            GarbageCollectorConfig {
                trait_versions_max: 2,
                trait_versions_leeway: 3,
                min_operation_age: Duration::from_secs(1),
                ..Default::default()
            },
            clock,
        );
        gc.maybe_flag_for_collection(&aggregator);
        assert_queue_len(&gc, 0);
    }

    #[test]
    fn cannot_collect_if_any_operations_in_pending() {
        let node = LocalNode::generate();
        let now = Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
        let clock = Clock::new_fixed_mocked(now);

        let put1_op: u64 = clock.consistent_time(node.node()).into();
        let put2_op: u64 = put1_op + 1;
        let put3_op: u64 = put2_op + 1;
        let put4_op: u64 = put3_op + 1;
        let put5_op: u64 = put4_op + 1;
        let mutations = vec![
            mock_put_trait("trt1", "typ", Some(2), put1_op, None, None),
            mock_put_trait("trt1", "typ", Some(3), put2_op, None, None),
            mock_put_trait("trt1", "typ", Some(4), put3_op, None, None),
            mock_put_trait("trt1", "typ", Some(5), put4_op, None, None),
            mock_put_trait(
                "trt1", "typ", None, /* in pending */
                put5_op, None, None,
            ),
        ];
        let aggregator = EntityAggregator::new(mutations.into_iter());

        // should be collecting, but one operation in pending, so won't collect
        let gc = GarbageCollector::new(
            GarbageCollectorConfig {
                trait_versions_max: 2,
                trait_versions_leeway: 3,
                min_operation_age: Duration::from_secs(1),
                ..Default::default()
            },
            clock,
        );
        gc.maybe_flag_for_collection(&aggregator);
        assert_queue_len(&gc, 0);
    }

    fn assert_queue_len(gc: &GarbageCollector, len: usize) {
        let inner = gc.inner.read().unwrap();
        assert_eq!(inner.entity_ids.len(), len);
    }

    fn extract_ops(muts: Vec<EntityMutation>) -> Vec<OperationId> {
        muts.into_iter()
            .flat_map(|m| match m.mutation.unwrap() {
                entity_mutation::Mutation::DeleteOperations(del_mut) => del_mut.operation_ids,
                _ => vec![],
            })
            .collect()
    }

    fn mock_metadata<T: Into<TraitId>>(
        block_offset: Option<BlockOffset>,
        operation_id: OperationId,
        trait_id: T,
    ) -> MutationMetadata {
        MutationMetadata {
            operation_id,
            block_offset,
            entity_id: String::new(),
            mutation_type: MutationType::TraitPut(PutTraitMetadata {
                trait_id: trait_id.into(),
                trait_type: None,
                creation_date: None,
                modification_date: None,
                has_reference: false,
            }),
            sort_value: OrderingValueWrapper {
                value: OrderingValue {
                    operation_id: 0,
                    value: None,
                },
                reverse: false,
                ignore: false,
            },
        }
    }
}
