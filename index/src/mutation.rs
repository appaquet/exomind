use prost::Message;

use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_core::protos::generated::exocore_index::entity_mutation::Mutation;
use exocore_core::protos::generated::exocore_index::{
    compact_trait_mutation, CompactTraitMutation, DeleteEntityMutation, DeleteTraitMutation,
    EntityMutation, MutationResult, PutTraitMutation, Trait,
};
use exocore_core::protos::generated::index_transport_capnp::{mutation_request, mutation_response};
use exocore_core::protos::prost::ProstMessageExt;

use crate::entity::{EntityId, TraitId};
use crate::error::Error;
use exocore_chain::operation::OperationId;

pub struct MutationBuilder;

impl MutationBuilder {
    pub fn put_trait<E: Into<EntityId>>(entity_id: E, trt: Trait) -> EntityMutation {
        EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::PutTrait(PutTraitMutation { r#trait: Some(trt) })),
        }
    }

    pub fn delete_trait<E: Into<EntityId>, T: Into<TraitId>>(
        entity_id: E,
        trait_id: T,
    ) -> EntityMutation {
        EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::DeleteTrait(DeleteTraitMutation {
                trait_id: trait_id.into(),
            })),
        }
    }

    pub fn delete_entity<E: Into<EntityId>>(entity_id: E) -> EntityMutation {
        EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::DeleteEntity(DeleteEntityMutation {})),
        }
    }

    #[allow(unused)]
    pub(crate) fn compact_traits<E: Into<TraitId>>(
        entity_id: E,
        trt: Trait,
        compacted_operations: Vec<OperationId>,
    ) -> EntityMutation {
        let operations = compacted_operations
            .iter()
            .map(|id| compact_trait_mutation::Operation { operation_id: *id })
            .collect();

        EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::CompactTrait(CompactTraitMutation {
                r#trait: Some(trt),
                compacted_operations: operations,
            })),
        }
    }

    #[cfg(test)]
    pub(crate) fn fail_mutation<E: Into<EntityId>>(entity_id: E) -> EntityMutation {
        EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::Test(
                exocore_core::protos::generated::exocore_index::TestMutation { success: false },
            )),
        }
    }
}

pub fn mutation_to_request_frame(
    entity_mutation: EntityMutation,
) -> Result<CapnpFrameBuilder<mutation_request::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<mutation_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    let buf = entity_mutation.encode_to_vec()?;
    msg_builder.set_request(&buf);

    Ok(frame_builder)
}

pub fn mutation_from_request_frame<I>(
    frame: TypedCapnpFrame<I, mutation_request::Owned>,
) -> Result<EntityMutation, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    let data = reader.get_request()?;

    let entity_mutation = EntityMutation::decode(data)?;

    Ok(entity_mutation)
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
        let mutation_result = MutationResult::decode(data)?;

        Ok(mutation_result)
    }
}
