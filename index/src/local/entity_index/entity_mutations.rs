use super::super::mutation_index::{MutationMetadata, MutationMetadataType, PutTraitMetadata};
use super::result_hasher;
use crate::entity::TraitId;
use crate::error::Error;
use crate::query::ResultHash;
use exocore_chain::operation::OperationId;
use exocore_core::time::ConsistentTimestamp;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

/// Traits metadata of an entity as retrieved from the traits index, as opposed
/// as being complete from the chain layer.
pub struct EntityMutations {
    // final traits of the entity once all mutations were aggregated
    pub traits: HashMap<TraitId, MutationMetadata>,

    // ids of operations that are still active (ex: were not overridden by another mutation)
    pub active_operations_id: HashSet<OperationId>,

    // hash of the operations of the entity
    pub hash: ResultHash,
}

impl EntityMutations {
    pub fn new<I>(mutations_metadata: I) -> Result<EntityMutations, Error>
    where
        I: Iterator<Item = MutationMetadata>,
    {
        let ordered_mutations_metadata =
            mutations_metadata.sorted_by_key(|result| result.operation_id);

        let mut hasher = result_hasher();
        // only keep last operation for each trait, and remove trait if it's a tombstone
        // we keep last operations id that have affected current traits / entities
        let mut traits = HashMap::<TraitId, MutationMetadata>::new();
        let mut active_operations_id = HashSet::<OperationId>::new();
        for mut trait_metadata in ordered_mutations_metadata {
            // hashing operations instead of traits content allow invalidating results as
            // soon as one operation is made since we can't guarantee anything
            hasher.write_u64(trait_metadata.operation_id);

            match &mut trait_metadata.mutation_type {
                MutationMetadataType::TraitPut(put_trait) => {
                    let opt_prev_trait = traits.get(&put_trait.trait_id);
                    if let Some(prev_trait) = opt_prev_trait {
                        active_operations_id.remove(&prev_trait.operation_id);
                    }

                    Self::merge_trait_metadata_dates(
                        trait_metadata.operation_id,
                        put_trait,
                        opt_prev_trait,
                    );

                    active_operations_id.insert(trait_metadata.operation_id);
                    traits.insert(put_trait.trait_id.clone(), trait_metadata);
                }
                MutationMetadataType::TraitTombstone(trait_id) => {
                    if let Some(prev_trait) = traits.get(trait_id) {
                        active_operations_id.remove(&prev_trait.operation_id);
                    }
                    active_operations_id.insert(trait_metadata.operation_id);
                    traits.remove(trait_id);
                }
                MutationMetadataType::EntityTombstone => {
                    active_operations_id.clear();
                    active_operations_id.insert(trait_metadata.operation_id);
                    traits.clear();
                }
            }
        }

        Ok(EntityMutations {
            traits,
            active_operations_id,
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
            if let MutationMetadataType::TraitPut(prev_trait) = &prev_trait.mutation_type {
                if modification_date.is_none() {
                    modification_date = Some(op_time);
                }

                if prev_trait.creation_date.is_some() && creation_date > prev_trait.creation_date {
                    creation_date = prev_trait.creation_date
                }

                if prev_trait.modification_date.is_some()
                    && modification_date < prev_trait.modification_date
                {
                    modification_date = prev_trait.modification_date;
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
