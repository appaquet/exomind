use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
    time::Duration,
};

use exocore_chain::{block::BlockOffset, operation::OperationId};
use exocore_core::time::{Clock, ConsistentTimestamp};
use exocore_protos::{core::EntityGarbageCollectorConfig, store::EntityMutation};

use super::{sort_mutations_commit_time, EntityAggregator};
use crate::{
    entity::{EntityId, EntityIdRef, TraitId},
    error::Error,
    local::mutation_index::{EntityMutationResults, MutationMetadata},
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
/// can delete mutations by operation ids as it indexes them.
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
/// collection run. This run only happen if there has been sufficient number of
/// indexed blocks from chain so that we don't re-collect entities for which we
/// have mutations that have already been deleted in the chain index, but not
/// indexed yet because they are in the pending store.
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
            operations: Vec::with_capacity(config.queue_size),
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
        if aggregator.in_pending || aggregator.pending_deletion {
            // we don't collect if any of the operations are still in pending store or if
            // a deletion is pending for the entity
            return;
        }

        let last_block_offset = aggregator
            .last_block_offset
            .expect("No last_block_offset, but no operations in pending");

        let now = self.clock.now_chrono();

        if let Some(deletion_date) = &aggregator.deletion_date {
            let elapsed = now.signed_duration_since(*deletion_date);
            if elapsed >= self.max_entity_deleted_duration {
                trace!(
                    "Collecting entity {} since got deleted since",
                    aggregator.entity_id
                );
                let mut inner = self.inner.write().expect("Fail to acquire inner lock");
                inner.maybe_enqueue(Operation::DeleteEntity(
                    last_block_offset,
                    aggregator.entity_id.to_string(),
                ));
                return;
            }
        }

        for (trait_id, trait_aggr) in &aggregator.traits {
            if let Some(deletion_date) = &trait_aggr.deletion_date {
                let elapsed = now.signed_duration_since(*deletion_date);
                if elapsed >= self.max_trait_deleted_duration {
                    trace!(
                        "Collecting entity {} trait {} since it got deleted",
                        aggregator.entity_id,
                        trait_id
                    );
                    let mut inner = self.inner.write().expect("Fail to acquire inner lock");
                    inner.maybe_enqueue(Operation::DeleteTrait(
                        last_block_offset,
                        aggregator.entity_id.to_string(),
                        trait_id.clone(),
                    ));
                    continue;
                }
            }

            if trait_aggr.mutation_count > self.config.trait_versions_leeway {
                let mut inner = self.inner.write().expect("Fail to acquire inner lock");
                trace!(
                    "Collecting entity {} trait {} since it has too many versions ({})",
                    aggregator.entity_id,
                    trait_id,
                    trait_aggr.mutation_count,
                );
                inner.maybe_enqueue(Operation::CompactTrait(
                    last_block_offset,
                    aggregator.entity_id.to_string(),
                    trait_id.clone(),
                ));
            }
        }
    }

    /// Garbage collect the entities currently in queue.
    pub fn run<F>(&self, entity_fetcher: F) -> Vec<EntityMutation>
    where
        F: Fn(EntityIdRef) -> Result<EntityMutationResults, Error>,
    {
        let (operations, entity_count) = {
            let mut inner = self.inner.write().expect("Fail to acquire inner lock");
            let entity_count = inner.entity_ids.len();

            inner.entity_ids.clear();
            let mut queue = Vec::with_capacity(self.config.queue_size);
            std::mem::swap(&mut queue, &mut inner.operations);

            (queue, entity_count)
        };

        if operations.is_empty() {
            return Vec::new();
        }

        let min_op_time = self.clock.now_chrono() - self.trait_versions_min_age;

        debug!(
            "Starting a garbage collection pass with {} operations for {} entities...",
            operations.len(),
            entity_count,
        );
        let mut deletions = Vec::new();
        for op in operations {
            let until_block_offset = op.until_block_offset();
            let entity_id = op.entity_id();

            // get all mutations for entity until block offset where we deemed the entity to
            // be collectable and until minimum operation age.
            let mutations = if let Ok(mut_res) = entity_fetcher(entity_id) {
                let mutations = mut_res.mutations.into_iter().take_while(|mutation| {
                    let op_time = ConsistentTimestamp::from(mutation.operation_id).to_datetime();
                    if op_time > min_op_time {
                        return false;
                    }

                    matches!(mutation.block_offset, Some(offset) if offset <= until_block_offset)
                });

                sort_mutations_commit_time(mutations)
            } else {
                error!("Couldn't fetch mutations for entity {}", entity_id);
                continue;
            };

            let opt_deletion = match &op {
                Operation::DeleteEntity(_, entity_id) => {
                    collect_delete_entity(entity_id, mutations)
                }
                Operation::DeleteTrait(_, entity_id, trait_id) => {
                    collect_delete_trait(entity_id, trait_id, mutations)
                }
                Operation::CompactTrait(_, entity_id, trait_id) => collect_trait_versions(
                    entity_id,
                    trait_id,
                    self.config.trait_versions_max,
                    mutations,
                ),
            };

            if let Some(deletion) = opt_deletion {
                deletions.push(deletion);
            }
        }

        if !deletions.is_empty() {
            info!(
                "Garbage collection generated {} deletion operations for {} entities",
                deletions.len(),
                entity_count,
            );
        }

        deletions
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
fn collect_delete_entity<I>(entity_id: &str, mutations: I) -> Option<EntityMutation>
where
    I: Iterator<Item = MutationMetadata>,
{
    let operation_ids: Vec<OperationId> = mutations.map(|mutation| mutation.operation_id).collect();
    if operation_ids.is_empty() {
        return None;
    }

    debug!(
        "Creating delete operation to garbage collect deleted entity {} with operations {:?}",
        entity_id, operation_ids,
    );
    MutationBuilder::new()
        .delete_operations(entity_id, operation_ids)
        .build()
        .mutations
        .into_iter()
        .next()
}

/// Creates a deletion mutation for all operations of a trait of an entity.
fn collect_delete_trait<I>(entity_id: &str, trait_id: &str, mutations: I) -> Option<EntityMutation>
where
    I: Iterator<Item = MutationMetadata>,
{
    let trait_mutations = filter_trait_mutations(mutations, trait_id);
    let operation_ids: Vec<OperationId> = trait_mutations
        .map(|mutation| mutation.operation_id)
        .collect();
    if operation_ids.is_empty() {
        return None;
    }

    debug!(
        "Creating delete operation to garbage collect deleted trait {} of entity {} with operations {:?}",
        trait_id, entity_id, operation_ids,
    );
    MutationBuilder::new()
        .delete_operations(entity_id, operation_ids)
        .build()
        .mutations
        .into_iter()
        .next()
}

/// Creates a deletion mutation for the N oldest operations of a trait so that
/// we only keep the most recent ones.
fn collect_trait_versions<I>(
    entity_id: &str,
    trait_id: &str,
    max_versions: usize,
    mutations: I,
) -> Option<EntityMutation>
where
    I: Iterator<Item = MutationMetadata>,
{
    let trait_operations: Vec<OperationId> = filter_trait_mutations(mutations, trait_id)
        .map(|mutation| mutation.operation_id)
        .collect();

    if trait_operations.is_empty() || trait_operations.len() <= max_versions {
        return None;
    }

    let to_delete_count = trait_operations.len() - max_versions;
    let operation_ids = trait_operations.into_iter().take(to_delete_count).collect();

    debug!(
        "Creating delete operation to garbage collect operations {:?} of trait {} of entity {}",
        operation_ids, trait_id, entity_id
    );
    MutationBuilder::new()
        .delete_operations(entity_id, operation_ids)
        .build()
        .mutations
        .into_iter()
        .next()
}

/// Filters mutations to only return those that are for a specified trait.
fn filter_trait_mutations<'i, I>(
    mutations: I,
    trait_id: &'i str,
) -> impl Iterator<Item = MutationMetadata> + 'i
where
    I: Iterator<Item = MutationMetadata> + 'i,
{
    use crate::local::mutation_index::MutationType;
    mutations.filter(move |mutation| match &mutation.mutation_type {
        MutationType::TraitPut(put_mut) if put_mut.trait_id == trait_id => true,
        MutationType::TraitTombstone(tomb_trait_id) if tomb_trait_id == trait_id => true,
        _ => false,
    })
}

struct Inner {
    config: GarbageCollectorConfig,
    operations: Vec<Operation>,
    entity_ids: HashSet<EntityId>,
}

impl Inner {
    fn is_full(&self) -> bool {
        self.operations.len() > self.config.queue_size
    }

    fn maybe_enqueue(&mut self, op: Operation) {
        if self.is_full() {
            return;
        }

        if self.entity_ids.contains(op.entity_id()) {
            return;
        }

        self.entity_ids.insert(op.entity_id().to_string());
        self.operations.push(op);
    }
}

enum Operation {
    DeleteEntity(BlockOffset, EntityId),
    DeleteTrait(BlockOffset, EntityId, TraitId),
    CompactTrait(BlockOffset, EntityId, TraitId),
}

impl Operation {
    #[inline]
    fn entity_id(&self) -> EntityIdRef {
        match self {
            Operation::DeleteEntity(_, entity_id) => entity_id,
            Operation::DeleteTrait(_, entity_id, _) => entity_id,
            Operation::CompactTrait(_, entity_id, _) => entity_id,
        }
    }

    #[inline]
    fn until_block_offset(&self) -> BlockOffset {
        match self {
            Operation::DeleteEntity(offset, _) => *offset,
            Operation::DeleteTrait(offset, _, _) => *offset,
            Operation::CompactTrait(offset, _, _) => *offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

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
        let now = Instant::now() - Duration::from_secs(60);
        let clock = Clock::new_fixed_mocked(now);

        let config = GarbageCollectorConfig {
            deleted_entity_collection: Duration::from_secs(1),
            min_operation_age: Duration::from_secs(1),
            ..Default::default()
        };
        let gc = GarbageCollector::new(config, clock.clone());

        let put_op: u64 = clock.consistent_time(node.node()).into();
        let del_op: u64 = put_op + 1;
        let mutations = vec![
            mock_put_trait("trt1", "typ", Some(9), put_op, None, None),
            mock_delete_entity(Some(10), del_op),
        ];
        let aggregator = EntityAggregator::new(mutations.into_iter()).unwrap();

        {
            // entity cannot be collected since deletion date is not 1 sec later
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 0);
        }

        // move time a second later
        clock.add_fixed_instant_duration(Duration::from_secs(2));

        {
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 1);

            // operation considered not in chain, shouldn't get deleted
            let deletions = gc.run(|_entity_id| {
                Ok(EntityMutationResults {
                    mutations: vec![
                        mock_metadata(None, put_op, "trt1"),
                        mock_metadata(None, del_op, "trt1"),
                    ],
                })
            });
            assert_eq!(deletions.len(), 0);
        }

        {
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 1);

            // operation considered now in chain, should get deleted
            let deletions = gc.run(|_entity_id| {
                Ok(EntityMutationResults {
                    mutations: vec![
                        mock_metadata(Some(9), put_op, "trt1"),
                        mock_metadata(Some(10), del_op, "trt1"),
                    ],
                })
            });
            assert_eq!(deletions.len(), 1);
            assert_eq!(extract_ops(deletions), vec![put_op, del_op]);
        }
    }

    #[test]
    fn collect_old_deleted_trait() {
        let node = LocalNode::generate();
        let now = Instant::now() - Duration::from_secs(60);
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
        let mutations = vec![
            mock_put_trait("trt1", "typ", Some(9), put1_op, None, None),
            mock_put_trait("trt2", "typ", Some(9), put2_op, None, None),
            mock_delete_trait("trt1", Some(10), del_op),
        ];
        let aggregator = EntityAggregator::new(mutations.into_iter()).unwrap();

        {
            // entity cannot be collected since deletion date is not 1 sec later
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 0);
        }

        // move time a second later
        clock.add_fixed_instant_duration(Duration::from_secs(2));

        {
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 1);

            // operation considered not in chain, shouldn't get deleted
            let deletions = gc.run(|_entity_id| {
                Ok(EntityMutationResults {
                    mutations: vec![
                        mock_metadata(None, put1_op, "trt1"),
                        mock_metadata(None, del_op, "trt1"),
                    ],
                })
            });
            assert_eq!(deletions.len(), 0);
        }

        {
            gc.maybe_flag_for_collection(&aggregator);
            assert_queue_len(&gc, 1);

            // operation considered now in chain, should get deleted
            let deletions = gc.run(|_entity_id| {
                Ok(EntityMutationResults {
                    mutations: vec![
                        mock_metadata(Some(9), put1_op, "trt1"),
                        mock_metadata(Some(10), del_op, "trt1"),
                    ],
                })
            });
            assert_eq!(deletions.len(), 1);
            assert_eq!(extract_ops(deletions), vec![put1_op, del_op]);
        }
    }

    #[test]
    fn collect_old_trait_versions() {
        let node = LocalNode::generate();
        let now = Instant::now() - Duration::from_secs(60);
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
        let aggregator = EntityAggregator::new(mutations.into_iter()).unwrap();

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
            Ok(EntityMutationResults {
                mutations: vec![
                    mock_metadata(Some(2), put1_op, "trt1"),
                    mock_metadata(Some(3), put2_op, "trt1"),
                    mock_metadata(Some(4), put3_op, "trt1"),
                    mock_metadata(Some(5), put4_op, "trt1"),
                    mock_metadata(Some(6), put5_op, "trt1"),
                ],
            })
        });
        assert_eq!(deletions.len(), 1);
        assert_eq!(extract_ops(deletions), vec![put1_op, put2_op]);
    }

    #[test]
    fn cannot_collect_if_pending_deletion() {
        let node = LocalNode::generate();
        let now = Instant::now() - Duration::from_secs(60);
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
        let aggregator = EntityAggregator::new(mutations.into_iter()).unwrap();

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
        let now = Instant::now() - Duration::from_secs(60);
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
        let aggregator = EntityAggregator::new(mutations.into_iter()).unwrap();

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
        assert_eq!(inner.operations.len(), len);
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
