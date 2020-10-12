use super::super::mutation_index::{MutationMetadata, MutationType, PutTraitMetadata};
use crate::entity::TraitId;
use crate::error::Error;
use crate::query::ResultHash;
use chrono::{DateTime, Utc};
use exocore_chain::operation::OperationId;
use exocore_core::protos::{
    reflect::{FieldId, MutableReflectMessage, ReflectMessage},
    registry::Registry,
    store::{Projection, Trait, TraitDetails},
};
use exocore_core::time::ConsistentTimestamp;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::{hash::Hasher, rc::Rc};

/// Aggregates mutations metadata of an entity retrieved from the mutations
/// index. Once merged, only the latest / active mutations are remaining, and
/// can then be fetched from the chain.
///
/// Operations are merged in order they got committed to the chain, and then
/// operation id within a block. If an old operation gets committed after a
/// newer operation, the old operation gets discarded to prevent inconsistency.
pub struct EntityAggregator {
    /// traits aggregator
    pub traits: HashMap<TraitId, TraitAggregator>,

    // ids of operations that are still active (ex: were not overridden by another mutation)
    pub active_operations: HashSet<OperationId>,

    // hash of the operations of the entity
    pub hash: ResultHash,

    // date of the creation of entity, based on put trait creation date OR first operation
    pub creation_date: Option<DateTime<Utc>>,

    // date of the modification of entity, based on put trait modifcation date OR last operation
    pub modification_date: Option<DateTime<Utc>>,

    // date of the deletion of the entity if all traits are deleted once all mutations are applied
    pub deletion_date: Option<DateTime<Utc>>,

    pub last_operatin_id: OperationId,
}

impl EntityAggregator {
    pub fn new<I>(mutations_metadata: I) -> Result<EntityAggregator, Error>
    where
        I: Iterator<Item = MutationMetadata>,
    {
        // Sort mutations in order they got committed (block offset/pending, then
        // operation id)
        let ordered_mutations_metadata = mutations_metadata.sorted_by_key(|result| {
            let block_offset = result.block_offset.unwrap_or(std::u64::MAX);
            (block_offset, result.operation_id)
        });

        let mut entity_creation_date = None;
        let mut entity_modification_date = None;
        let mut entity_deletion_date = None;

        let mut hasher = result_hasher();
        let mut traits = HashMap::<TraitId, TraitAggregator>::new();
        let mut active_operation_ids = HashSet::<OperationId>::new();
        let mut last_operation_id = None;

        for current_mutation in ordered_mutations_metadata {
            let current_operation_id = current_mutation.operation_id;
            let current_operation_time =
                ConsistentTimestamp::from(current_operation_id).to_datetime();

            // hashing operations instead of traits content allow invalidating results as
            // soon as one operation is made since we can't guarantee anything
            hasher.write_u64(current_operation_id);

            match &current_mutation.mutation_type {
                MutationType::TraitPut(put_trait) => {
                    let agg = TraitAggregator::get_for_trait(&mut traits, &put_trait.trait_id);

                    if let Some(last_operation_id) = agg.last_operation_id {
                        // discard the new mutation if it happened before the last mutation, but got
                        // commited late, to prevent inconsistency
                        if current_operation_id < last_operation_id {
                            continue;
                        }

                        active_operation_ids.remove(&last_operation_id);
                    }

                    agg.push_put_mutation(current_mutation);
                    active_operation_ids.insert(current_operation_id);

                    update_if_older(&mut entity_creation_date, agg.creation_date);
                    update_if_newer(&mut entity_modification_date, agg.modification_date);
                    entity_deletion_date = None;
                }
                MutationType::TraitTombstone(trait_id) => {
                    let agg = TraitAggregator::get_for_trait(&mut traits, trait_id);

                    if let Some(last_operation_id) = agg.last_operation_id {
                        // discard the new mutation if it happened before the last mutation, but got
                        // commited late, to prevent inconsistency
                        if current_operation_id < last_operation_id {
                            continue;
                        }

                        active_operation_ids.remove(&last_operation_id);
                    }

                    active_operation_ids.insert(current_operation_id);
                    agg.push_delete_mutation(current_operation_id);

                    update_if_newer(&mut entity_modification_date, Some(current_operation_time));

                    if TraitAggregator::all_deleted(&traits) {
                        entity_creation_date = None;
                        entity_modification_date = None;
                        entity_deletion_date = Some(current_operation_time);
                    }
                }
                MutationType::EntityTombstone => {
                    if let Some(latest_operation_id) = last_operation_id {
                        // discard the new mutation if it happened before the latest operation, but
                        // got committed late, to prevent inconsistency
                        if current_operation_id < latest_operation_id {
                            continue;
                        }
                    }

                    for aggregated_trait in traits.values_mut() {
                        aggregated_trait.push_delete_mutation(current_operation_id);
                    }

                    active_operation_ids.clear();
                    active_operation_ids.insert(current_operation_id);

                    entity_creation_date = None;
                    entity_modification_date = None;
                    entity_deletion_date = Some(current_operation_time);
                }
            }

            if current_operation_id > last_operation_id.unwrap_or(std::u64::MIN) {
                last_operation_id = Some(current_operation_id);
            }
        }

        Ok(EntityAggregator {
            traits,
            active_operations: active_operation_ids,
            hash: hasher.finish(),
            creation_date: entity_creation_date,
            modification_date: entity_modification_date,
            deletion_date: entity_deletion_date,
            last_operatin_id: last_operation_id.unwrap_or_default(),
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

        'traits_loop: for trait_agg in self.traits.values_mut() {
            if let Some((_mutation, pm)) = trait_agg.last_put_mutation() {
                for projection in &projections_rc {
                    if projection_matches_trait(pm.trait_type.as_deref(), projection) {
                        trait_agg.projection = Some(projection.clone());
                        continue 'traits_loop;
                    }
                }
            }
        }
    }
}

#[derive(Default)]
/// Aggregates mutations metadata of an entity's trait retrieved from the
/// mutations index. Once merged, only the latest / active mutations are
/// remaining, and can then be fetched from the chain.
pub struct TraitAggregator {
    pub put_mutations: Vec<MutationMetadata>,
    pub last_operation_id: Option<OperationId>,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
    pub deletion_date: Option<DateTime<Utc>>,
    pub projection: Option<Rc<Projection>>,
}

impl TraitAggregator {
    fn get_for_trait<'t>(
        traits: &'t mut HashMap<TraitId, TraitAggregator>,
        trait_id: &str,
    ) -> &'t mut TraitAggregator {
        if !traits.contains_key(trait_id) {
            traits.insert(trait_id.to_string(), TraitAggregator::default());
        }

        traits.get_mut(trait_id).unwrap()
    }

    fn all_deleted(traits: &HashMap<TraitId, TraitAggregator>) -> bool {
        traits.values().all(|t| t.deletion_date.is_some())
    }

    fn push_put_mutation(&mut self, mutation: MutationMetadata) {
        let op_id = mutation.operation_id;
        let op_time = ConsistentTimestamp::from(op_id).to_datetime();

        let put_trait = if let MutationType::TraitPut(put_trait) = &mutation.mutation_type {
            put_trait
        } else {
            return;
        };

        let modification_date = if let Some(modification_date) = put_trait.modification_date {
            Some(modification_date)
        } else if self.creation_date.is_some() {
            Some(op_time)
        } else {
            None
        };
        update_if_newer(&mut self.modification_date, modification_date);

        let creation_date = put_trait.creation_date.unwrap_or(op_time);
        update_if_older(&mut self.creation_date, Some(creation_date));

        self.deletion_date = None;

        self.put_mutations.push(mutation);
        self.last_operation_id = Some(op_id);
    }

    fn push_delete_mutation(&mut self, operation_id: OperationId) {
        let op_time = ConsistentTimestamp::from(operation_id).to_datetime();
        self.creation_date = None;
        self.modification_date = None;
        self.deletion_date = Some(op_time);
        self.last_operation_id = Some(operation_id);
    }

    pub fn last_put_mutation(&self) -> Option<(&MutationMetadata, &PutTraitMetadata)> {
        let mutation = self.put_mutations.last()?;

        match &mutation.mutation_type {
            MutationType::TraitPut(put) => Some((mutation, put)),
            _ => None,
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
        trt.details = TraitDetails::Partial.into();
    }

    Ok(())
}

fn update_if_newer(current: &mut Option<DateTime<Utc>>, new: Option<DateTime<Utc>>) {
    if current.is_none() || new > *current {
        *current = new;
    }
}

fn update_if_older(current: &mut Option<DateTime<Utc>>, new: Option<DateTime<Utc>>) {
    if current.is_none() || new < *current {
        *current = new;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ordering::OrderingValueWrapper;
    use exocore_chain::block::BlockOffset;
    use exocore_core::protos::prost::ProstAnyPackMessageExt;
    use exocore_core::protos::{reflect::FieldGroupId, store::OrderingValue, test::TestMessage};
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

        let em = EntityAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().last_operation_id, Some(3));
        assert_eq!(em.traits.get(&t2).unwrap().last_operation_id, Some(5));
        assert_eq!(em.traits.get(&t3).unwrap().last_operation_id, Some(6));
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
        let em = EntityAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().last_operation_id, Some(2));
        assert!(em.active_operations.contains(&2));
    }

    #[test]
    fn delete_trait() {
        let t1 = "t1".to_string();

        let mutations = vec![
            mock_put_trait(&t1, TYPE1, Some(1), 1, None, None),
            mock_delete_trait(&t1, Some(2), 2),
        ];

        let em = EntityAggregator::new(mutations.into_iter()).unwrap();
        assert!(em.deletion_date.is_some());
        assert!(em.traits.get(&t1).unwrap().deletion_date.is_some());
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
        let em = EntityAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().last_operation_id, Some(2));
        assert!(em.active_operations.contains(&2))
    }

    #[test]
    fn delete_entity() {
        let t1 = "t1".to_string();

        let mutations = vec![
            mock_put_trait(&t1, TYPE1, Some(1), 1, None, None),
            mock_delete_entity(Some(2), 2),
        ];

        let em = EntityAggregator::new(mutations.into_iter()).unwrap();
        assert!(em.deletion_date.is_some());
        assert!(em.traits.get(&t1).unwrap().deletion_date.is_some());
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

        let em = EntityAggregator::new(mutations.into_iter()).unwrap();
        assert_eq!(em.traits.get(&t1).unwrap().last_operation_id, Some(1));
        assert!(em.active_operations.contains(&1));
    }

    #[test]
    fn trait_dates() {
        let t1 = "t1".to_string();

        let merge_mutations = |mutations: Vec<MutationMetadata>| -> TraitAggregator {
            let mut em = EntityAggregator::new(mutations.into_iter()).unwrap();
            em.traits.remove(&t1).unwrap()
        };

        fn assert_dates(
            agg: &TraitAggregator,
            creation: Option<OperationId>,
            modification: Option<OperationId>,
            deletion: Option<OperationId>,
        ) {
            assert_eq!(
                agg.creation_date,
                creation.map(|c| ConsistentTimestamp::from(c).to_datetime()),
            );
            assert_eq!(
                agg.modification_date,
                modification.map(|m| ConsistentTimestamp::from(m).to_datetime()),
            );
            assert_eq!(
                agg.deletion_date,
                deletion.map(|m| ConsistentTimestamp::from(m).to_datetime()),
            );
        }

        {
            // if no dates specified, creation date is based on first operation
            let agg = merge_mutations(vec![mock_put_trait(&t1, TYPE1, Some(1), 1, None, None)]);
            assert_dates(&agg, Some(1), None, None);
        }

        {
            // if no dates specified, modification date is based on last operation
            let agg = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 1, None, None),
                mock_put_trait(&t1, TYPE1, Some(2), 2, None, None),
            ]);
            assert_dates(&agg, Some(1), Some(2), None);
        }

        {
            // oldest specified creation date has priority
            let agg = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 5, None, None),
                mock_put_trait(&t1, TYPE1, Some(2), 6, Some(1), None),
            ]);
            assert_dates(&agg, Some(1), Some(6), None);
        }

        {
            // last operation always override older specified modification date
            let agg = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 5, None, Some(2)),
                mock_put_trait(&t1, TYPE1, Some(2), 6, None, None),
                mock_put_trait(&t1, TYPE1, Some(2), 7, None, None),
            ]);
            assert_dates(&agg, Some(5), Some(7), None);
        }

        {
            // deleting trait should reset dates & mark as deleted
            let agg = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 5, None, Some(2)),
                mock_delete_trait(&t1, Some(2), 6),
            ]);
            assert_dates(&agg, None, None, Some(6));
        }

        {
            // deleting entity should reset dates & mark as deleted
            let agg = merge_mutations(vec![
                mock_put_trait(&t1, TYPE1, Some(1), 5, None, Some(2)),
                mock_delete_entity(Some(2), 6),
            ]);
            assert_dates(&agg, None, None, Some(6));
        }
    }

    #[test]
    fn entity_dates() {
        let t1 = "t1".to_string();
        let t2 = "t2".to_string();

        fn assert_dates(
            agg: &EntityAggregator,
            creation: Option<OperationId>,
            modification: Option<OperationId>,
            deletion: Option<OperationId>,
        ) {
            assert_eq!(
                agg.creation_date,
                creation.map(|c| ConsistentTimestamp::from(c).to_datetime()),
            );
            assert_eq!(
                agg.modification_date,
                modification.map(|m| ConsistentTimestamp::from(m).to_datetime()),
            );
            assert_eq!(
                agg.deletion_date,
                deletion.map(|m| ConsistentTimestamp::from(m).to_datetime()),
            );
        }

        {
            // creation date based on operation
            let mutations = vec![mock_put_trait(&t1, TYPE1, Some(2), 1, None, None)];
            let em = EntityAggregator::new(mutations.into_iter()).unwrap();
            assert_dates(&em, Some(1), None, None);
        }

        {
            // explicit dates
            let mutations = vec![mock_put_trait(&t1, TYPE1, Some(2), 10, Some(1), Some(2))];
            let em = EntityAggregator::new(mutations.into_iter()).unwrap();
            assert_dates(&em, Some(1), Some(2), None);
        }

        {
            // multiple mutations operations based dates
            let mutations = vec![
                mock_put_trait(&t1, TYPE1, Some(2), 10, None, None),
                mock_put_trait(&t1, TYPE1, Some(3), 11, None, None),
            ];
            let em = EntityAggregator::new(mutations.into_iter()).unwrap();
            assert_dates(&em, Some(10), Some(11), None);
        }

        {
            // trait deletion counts as modification
            let mutations = vec![
                mock_put_trait(&t1, TYPE1, Some(2), 10, None, None),
                mock_put_trait(&t2, TYPE1, Some(2), 10, None, None),
                mock_delete_trait(&t1, Some(3), 11),
            ];
            let em = EntityAggregator::new(mutations.into_iter()).unwrap();
            assert_dates(&em, Some(10), Some(11), None);
        }

        {
            // deleting all traits should mark entity as deleted
            let mutations = vec![
                mock_put_trait(&t1, TYPE1, Some(2), 10, None, None),
                mock_delete_trait(&t1, Some(3), 11),
            ];
            let em = EntityAggregator::new(mutations.into_iter()).unwrap();
            assert_dates(&em, None, None, Some(11));
        }
        {
            // deleting entity should mark entity as deleted
            let mutations = vec![
                mock_put_trait(&t1, TYPE1, Some(2), 10, None, None),
                mock_delete_entity(Some(2), 11),
            ];
            let em = EntityAggregator::new(mutations.into_iter()).unwrap();
            assert_dates(&em, None, None, Some(11));
        }

        {
            // entity deletion resets dates
            let mutations = vec![
                mock_put_trait(&t1, TYPE1, Some(2), 1, Some(10), Some(11)),
                mock_delete_entity(Some(2), 2),
                mock_put_trait(&t1, TYPE1, Some(3), 3, Some(20), Some(21)),
            ];
            let em = EntityAggregator::new(mutations.into_iter()).unwrap();
            assert_dates(&em, Some(20), Some(21), None);
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

        fn assert_projection_id(em: &EntityAggregator, trait_id: &str, id: u32) {
            let trt = em.traits.get(trait_id);
            let proj = trt.unwrap().projection.as_ref().unwrap();
            assert_eq!(proj.field_ids[0], id,);
        }

        {
            // prefix match
            let mutations = vec![
                mock_put_trait(t1, TYPE1, Some(1), 1, None, None),
                mock_put_trait(t2, TYPE2, Some(1), 2, None, None),
            ];
            let mut em = EntityAggregator::new(mutations.into_iter()).unwrap();
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
            let mut em = EntityAggregator::new(mutations.into_iter()).unwrap();
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

        let project = |fields: Vec<FieldId>, groups: Vec<FieldGroupId>| {
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

            let msg = TestMessage::decode(trt.message.as_ref().unwrap().value.as_slice()).unwrap();

            (msg, TraitDetails::from_i32(trt.details).unwrap())
        };

        assert_eq!(project(vec![], vec![]), (msg.clone(), TraitDetails::Full));

        assert_eq!(
            project(vec![1], vec![]),
            (
                TestMessage {
                    string1: "string1".to_string(),
                    ..Default::default()
                },
                TraitDetails::Partial
            )
        );

        assert_eq!(
            project(vec![1, 2], vec![]),
            (
                TestMessage {
                    string1: "string1".to_string(),
                    string2: "string2".to_string(),
                    ..Default::default()
                },
                TraitDetails::Partial
            )
        );

        assert_eq!(
            project(vec![2], vec![1]),
            (
                TestMessage {
                    string2: "string2".to_string(),
                    grouped1: "grouped1".to_string(),
                    grouped2: "grouped2".to_string(),
                    ..Default::default()
                },
                TraitDetails::Partial
            )
        );

        assert_eq!(
            project(vec![], vec![2]),
            (
                TestMessage {
                    grouped2: "grouped2".to_string(),
                    ..Default::default()
                },
                TraitDetails::Partial
            )
        );

        Ok(())
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
