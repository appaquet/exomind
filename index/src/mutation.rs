use crate::error::Error;
use exocore_common::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_common::protos::generated::exocore_index::entity_mutation::Mutation;
use exocore_common::protos::generated::exocore_index::{
    DeleteTraitMutation, EntityMutation, MutationResult, PutTraitMutation, TestMutation, Trait,
};
use exocore_common::protos::index_transport_capnp::{mutation_request, mutation_response};
use exocore_common::protos::prost::ProstMessageExt;
use prost::Message;

pub struct MutationBuilder;

impl MutationBuilder {
    pub fn put_trait<E: Into<String>>(entity_id: E, trt: Trait) -> EntityMutation {
        EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::PutTrait(PutTraitMutation { r#trait: Some(trt) })),
        }
    }

    pub fn delete_trait<E: Into<String>, T: Into<String>>(
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

    pub fn fail_mutation<E: Into<String>>(entity_id: E) -> EntityMutation {
        EntityMutation {
            entity_id: entity_id.into(),
            mutation: Some(Mutation::Test(TestMutation { success: false })),
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
