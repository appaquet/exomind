use std::sync::Arc;

use exocore_index::mutation::MutationBuilder as InnerMutationBuilder;
use exocore_index::store::remote::ClientHandle;
use exocore_schema::entity::{Entity, FieldValue, RecordBuilder, TraitBuilder};
use exocore_schema::schema::Schema;
use exocore_schema::serialization::with_schema;
use js_sys::Uint8Array;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::js::into_js_error;
use exocore_common::protos::generated::exocore_index::EntityMutation;

#[wasm_bindgen]
pub struct MutationBuilder {
    schema: Arc<Schema>,
    store_handle: Arc<ClientHandle>,

    inner: Option<EntityMutation>,
}

#[wasm_bindgen]
impl MutationBuilder {
    pub(crate) fn new(schema: Arc<Schema>, store_handle: Arc<ClientHandle>) -> MutationBuilder {
        MutationBuilder {
            schema,
            store_handle,
            inner: None,
        }
    }

    #[wasm_bindgen]
    pub fn put_trait(
        mut self,
        entity_id: String,
        trait_type: &str,
        data: JsValue,
    ) -> MutationBuilder {
        // TODO: Parse
        //        let trait_builder = self.jsdata_to_trait_builder(trait_type, data);
        //        let trt = trait_builder.build().expect("Couldn't build trait");
        //        self.inner = Some(InnerMutationBuilder::put_trait(entity_id, trt));
        self
    }

    #[wasm_bindgen]
    pub fn delete_trait(mut self, entity_id: String, trait_id: String) -> MutationBuilder {
        self.inner = Some(InnerMutationBuilder::delete_trait(entity_id, trait_id));
        self
    }

    #[wasm_bindgen]
    pub fn create_entity(self, trait_type: &str, data: JsValue) -> MutationBuilder {
        let entity_id = Entity::generate_random_id(); // TODO: move to some method in core
        self.put_trait(entity_id, trait_type, data)
    }

    #[wasm_bindgen]
    pub fn execute(self) -> js_sys::Promise {
        let store_handle = self.store_handle;
        let mutation = self.inner.expect("Mutation was not initialized");
        let schema = self.schema;

        wasm_bindgen_futures::future_to_promise(async move {
            let result = store_handle.mutate(mutation).await;

            match result {
                Ok(res) => {
                    // TODO:
                    //                    let serialized = with_schema(&schema, || JsValue::from_serde(&res));
                    //                    serialized.map_err(into_js_error)
                    Ok("".into())
                }
                Err(err) => Err(into_js_error(err)),
            }
        })
    }
}
