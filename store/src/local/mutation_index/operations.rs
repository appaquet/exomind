use exocore_chain::{
    block::BlockOffset,
    engine::EngineOperation,
    operation::{Operation, OperationId},
};
use exocore_protos::{
    generated::exocore_store::{entity_mutation::Mutation, EntityMutation, Trait},
    prost::Message,
};
use smallvec::SmallVec;

use crate::entity::{EntityId, TraitId};

/// Operation to be executed on the entities mutation index.
pub enum IndexOperation {
    /// Mutation that puts a new version of a trait at a new position in chain
    /// or pending
    PutTrait(PutTraitMutation),

    /// Mutation that marks a trait has being deleted without deleting it.
    PutTraitTombstone(PutTraitTombstoneMutation),

    /// Mutation that marks an entity has being deleted without deleting it.
    PutEntityTombstone(PutEntityTombstoneMutation),

    /// Delete an indexed mutation for specified entity id by its operation id.
    /// Independent from `DeleteOperation` so that we can indicate flush entity
    /// cache.
    DeleteEntityOperation(EntityId, OperationId),

    /// Indicates that the entity has a pending deletion. This operation is only
    /// stored in the pending index and is used to indicate that an entity
    /// cannot be garbage collected furthermore until the actual deletions
    /// are done in the chain index.
    PendingDeletionMarker(EntityId, OperationId),
}

pub struct PutTraitMutation {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
    pub trt: Trait,
}

pub struct PutTraitTombstoneMutation {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
    pub trait_id: TraitId,
}

pub struct PutEntityTombstoneMutation {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
}

impl IndexOperation {
    /// Creates an index operation from an engine operation stored in the
    /// pending store of the chain layer.
    pub fn from_pending_engine_operation(
        operation: EngineOperation,
    ) -> SmallVec<[IndexOperation; 1]> {
        let Some(entity_mutation) = Self::extract_entity_mutation(&operation) else {
            return smallvec![];
        };

        let Some(mutation) = entity_mutation.mutation else {
            return smallvec![];
        };

        match mutation {
            Mutation::PutTrait(trt_mut) => {
                let Some(trt) = trt_mut.r#trait else {
                    return smallvec![];
                };

                smallvec![IndexOperation::PutTrait(PutTraitMutation {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trt,
                })]
            }
            Mutation::DeleteTrait(trt_del) => smallvec![IndexOperation::PutTraitTombstone(
                PutTraitTombstoneMutation {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trait_id: trt_del.trait_id,
                }
            )],
            Mutation::DeleteEntity(_) => smallvec![IndexOperation::PutEntityTombstone(
                PutEntityTombstoneMutation {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                }
            )],
            Mutation::DeleteOperations(_) => {
                smallvec![IndexOperation::PendingDeletionMarker(
                    entity_mutation.entity_id,
                    operation.operation_id
                )]
            }
            Mutation::Test(_mutation) => smallvec![],
        }
    }

    /// Creates an index operation from an engine operation store in the chain
    /// of the chain layer.
    pub fn from_chain_engine_operation(
        operation: EngineOperation,
        block_offset: BlockOffset,
    ) -> (SmallVec<[IndexOperation; 1]>, EntityId) {
        let Some(entity_mutation) = Self::extract_entity_mutation(&operation) else {
            return (smallvec![], String::new());
        };

        let entity_id = entity_mutation.entity_id.clone();

        (
            Self::from_chain_entity_mutation(entity_mutation, operation.operation_id, block_offset),
            entity_id,
        )
    }

    /// Creates an index operation from an entity mutation that will target the
    /// chain index.
    pub fn from_chain_entity_mutation(
        entity_mutation: EntityMutation,
        operation_id: OperationId,
        block_offset: BlockOffset,
    ) -> SmallVec<[IndexOperation; 1]> {
        let Some(mutation) = entity_mutation.mutation else {
            return smallvec![];
        };

        match mutation {
            Mutation::PutTrait(trait_put) => {
                let Some(trt) = trait_put.r#trait else {
                    return smallvec![];
                };

                smallvec![IndexOperation::PutTrait(PutTraitMutation {
                    block_offset: Some(block_offset),
                    operation_id,
                    entity_id: entity_mutation.entity_id,
                    trt,
                })]
            }
            Mutation::DeleteTrait(trt_del) => smallvec![IndexOperation::PutTraitTombstone(
                PutTraitTombstoneMutation {
                    block_offset: Some(block_offset),
                    operation_id,
                    entity_id: entity_mutation.entity_id,
                    trait_id: trt_del.trait_id,
                }
            )],
            Mutation::DeleteEntity(_) => smallvec![IndexOperation::PutEntityTombstone(
                PutEntityTombstoneMutation {
                    block_offset: Some(block_offset),
                    operation_id,
                    entity_id: entity_mutation.entity_id,
                }
            )],
            Mutation::DeleteOperations(del_mut) => {
                let mut index_mutations = SmallVec::with_capacity(del_mut.operation_ids.len());
                for op_id in del_mut.operation_ids {
                    index_mutations.push(IndexOperation::DeleteEntityOperation(
                        entity_mutation.entity_id.clone(),
                        op_id,
                    ));
                }
                index_mutations
            }
            Mutation::Test(_) => smallvec![],
        }
    }

    /// Extracts an EntityMutation out of an EngineOperation if it contains one.
    pub fn extract_entity_mutation(operation: &EngineOperation) -> Option<EntityMutation> {
        let entry_data = match operation.as_entry_data() {
            Ok(data) => data,
            Err(err) => {
                trace!(
                    "Operation (id={} status={:?}) didn't have any data to index: {}",
                    operation.operation_id,
                    operation.status,
                    err,
                );
                return None;
            }
        };

        match EntityMutation::decode(entry_data) {
            Ok(mutation) => Some(mutation),
            Err(err) => {
                error!(
                    "Operation (id={} status={:?}) entity mutation couldn't be decoded: {}",
                    operation.operation_id, operation.status, err
                );
                None
            }
        }
    }
}
