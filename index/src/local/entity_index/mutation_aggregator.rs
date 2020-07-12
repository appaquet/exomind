use super::super::mutation_index::{MutationMetadata, MutationType, PutTraitMetadata};
use super::result_hasher;
use crate::entity::TraitId;
use crate::error::Error;
use crate::query::ResultHash;
use exocore_chain::operation::OperationId;
use exocore_core::time::ConsistentTimestamp;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

/// Aggregates mutations metadata of an entity retrieved from the mutations
/// index. Once merged, only the latest / active mutations are remaining, and
/// can then be fetched from the chain.
///
/// Operations are merged in order they got committed to the chain, and then
/// operation id within a block. If an old operation gets committed after a
/// newer operation, the old operation gets discarded to prevent inconsistency.
pub struct MutationAggregator {
    // final traits of the entity once all mutations were aggregated
    pub traits: HashMap<TraitId, MutationMetadata>,

    // ids of operations that are still active (ex: were not overridden by another mutation)
    pub active_operations: HashSet<OperationId>,

    // hash of the operations of the entity
    pub hash: ResultHash,
}

impl MutationAggregator {
    pub fn new<I>(mutations_metadata: I) -> Result<MutationAggregator, Error>
    where
        I: Iterator<Item = MutationMetadata>,
    {
        // Sort mutations in order they got committed (block offset/pending, then
        // operation id)
        let ordered_mutations_metadata = mutations_metadata.sorted_by_key(|result| {
            let block_offset = result.block_offset.unwrap_or(std::u64::MAX);
            (block_offset, result.operation_id)
        });

        let mut hasher = result_hasher();
        let mut trait_mutations = HashMap::<TraitId, MutationMetadata>::new();
        let mut active_operation_ids = HashSet::<OperationId>::new();
        let mut latest_operation_id = None;
        for mut current_mutation in ordered_mutations_metadata {
            let current_operation_id = current_mutation.operation_id;

            // hashing operations instead of traits content allow invalidating results as
            // soon as one operation is made since we can't guarantee anything
            hasher.write_u64(current_operation_id);

            match &mut current_mutation.mutation_type {
                MutationType::TraitPut(put_trait) => {
                    let opt_prev_trait_mutation = trait_mutations.get(&put_trait.trait_id);
                    if let Some(prev_trait_mutation) = opt_prev_trait_mutation {
                        // discard the new mutation if it happened before the last mutation, but got
                        // commited late, to prevent inconsistency
                        if current_operation_id < prev_trait_mutation.operation_id {
                            continue;
                        }

                        active_operation_ids.remove(&prev_trait_mutation.operation_id);
                    }

                    Self::merge_trait_metadata_dates(
                        current_operation_id,
                        put_trait,
                        opt_prev_trait_mutation,
                    );

                    active_operation_ids.insert(current_operation_id);
                    trait_mutations.insert(put_trait.trait_id.clone(), current_mutation);
                }
                MutationType::TraitTombstone(trait_id) => {
                    if let Some(prev_trait_mutation) = trait_mutations.get(trait_id) {
                        // discard the new mutation if it happened before the last mutation, but got
                        // commited late, to prevent inconsistency
                        if current_operation_id < prev_trait_mutation.operation_id {
                            continue;
                        }

                        active_operation_ids.remove(&prev_trait_mutation.operation_id);
                    }
                    active_operation_ids.insert(current_operation_id);
                    trait_mutations.remove(trait_id);
                }
                MutationType::EntityTombstone => {
                    if let Some(latest_operation_id) = latest_operation_id {
                        // discard the new mutation if it happened before the latest operation, but
                        // got committed late, to prevent inconsistency
                        if current_operation_id < latest_operation_id {
                            continue;
                        }
                    }

                    active_operation_ids.clear();
                    active_operation_ids.insert(current_operation_id);
                    trait_mutations.clear();
                }
            }

            if current_operation_id > latest_operation_id.unwrap_or(std::u64::MIN) {
                latest_operation_id = Some(current_operation_id);
            }
        }

        Ok(MutationAggregator {
            traits: trait_mutations,
            active_operations: active_operation_ids,
            hash: hasher.finish(),
        })
    }

    /// Populates the creation and modification dates of a PutTraitMetadata
    /// based on current operation id, previous version of the same trait
    /// and current dates data.
    fn merge_trait_metadata_dates(
        operation_id: OperationId,
        put_trait: &mut PutTraitMetadata,
        opt_prev_trait: Option<&MutationMetadata>,
    ) {
        let op_time = ConsistentTimestamp::from(operation_id).to_datetime();
        let mut creation_date;
        let mut modification_date = None;

        // dates on trait takes precedence
        if put_trait.creation_date.is_some() {
            creation_date = put_trait.creation_date;
        } else {
            creation_date = Some(op_time);
        }
        if put_trait.modification_date.is_some() {
            modification_date = put_trait.modification_date;
        }

        // if we currently have a mutation for this trait, we merge creation and
        // modifications dates so that creation date is the oldest date and
        // modification is the newest
        if let Some(prev_trait) = opt_prev_trait {
            if let MutationType::TraitPut(prev_trait) = &prev_trait.mutation_type {
                if modification_date.is_none() {
                    modification_date = Some(op_time);
                }

                if prev_trait.creation_date.is_some() && creation_date > prev_trait.creation_date {
                    creation_date = prev_trait.creation_date
                }
            }
        }

        // update the new trait creation date and modification date
        if put_trait.creation_date.is_none() || creation_date < put_trait.creation_date {
            put_trait.creation_date = creation_date;
        }
        if put_trait.modification_date.is_none() || modification_date > put_trait.modification_date
        {
            put_trait.modification_date = modification_date;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ordering::OrderingValueWrapper;
    use exocore_chain::block::BlockOffset;
    use exocore_core::protos::index::OrderingValue;
    use std::rc::Rc;

    #[test]
    fn mutations_ordering() {
        let t1 = "t1".to_string();
        let t2 = "t2".to_string();
        let t3 = "t3".to_string();

        let mutations = vec![
            mock_put_mutation(&t1, Some(2), 3, None, None),
            mock_put_mutation(&t3, Some(1), 0, None, None),
            mock_put_mutation(&t1, Some(1), 2, None, None),
            mock_put_mutation(&t2, Some(3), 5, None, None),
            mock_put_mutation(&t3, None, 6, None, None),
            mock_put_mutation(&t2, Some(1), 1, None, None),
            mock_put_mutation(&t2, Some(2), 4, None, None),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().operation_id, 3);
        assert_eq!(em.traits.get(&t2).unwrap().operation_id, 5);
        assert_eq!(em.traits.get(&t3).unwrap().operation_id, 6);
        assert_eq!(em.active_operations.len(), 3);
        assert!(em.active_operations.contains(&3));
        assert!(em.active_operations.contains(&5));
        assert!(em.active_operations.contains(&6));
    }

    #[test]
    fn put_trait_conflict() {
        let t1 = "t1".to_string();

        // operation 2 got committed before operation 1
        let mutations = vec![
            mock_put_mutation(&t1, Some(1), 2, None, None),
            mock_put_mutation(&t1, Some(2), 1, None, None),
        ];

        // operation 1 should be discarded, and only operation 2 active
        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().operation_id, 2);
        assert!(em.active_operations.contains(&2));
    }

    #[test]
    fn delete_trait() {
        let t1 = "t1".to_string();

        let mutations = vec![
            mock_put_mutation(&t1, Some(1), 1, None, None),
            mock_delete_mutation(&t1, Some(2), 2),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert!(em.traits.get(&t1).is_none());
        assert!(em.active_operations.contains(&2));
    }

    #[test]
    fn delete_trait_conflict() {
        let t1 = "t1".to_string();

        // delete operation 1 got committed after operation 2
        let mutations = vec![
            mock_put_mutation(&t1, Some(1), 2, None, None),
            mock_delete_mutation(&t1, Some(2), 1),
        ];

        // delete operation should be discarded since an operation happened on the trait
        // before it got committed
        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().operation_id, 2);
        assert!(em.active_operations.contains(&2))
    }

    #[test]
    fn delete_entity() {
        let t1 = "t1".to_string();

        let mutations = vec![
            mock_put_mutation(&t1, Some(1), 1, None, None),
            mock_delete_entity(Some(2), 2),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert!(em.traits.get(&t1).is_none());
        assert!(em.active_operations.contains(&2));
    }

    #[test]
    fn mutations_delete_entity_conflict() {
        let t1 = "t1".to_string();

        // delete entity operation got committed after an newer operation
        let mutations = vec![
            mock_delete_entity(Some(1), 2),
            mock_put_mutation(&t1, Some(2), 1, None, None),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().operation_id, 1);
        assert!(em.active_operations.contains(&1));
    }

    #[test]
    fn merge_mutations_dates() {
        let t1 = "t1".to_string();

        let merge_mutations = |mutations: Vec<MutationMetadata>| -> MutationMetadata {
            let mut em = MutationAggregator::new(mutations.into_iter()).unwrap();
            em.traits.remove(&t1).unwrap()
        };

        {
            // if no dates specified, creation date is based on first operation
            let mutation = merge_mutations(vec![mock_put_mutation(&t1, Some(1), 1, None, None)]);
            assert_creation_date(&mutation, Some(1));
        }

        {
            // if no dates specified, modification date is based on last operation
            let mutation = merge_mutations(vec![
                mock_put_mutation(&t1, Some(1), 1, None, None),
                mock_put_mutation(&t1, Some(2), 2, None, None),
            ]);
            assert_creation_date(&mutation, Some(1));
            assert_modification_date(&mutation, Some(2));
        }

        {
            // oldest specified creation date has priority
            let mutation = merge_mutations(vec![
                mock_put_mutation(&t1, Some(1), 5, None, None),
                mock_put_mutation(&t1, Some(2), 6, Some(1), None),
            ]);
            assert_creation_date(&mutation, Some(1));
            assert_modification_date(&mutation, Some(6));
        }

        {
            // last operation always override older specified modification date
            let mutation = merge_mutations(vec![
                mock_put_mutation(&t1, Some(1), 5, None, Some(2)),
                mock_put_mutation(&t1, Some(2), 6, None, None),
                mock_put_mutation(&t1, Some(2), 7, None, None),
            ]);
            assert_modification_date(&mutation, Some(7));
        }
    }

    fn assert_creation_date(mutation: &MutationMetadata, time_op_id: Option<OperationId>) {
        match &mutation.mutation_type {
            MutationType::TraitPut(put_mut) => {
                let date = time_op_id.map(|op_id| ConsistentTimestamp::from(op_id).to_datetime());
                assert_eq!(put_mut.creation_date, date);
            }
            other => {
                panic!("Expected put trait mutation, got {:?}", other);
            }
        }
    }

    fn assert_modification_date(mutation: &MutationMetadata, time_op_id: Option<OperationId>) {
        match &mutation.mutation_type {
            MutationType::TraitPut(put_mut) => {
                let date = time_op_id.map(|op_id| ConsistentTimestamp::from(op_id).to_datetime());
                assert_eq!(put_mut.modification_date, date);
            }
            other => {
                panic!("Expected put trait mutation, got {:?}", other);
            }
        }
    }

    fn mock_put_mutation<T: Into<String>>(
        trait_id: T,
        block_offset: Option<BlockOffset>,
        operation_id: OperationId,
        created_time_op_id: Option<OperationId>,
        modification_time_op_id: Option<OperationId>,
    ) -> MutationMetadata {
        let creation_date =
            created_time_op_id.map(|op_id| ConsistentTimestamp::from(op_id).to_datetime());
        let modification_date =
            modification_time_op_id.map(|op_id| ConsistentTimestamp::from(op_id).to_datetime());

        MutationMetadata {
            operation_id,
            block_offset,
            entity_id: String::new(),
            mutation_type: MutationType::TraitPut(PutTraitMetadata {
                trait_id: trait_id.into(),
                creation_date,
                modification_date,
            }),
            sort_value: Rc::new(OrderingValueWrapper {
                value: OrderingValue::default(),
                ignore: true,
                reverse: true,
            }),
        }
    }

    fn mock_delete_mutation<T: Into<String>>(
        trait_id: T,
        block_offset: Option<BlockOffset>,
        operation_id: OperationId,
    ) -> MutationMetadata {
        MutationMetadata {
            operation_id,
            block_offset,
            entity_id: String::new(),
            mutation_type: MutationType::TraitTombstone(trait_id.into()),
            sort_value: Rc::new(OrderingValueWrapper {
                value: OrderingValue::default(),
                ignore: true,
                reverse: true,
            }),
        }
    }

    fn mock_delete_entity(
        block_offset: Option<BlockOffset>,
        operation_id: OperationId,
    ) -> MutationMetadata {
        MutationMetadata {
            operation_id,
            block_offset,
            entity_id: String::new(),
            mutation_type: MutationType::EntityTombstone,
            sort_value: Rc::new(OrderingValueWrapper {
                value: OrderingValue::default(),
                ignore: true,
                reverse: true,
            }),
        }
    }
}
