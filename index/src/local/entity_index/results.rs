use super::super::mutation_index::{MutationMetadata, MutationType, PutTraitMetadata};
use crate::entity::TraitId;
use crate::error::Error;
use crate::{ordering::OrderingValueWrapper, query::ResultHash};
use exocore_chain::operation::OperationId;
use exocore_core::protos::{
    generated::exocore_index::EntityResult as EntityResultProto,
    index::{Projection, Trait},
    reflect::{FieldId, MutableReflectMessage, ReflectMessage},
    registry::Registry,
};
use exocore_core::time::ConsistentTimestamp;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::{hash::Hasher, rc::Rc};

/// Wrapper for entity result with matched mutation from index layer along
/// aggregated traits.
pub struct EntityResult {
    pub matched_mutation: MutationMetadata,
    pub ordering_value: OrderingValueWrapper,
    pub proto: EntityResultProto,
    pub mutations: Rc<MutationAggregator>,
}

/// Aggregates mutations metadata of an entity retrieved from the mutations
/// index. Once merged, only the latest / active mutations are remaining, and
/// can then be fetched from the chain.
///
/// Operations are merged in order they got committed to the chain, and then
/// operation id within a block. If an old operation gets committed after a
/// newer operation, the old operation gets discarded to prevent inconsistency.
pub struct MutationAggregator {
    // final traits mutation metadata of the entity once all mutations were aggregated
    pub trait_mutations: HashMap<TraitId, MutationMetadata>,

    // traits' projections (ex: filter out fields) to be applied
    pub trait_projections: HashMap<TraitId, Rc<Projection>>,

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
            trait_mutations,
            trait_projections: HashMap::new(),
            active_operations: active_operation_ids,
            hash: hasher.finish(),
        })
    }

    /// Annotates each trait with projections that are matching them in a query.
    ///
    /// Projections allow returning only a subset of the traits or a part of its
    /// data. See `project_trait` method for the actual projections no the data
    /// of a retrieved trait.
    pub fn annotate_projections(&mut self, projections: &[Projection]) {
        if projections.is_empty() {
            return;
        }

        let projections_rc = projections.iter().map(|p| Rc::new(p.clone())).collect_vec();

        'traits_loop: for (trait_id, mutation_metadata) in &self.trait_mutations {
            match &mutation_metadata.mutation_type {
                MutationType::TraitPut(pm) => {
                    for projection in &projections_rc {
                        if projection_matches_trait(pm.trait_type.as_deref(), projection) {
                            self.trait_projections
                                .insert(trait_id.clone(), projection.clone());
                            continue 'traits_loop;
                        }
                    }
                }
                _other => {}
            }
        }
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

pub fn result_hasher() -> impl std::hash::Hasher {
    crc::crc64::Digest::new(crc::crc64::ECMA)
}

fn projection_matches_trait(trait_type: Option<&str>, projection: &Projection) -> bool {
    if projection.package.is_empty() {
        return true;
    }

    let trait_type = if let Some(trait_type) = trait_type {
        trait_type
    } else {
        return false;
    };

    for package in &projection.package {
        if (package.ends_with('$') && Some(trait_type) == package.strip_suffix('$'))
            || trait_type.starts_with(package)
        {
            return true;
        }
    }

    false
}

pub fn project_trait(
    registry: &Registry,
    trt: &mut Trait,
    projection: &Projection,
) -> Result<(), Error> {
    let any_msg = if let Some(msg) = &trt.message {
        msg
    } else {
        return Ok(());
    };

    // early exit if nothing to project since unmarshal+marshal is costly
    if projection.field_ids.is_empty() && projection.field_group_ids.is_empty() {
        return Ok(());
    }

    let mut dyn_msg = exocore_core::protos::reflect::from_prost_any(registry, any_msg)?;

    let field_ids_set: HashSet<FieldId> = HashSet::from_iter(projection.field_ids.iter().cloned());
    let field_groups_set: HashSet<FieldId> =
        HashSet::from_iter(projection.field_group_ids.iter().cloned());

    let mut fields_to_clear = Vec::new();
    for (field_id, field) in dyn_msg.fields() {
        let direct_field_match = field_ids_set.contains(field_id);
        let group_field_match = field
            .groups
            .iter()
            .any(|gid| field_groups_set.contains(gid));

        if !direct_field_match && !group_field_match {
            fields_to_clear.push(*field_id);
        }
    }

    if !fields_to_clear.is_empty() {
        for field_id in fields_to_clear {
            let _ = dyn_msg.clear_field_value(field_id);
        }
        trt.message = Some(dyn_msg.encode_to_prost_any()?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ordering::OrderingValueWrapper;
    use exocore_chain::block::BlockOffset;
    use exocore_core::protos::prost::ProstAnyPackMessageExt;
    use exocore_core::protos::{index::OrderingValue, reflect::FieldGroupId, test::TestMessage};
    use prost::Message;

    const TYPE1: &str = "exocore.test.TestMessage";
    const TYPE2: &str = "exocore.test.TestMessage2";

    #[test]
    fn mutations_ordering() {
        let t1 = "t1".to_string();
        let t2 = "t2".to_string();
        let t3 = "t3".to_string();

        let mutations = vec![
            mock_put_trait(&t1, TYPE1, Some(2), 3, None, None),
            mock_put_trait(&t3, TYPE1, Some(1), 0, None, None),
            mock_put_trait(&t1, TYPE1, Some(1), 2, None, None),
            mock_put_trait(&t2, TYPE1, Some(3), 5, None, None),
            mock_put_trait(&t3, TYPE1, None, 6, None, None),
            mock_put_trait(&t2, TYPE1, Some(1), 1, None, None),
            mock_put_trait(&t2, TYPE1, Some(2), 4, None, None),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.trait_mutations.get(&t1).unwrap().operation_id, 3);
        assert_eq!(em.trait_mutations.get(&t2).unwrap().operation_id, 5);
        assert_eq!(em.trait_mutations.get(&t3).unwrap().operation_id, 6);
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
            mock_put_trait(&t1, TYPE1, Some(1), 2, None, None),
            mock_put_trait(&t1, TYPE1, Some(2), 1, None, None),
        ];

        // operation 1 should be discarded, and only operation 2 active
        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.trait_mutations.get(&t1).unwrap().operation_id, 2);
        assert!(em.active_operations.contains(&2));
    }

    #[test]
    fn delete_trait() {
        let t1 = "t1".to_string();

        let mutations = vec![
            mock_put_trait(&t1, TYPE1, Some(1), 1, None, None),
            mock_delete_trait(&t1, Some(2), 2),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert!(em.trait_mutations.get(&t1).is_none());
        assert!(em.active_operations.contains(&2));
    }

    #[test]
    fn delete_trait_conflict() {
        let t1 = "t1".to_string();

        // delete operation 1 got committed after operation 2
        let mutations = vec![
            mock_put_trait(&t1, TYPE1, Some(1), 2, None, None),
            mock_delete_trait(&t1, Some(2), 1),
        ];

        // delete operation should be discarded since an operation happened on the trait
        // before it got committed
        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.trait_mutations.get(&t1).unwrap().operation_id, 2);
        assert!(em.active_operations.contains(&2))
    }

    #[test]
    fn delete_entity() {
        let t1 = "t1".to_string();

        let mutations = vec![
            mock_put_trait(&t1, TYPE1, Some(1), 1, None, None),
            mock_delete_entity(Some(2), 2),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert!(em.trait_mutations.get(&t1).is_none());
        assert!(em.active_operations.contains(&2));
    }

    #[test]
    fn mutations_delete_entity_conflict() {
        let t1 = "t1".to_string();

        // delete entity operation got committed after an newer operation
        let mutations = vec![
            mock_delete_entity(Some(1), 2),
            mock_put_trait(&t1, TYPE1, Some(2), 1, None, None),
        ];

        let em = MutationAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.trait_mutations.get(&t1).unwrap().operation_id, 1);
        assert!(em.active_operations.contains(&1));
    }

    #[test]
    fn merge_mutations_dates() {
        let t1 = "t1".to_string();

        let merge_mutations = |mutations: Vec<MutationMetadata>| -> MutationMetadata {
            let mut em = MutationAggregator::new(mutations.into_iter()).unwrap();
            em.trait_mutations.remove(&t1).unwrap()
        };

        {
            // if no dates specified, creation date is based on first operation
            let mutation =
                merge_mutations(vec![mock_put_trait(&t1, TYPE1, Some(1), 1, None, None)]);
            assert_creation_date(&mutation, Some(1));
        }

        {
            // if no dates specified, modification date is based on last operation
            let mutation = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 1, None, None),
                mock_put_trait(&t1, TYPE1, Some(2), 2, None, None),
            ]);
            assert_creation_date(&mutation, Some(1));
            assert_modification_date(&mutation, Some(2));
        }

        {
            // oldest specified creation date has priority
            let mutation = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 5, None, None),
                mock_put_trait(&t1, TYPE1, Some(2), 6, Some(1), None),
            ]);
            assert_creation_date(&mutation, Some(1));
            assert_modification_date(&mutation, Some(6));
        }

        {
            // last operation always override older specified modification date
            let mutation = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 5, None, Some(2)),
                mock_put_trait(&t1, TYPE1, Some(2), 6, None, None),
                mock_put_trait(&t1, TYPE1, Some(2), 7, None, None),
            ]);
            assert_modification_date(&mutation, Some(7));
        }
    }

    #[test]
    fn test_projection_matches_trait() {
        {
            // prefix match
            let proj1 = Projection {
                package: vec!["some.message".to_string()],
                ..Default::default()
            };

            assert!(projection_matches_trait(Some("some.message.Type"), &proj1));
            assert!(!projection_matches_trait(None, &proj1));
            assert!(!projection_matches_trait(
                Some("other.message.Type2"),
                &proj1
            ));
        }

        {
            // match all
            let proj2 = Projection {
                package: vec![],
                ..Default::default()
            };
            assert!(projection_matches_trait(Some("some.message.Type"), &proj2));
            assert!(projection_matches_trait(None, &proj2));
            assert!(projection_matches_trait(
                Some("other.message.Type2"),
                &proj2
            ));
        }

        {
            // exact match
            let proj1 = Projection {
                package: vec!["some.message.Type$".to_string()],
                ..Default::default()
            };

            assert!(projection_matches_trait(Some("some.message.Type"), &proj1));
            assert!(!projection_matches_trait(
                Some("some.message.Type2"),
                &proj1
            ));
        }
    }

    #[test]
    fn traits_projection_annotation() {
        let t1 = "t1";
        let t2 = "t2";

        fn assert_projection_id(em: &MutationAggregator, trait_id: &str, id: u32) {
            assert_eq!(em.trait_projections.get(trait_id).unwrap().field_ids[0], id,);
        }

        {
            // prefix match
            let mutations = vec![
                mock_put_trait(t1, TYPE1, Some(1), 1, None, None),
                mock_put_trait(t2, TYPE2, Some(1), 2, None, None),
            ];
            let mut em = MutationAggregator::new(mutations.into_iter()).unwrap();
            em.annotate_projections(&[Projection {
                package: vec!["exocore.test".to_string()],
                field_ids: vec![1],
                ..Default::default()
            }]);

            assert_projection_id(&em, t1, 1);
            assert_projection_id(&em, t2, 1);
        }

        {
            // exact match & catch-all
            let mutations = vec![
                mock_put_trait(t1, TYPE1, Some(1), 1, None, None),
                mock_put_trait(t2, TYPE2, Some(1), 2, None, None),
            ];
            let mut em = MutationAggregator::new(mutations.into_iter()).unwrap();
            em.annotate_projections(&[
                Projection {
                    package: vec![format!("{}$", TYPE1)],
                    field_ids: vec![1],
                    ..Default::default()
                },
                Projection {
                    field_ids: vec![2],
                    ..Default::default()
                },
            ]);

            assert_projection_id(&em, t1, 1);
            assert_projection_id(&em, t2, 2);
        }
    }

    #[test]
    fn traits_projection() -> anyhow::Result<()> {
        let registry = Registry::new_with_exocore_types();

        let msg = TestMessage {
            string1: "string1".to_string(),
            string2: "string2".to_string(),
            grouped1: "grouped1".to_string(),
            grouped2: "grouped2".to_string(),
            ..Default::default()
        };

        let project = |fields: Vec<FieldId>, groups: Vec<FieldGroupId>| -> TestMessage {
            let mut trt = Trait {
                message: Some(msg.pack_to_any().unwrap()),
                ..Default::default()
            };

            project_trait(
                &registry,
                &mut trt,
                &Projection {
                    field_ids: fields,
                    field_group_ids: groups,
                    ..Default::default()
                },
            )
            .unwrap();

            TestMessage::decode(trt.message.as_ref().unwrap().value.as_slice()).unwrap()
        };

        assert_eq!(
            project(vec![1], vec![]),
            TestMessage {
                string1: "string1".to_string(),
                ..Default::default()
            }
        );

        assert_eq!(
            project(vec![1, 2], vec![]),
            TestMessage {
                string1: "string1".to_string(),
                string2: "string2".to_string(),
                ..Default::default()
            }
        );

        assert_eq!(
            project(vec![2], vec![1]),
            TestMessage {
                string2: "string2".to_string(),
                grouped1: "grouped1".to_string(),
                grouped2: "grouped2".to_string(),
                ..Default::default()
            }
        );

        assert_eq!(
            project(vec![], vec![2]),
            TestMessage {
                grouped2: "grouped2".to_string(),
                ..Default::default()
            }
        );

        Ok(())
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

    fn mock_put_trait<I: Into<String>, T: Into<String>>(
        trait_id: I,
        trait_type: T,
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
                trait_type: Some(trait_type.into()),
                creation_date,
                modification_date,
            }),
            sort_value: OrderingValueWrapper {
                value: OrderingValue::default(),
                ignore: true,
                reverse: true,
            },
        }
    }

    fn mock_delete_trait<T: Into<String>>(
        trait_id: T,
        block_offset: Option<BlockOffset>,
        operation_id: OperationId,
    ) -> MutationMetadata {
        MutationMetadata {
            operation_id,
            block_offset,
            entity_id: String::new(),
            mutation_type: MutationType::TraitTombstone(trait_id.into()),
            sort_value: OrderingValueWrapper {
                value: OrderingValue::default(),
                ignore: true,
                reverse: true,
            },
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
            sort_value: OrderingValueWrapper {
                value: OrderingValue::default(),
                ignore: true,
                reverse: true,
            },
        }
    }
}
