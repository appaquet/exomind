use crate::entity::{EntityId, TraitId};
use exocore_chain::block::BlockOffset;
use exocore_chain::engine::EngineOperation;
use exocore_chain::operation::{Operation, OperationId};
use exocore_core::protos::generated::exocore_index::entity_mutation::Mutation;
use exocore_core::protos::generated::exocore_index::{EntityMutation, Trait};
use prost::Message;
use smallvec::SmallVec;

/// Mutation of the index.
pub enum IndexMutation {
    /// New version of a trait at a new position in chain or pending
    PutTrait(PutTraitMutation),

    /// Mark a trait has being deleted without deleting it.
    PutTraitTombstone(PutTraitTombstone),

    /// Mark an entity has being deleted without deleting it.
    PutEntityTombstone(PutEntityTombstone),

    /// Delete a document by its operation id.
    DeleteOperation(OperationId),
}

pub struct PutTraitMutation {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
    pub trt: Trait,
}

pub struct PutTraitTombstone {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
    pub trait_id: TraitId,
}

pub struct PutEntityTombstone {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
}

impl IndexMutation {
    /// Creates an index mutation from an engine operation stored in the pending store of the chain layer
    pub fn from_pending_engine_operation(
        operation: EngineOperation,
    ) -> SmallVec<[IndexMutation; 1]> {
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

                smallvec![IndexMutation::PutTrait(PutTraitMutation {
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

                smallvec![IndexMutation::PutTrait(PutTraitMutation {
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
            Mutation::DeleteTrait(trt_del) => {
                smallvec![IndexMutation::PutTraitTombstone(PutTraitTombstone {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trait_id: trt_del.trait_id,
                })]
            }
            Mutation::DeleteEntity(_) => {
                smallvec![IndexMutation::PutEntityTombstone(PutEntityTombstone {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                })]
            }
            Mutation::Test(_mutation) => smallvec![],
        }
    }

    /// Creates an index mutation from an engine operation store in the chain of the chain layer
    pub fn from_chain_engine_operation(
        operation: EngineOperation,
        block_offset: BlockOffset,
    ) -> SmallVec<[IndexMutation; 1]> {
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

                smallvec![IndexMutation::PutTrait(PutTraitMutation {
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
                    index_mutations.push(IndexMutation::DeleteOperation(op.operation_id));
                }

                index_mutations.push(IndexMutation::PutTrait(PutTraitMutation {
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
            Mutation::DeleteTrait(trt_del) => {
                smallvec![IndexMutation::PutTraitTombstone(PutTraitTombstone {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                    trait_id: trt_del.trait_id,
                })]
            }
            Mutation::DeleteEntity(_) => {
                smallvec![IndexMutation::PutEntityTombstone(PutEntityTombstone {
                    block_offset: None,
                    operation_id: operation.operation_id,
                    entity_id: entity_mutation.entity_id,
                })]
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
