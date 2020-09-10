use exocore_core::protos::{
    index::{Entity, Trait},
    NamedMessage,
};
use prost::Message;

pub type EntityId = String;
pub type TraitId = String;

pub struct TraitInstance<'e, M: NamedMessage + Message + Default> {
    pub instance: M,
    pub trt: &'e Trait,
}

pub trait EntityExt {
    fn trait_with_id<M: NamedMessage + Message + Default>(
        &self,
        id: &str,
    ) -> Option<TraitInstance<M>>;

    fn traits_of_type<M: NamedMessage + Message + Default>(&self) -> Vec<TraitInstance<M>>;

    fn trait_of_type<M: NamedMessage + Message + Default>(&self) -> Option<TraitInstance<M>>;
}

impl EntityExt for Entity {
    fn trait_with_id<M: NamedMessage + Message + Default>(
        &self,
        id: &str,
    ) -> Option<TraitInstance<M>> {
        let msg_any_url = M::protobuf_any_url();

        self.traits
            .iter()
            .filter(|t| t.id == id)
            .flat_map(|t| {
                let msg = t.message.as_ref()?;
                if msg.type_url == msg_any_url {
                    let instance = M::decode(msg.value.as_slice()).ok()?;
                    Some(TraitInstance { instance, trt: t })
                } else {
                    None
                }
            })
            .next()
    }

    fn traits_of_type<M: NamedMessage + Message + Default>(&self) -> Vec<TraitInstance<M>> {
        let msg_any_url = M::protobuf_any_url();

        self.traits
            .iter()
            .flat_map(|t| {
                let msg = t.message.as_ref()?;
                if msg.type_url == msg_any_url {
                    let instance = M::decode(msg.value.as_slice()).ok()?;
                    Some(TraitInstance { instance, trt: t })
                } else {
                    None
                }
            })
            .collect()
    }

    fn trait_of_type<M: NamedMessage + Message + Default>(&self) -> Option<TraitInstance<M>> {
        let msg_any_url = M::protobuf_any_url();

        self.traits
            .iter()
            .flat_map(|t| {
                let msg = t.message.as_ref()?;
                if msg.type_url == msg_any_url {
                    let instance = M::decode(msg.value.as_slice()).ok()?;
                    Some(TraitInstance { instance, trt: t })
                } else {
                    None
                }
            })
            .next()
    }
}
