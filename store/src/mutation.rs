use prost::Message;

use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_core::protos::generated::exocore_store::entity_mutation::Mutation;
use exocore_core::protos::generated::exocore_store::{
    compact_trait_mutation, CompactTraitMutation, DeleteEntityMutation, DeleteTraitMutation,
    EntityMutation, MutationRequest, MutationResult, PutTraitMutation, Trait,
};
use exocore_core::protos::generated::store_transport_capnp::{mutation_request, mutation_response};
use exocore_core::protos::prost::ProstMessageExt;

use crate::entity::{EntityId, TraitId};
use crate::error::Error;
use exocore_chain::operation::OperationId;

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
    pub(crate) fn compact_traits<E: Into<TraitId>>(
        mut self,
        entity_id: E,
        trt: Trait,
        compacted_operations: Vec<OperationId>,
    ) -> MutationBuilder {
        let operations = compacted_operations
            .iter()
            .map(|id| compact_trait_mutation::Operation { operation_id: *id })
            .collect();

        self.request.mutations.push(EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::CompactTrait(CompactTraitMutation {
                r#trait: Some(trt),
                compacted_operations: operations,
            })),
        });

        self
    }

    #[cfg(test)]
    pub(crate) fn fail_mutation<E: Into<EntityId>>(mut self, entity_id: E) -> MutationBuilder {
        self.request.mutations.push(EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::Test(
                exocore_core::protos::generated::exocore_store::TestMutation { success: false },
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

pub fn mutation_to_request_frame(
    request: MutationRequest,
) -> Result<CapnpFrameBuilder<mutation_request::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<mutation_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    let buf = request.encode_to_vec()?;
    msg_builder.set_request(&buf);

    Ok(frame_builder)
}

pub fn mutation_from_request_frame<I>(
    frame: TypedCapnpFrame<I, mutation_request::Owned>,
) -> Result<MutationRequest, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    let data = reader.get_request()?;
    Ok(MutationRequest::decode(data)?)
}

pub fn mutation_result_to_response_frame(
    result: Result<MutationResult, Error>,
) -> Result<CapnpFrameBuilder<mutation_response::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<mutation_response::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    match result {
        Ok(res) => {
            let buf = res.encode_to_vec()?;
            msg_builder.set_response(&buf);
        }
        Err(err) => {
            msg_builder.set_error(&err.to_string());
        }
    }

    Ok(frame_builder)
}

pub fn mutation_result_from_response_frame<I>(
    frame: TypedCapnpFrame<I, mutation_response::Owned>,
) -> Result<MutationResult, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    if reader.has_error() {
        Err(Error::Remote(reader.get_error()?.to_owned()))
    } else {
        let data = reader.get_response()?;
        Ok(MutationResult::decode(data)?)
    }
}
