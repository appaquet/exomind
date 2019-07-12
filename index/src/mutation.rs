use crate::domain::entity::{EntityId, Trait, TraitId};
use crate::domain::schema::Schema;
use crate::error::Error;
use std::sync::Arc;

#[serde(rename_all = "snake_case", tag = "type")]
#[derive(Serialize, Deserialize, Debug)]
pub enum Mutation {
    PutTrait(PutTraitMutation),
    DeleteTrait(DeleteTraitMutation),
}

impl Mutation {
    pub fn from_json_slice(schema: Arc<Schema>, json_bytes: &[u8]) -> Result<Mutation, Error> {
        crate::domain::serialization::with_schema(&schema, || {
            serde_json::from_slice(json_bytes).map_err(|err| err.into())
        })
    }

    pub fn to_json(&self, schema: Arc<Schema>) -> Result<String, Error> {
        crate::domain::serialization::with_schema(&schema, || {
            serde_json::to_string(self).map_err(|err| err.into())
        })
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
