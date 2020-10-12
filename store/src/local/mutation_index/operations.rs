use crate::entity::{EntityId, TraitId};
use exocore_chain::block::BlockOffset;
use exocore_chain::engine::EngineOperation;
use exocore_chain::operation::{Operation, OperationId};
use exocore_core::protos::generated::exocore_store::entity_mutation::Mutation;
use exocore_core::protos::generated::exocore_store::{EntityMutation, Trait};
use prost::Message;
use smallvec::SmallVec;

/// Operation to be executed on the entities mutations index.
pub enum IndexOperation {
    /// Mutation that puts a new version of a trait at a new position in chain
    /// or pending
    PutTrait(PutTraitMutation),

    /// Mutation that marks a trait has being deleted without deleting it.
    PutTraitTombstone(PutTraitTombstoneMutation),

    /// Mutation that marks an entity has being deleted without deleting it.
    PutEntityTombstone(PutEntityTombstoneMutation),

    /// Delete an indexed mutation by its operation id.
    DeleteOperation(OperationId),
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
        let entity_mutation = if let Some(mutation) = Self::extract_entity_mutation(&operation) {
            mutation
        } else {
            return smallvec![];
        };

        let mutation = if let Some(mutation) = entity_mutation.mutation {
            mutation
        } else {
            return smallvec![];
        };

        match mutation {
            Mutation::PutTrait(trt_mut) => {
                let trt = if let Some(trt) = trt_mut.r#trait {
                    trt
                } else {
                    return smallvec![];
                };

                smallvec![IndexOperation::PutTrait(PutTraitMutation {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trt,
                })]
            }
            Mutation::CompactTrait(cmpt_mut) => {
                let trt = if let Some(trt) = cmpt_mut.r#trait {
                    trt
                } else {
                    return smallvec![];
                };

                smallvec![IndexOperation::PutTrait(PutTraitMutation {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trt,
                })]
            }
            Mutation::UpdateTrait(_) => {
                // An update is handled at the store level where it will be succeeded by a put
                // created of the current value
                smallvec![]
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
            Mutation::Test(_mutation) => smallvec![],
        }
    }

    /// Creates an index operation from an engine operation store in the chain
    /// of the chain layer
    pub fn from_chain_engine_operation(
        operation: EngineOperation,
        block_offset: BlockOffset,
    ) -> SmallVec<[IndexOperation; 1]> {
        let entity_mutation = if let Some(mutation) = Self::extract_entity_mutation(&operation) {
            mutation
        } else {
            return smallvec![];
        };

        let mutation = if let Some(mutation) = entity_mutation.mutation {
            mutation
        } else {
            return smallvec![];
        };

        match mutation {
            Mutation::PutTrait(trait_put) => {
                let trt = if let Some(trt) = trait_put.r#trait {
                    trt
                } else {
                    return smallvec![];
                };

                smallvec![IndexOperation::PutTrait(PutTraitMutation {
                    block_offset: Some(block_offset),
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trt,
                })]
            }
            Mutation::CompactTrait(trait_compact) => {
                let trt = if let Some(trt) = trait_compact.r#trait {
                    trt
                } else {
                    return smallvec![];
                };

                let mut index_mutations = SmallVec::new();
                for op in trait_compact.compacted_operations {
                    index_mutations.push(IndexOperation::DeleteOperation(op.operation_id));
                }

                index_mutations.push(IndexOperation::PutTrait(PutTraitMutation {
                    block_offset: Some(block_offset),
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trt,
                }));

                index_mutations
            }
            Mutation::UpdateTrait(_trt_up) => {
                // An update is handled at the store level where it will be succeeded by a put
                // created of the current value
                smallvec![]
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
