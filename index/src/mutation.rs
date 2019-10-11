use crate::error::Error;
use exocore_common::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_common::protos::index_transport_capnp::{mutation_request, mutation_response};
use exocore_data::operation::OperationId;
use exocore_schema::entity::{EntityId, Trait, TraitId};
use exocore_schema::schema::Schema;
use exocore_schema::serialization::with_schema;
use std::sync::Arc;

#[serde(rename_all = "snake_case", tag = "type")]
#[derive(Serialize, Deserialize, Debug)]
pub enum Mutation {
    PutTrait(PutTraitMutation),
    DeleteTrait(DeleteTraitMutation),

    #[cfg(test)]
    TestFail(TestFailMutation),
}

impl Mutation {
    pub fn put_trait(entity_id: EntityId, trt: Trait) -> Mutation {
        Mutation::PutTrait(PutTraitMutation { entity_id, trt })
    }

    pub fn delete_trait(entity_id: EntityId, trait_id: TraitId) -> Mutation {
        Mutation::DeleteTrait(DeleteTraitMutation {
            entity_id,
            trait_id,
        })
    }

    pub fn from_json_slice(schema: Arc<Schema>, json_bytes: &[u8]) -> Result<Mutation, Error> {
        with_schema(&schema, || {
            serde_json::from_slice(json_bytes).map_err(|err| err.into())
        })
    }

    pub fn to_json(&self, schema: Arc<Schema>) -> Result<String, Error> {
        with_schema(&schema, || {
            serde_json::to_string(self).map_err(|err| err.into())
        })
    }

    pub fn to_mutation_request_frame(
        &self,
        schema: &Arc<Schema>,
    ) -> Result<CapnpFrameBuilder<mutation_request::Owned>, Error> {
        let mut frame_builder = CapnpFrameBuilder::<mutation_request::Owned>::new();
        let mut msg_builder = frame_builder.get_builder();
        let serialized_mutation = with_schema(schema, || serde_json::to_vec(&self))?;
        msg_builder.set_request(&serialized_mutation);

        Ok(frame_builder)
    }

    pub fn from_mutation_request_frame<I>(
        schema: &Arc<Schema>,
        frame: TypedCapnpFrame<I, mutation_request::Owned>,
    ) -> Result<Mutation, Error>
    where
        I: FrameReader,
    {
        let reader = frame.get_reader()?;
        let data = reader.get_request()?;
        let mutation = with_schema(schema, || serde_json::from_slice(data))?;

        Ok(mutation)
    }
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct PutTraitMutation {
    pub entity_id: EntityId,
    #[serde(rename = "trait")]
    pub trt: Trait,
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteTraitMutation {
    pub entity_id: EntityId,
    pub trait_id: TraitId,
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct TestFailMutation {}

///
/// Returned by store after executing mutation
///
#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct MutationResult {
    pub operation_id: OperationId,
}

impl MutationResult {
    pub fn result_to_response_frame(
        schema: &Arc<Schema>,
        result: Result<MutationResult, Error>,
    ) -> Result<CapnpFrameBuilder<mutation_response::Owned>, Error> {
        let mut frame_builder = CapnpFrameBuilder::<mutation_response::Owned>::new();
        let mut msg_builder = frame_builder.get_builder();

        match result {
            Ok(res) => {
                let serialized = with_schema(schema, || serde_json::to_vec(&res))?;
                msg_builder.set_response(&serialized);
            }
            Err(err) => {
                msg_builder.set_error(&err.to_string());
            }
        }

        Ok(frame_builder)
    }

    pub fn from_response_frame<I>(
        schema: &Arc<Schema>,
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
            let mutation_result = with_schema(schema, || serde_json::from_slice(data))?;
            Ok(mutation_result)
        }
    }
}
