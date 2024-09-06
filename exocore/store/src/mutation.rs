use exocore_protos::{
    generated::exocore_store::{
        entity_mutation::Mutation, DeleteEntityMutation, DeleteTraitMutation, EntityMutation,
        MutationRequest, PutTraitMutation, Trait,
    },
    store::DeleteOperationsMutation,
};

use crate::entity::{EntityId, TraitId};

pub type OperationId = u64;

pub struct MutationBuilder {
    request: MutationRequest,
}

impl MutationBuilder {
    pub fn new() -> MutationBuilder {
        MutationBuilder {
            request: MutationRequest {
                mutations: vec![],
                wait_indexed: false,
                return_entities: false,
                common_entity_id: false,
            },
        }
    }

    pub fn put_trait<E: Into<EntityId>>(mut self, entity_id: E, trt: Trait) -> MutationBuilder {
        self.request.mutations.push(EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::PutTrait(PutTraitMutation { r#trait: Some(trt) })),
        });

        self
    }

    pub fn delete_trait<E: Into<EntityId>, T: Into<TraitId>>(
        mut self,
        entity_id: E,
        trait_id: T,
    ) -> MutationBuilder {
        self.request.mutations.push(EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::DeleteTrait(DeleteTraitMutation {
                trait_id: trait_id.into(),
            })),
        });

        self
    }

    pub fn delete_entity<E: Into<EntityId>>(mut self, entity_id: E) -> MutationBuilder {
        self.request.mutations.push(EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::DeleteEntity(DeleteEntityMutation {})),
        });

        self
    }

    pub fn use_common_entity_id(mut self) -> MutationBuilder {
        self.request.common_entity_id = true;

        self
    }

    pub fn return_entities(mut self) -> MutationBuilder {
        self.request.return_entities = true;
        self
    }

    #[allow(unused)]
    pub(crate) fn delete_operations<E: Into<EntityId>>(
        mut self,
        entity_id: E,
        operation_ids: Vec<OperationId>,
    ) -> MutationBuilder {
        self.request.mutations.push(EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::DeleteOperations(DeleteOperationsMutation {
                operation_ids,
            })),
        });

        self
    }

    #[cfg(test)]
    pub(crate) fn fail_mutation<E: Into<EntityId>>(mut self, entity_id: E) -> MutationBuilder {
        self.request.mutations.push(EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::Test(
                exocore_protos::generated::exocore_store::TestMutation { success: false },
            )),
        });

        self
    }

    pub fn build(self) -> MutationRequest {
        self.request
    }
}

impl Default for MutationBuilder {
    fn default() -> Self {
        MutationBuilder::new()
    }
}

pub struct MutationRequestLike(pub MutationRequest);

impl From<MutationRequest> for MutationRequestLike {
    fn from(req: MutationRequest) -> Self {
        MutationRequestLike(req)
    }
}

impl From<EntityMutation> for MutationRequestLike {
    fn from(mutation: EntityMutation) -> Self {
        MutationRequestLike(MutationRequest {
            mutations: vec![mutation],
            ..Default::default()
        })
    }
}

impl From<MutationBuilder> for MutationRequestLike {
    fn from(builder: MutationBuilder) -> Self {
        MutationRequestLike(builder.build())
    }
}

impl std::ops::Deref for MutationRequestLike {
    type Target = MutationRequest;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
