use std::sync::Arc;

use exocore_index::mutation::Mutation;
use exocore_index::store::remote::ClientHandle;
use exocore_schema::entity::{Entity, FieldValue, RecordBuilder, TraitBuilder};
use exocore_schema::schema::Schema;
use exocore_schema::serialization::with_schema;
use wasm_bindgen::__rt::std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::js::into_js_error;

#[wasm_bindgen]
pub struct MutationBuilder {
    schema: Arc<Schema>,
    store_handle: Arc<ClientHandle>,

    inner: Option<Mutation>,
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
        let trait_builder = self.jsdata_to_trait_builder(trait_type, data);
        let trt = trait_builder.build().expect("Couldn't build trait");
        self.inner = Some(Mutation::put_trait(entity_id, trt));
        self
    }

    #[wasm_bindgen]
    pub fn delete_trait(mut self, entity_id: String, trait_id: String) -> MutationBuilder {
        self.inner = Some(Mutation::delete_trait(entity_id, trait_id));
        self
    }

    #[wasm_bindgen]
    pub fn create_entity(self, trait_type: &str, data: JsValue) -> MutationBuilder {
        let entity_id = Entity::generate_random_id();
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
                    let serialized = with_schema(&schema, || JsValue::from_serde(&res));
                    serialized.map_err(into_js_error)
                }
                Err(err) => Err(into_js_error(err)),
            }
        })
    }

    fn jsdata_to_trait_builder(&self, trait_type: &str, data: JsValue) -> TraitBuilder {
        let dict: HashMap<String, FieldValue> = data.into_serde().expect("Couldn't parse data");

        let mut trait_builder = TraitBuilder::new_full_name(&self.schema, trait_type)
            .expect("Couldn't create TraitBuilder");
        for (name, value) in dict {
            trait_builder = trait_builder.set(&name, value);
        }

        trait_builder
    }
}
