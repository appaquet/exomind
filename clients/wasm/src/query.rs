use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use futures::Future;
use wasm_bindgen::prelude::*;

use exocore_index::query::Query;
use exocore_index::store::remote::ClientHandle;
use exocore_index::store::AsyncStore;
use exocore_schema::schema::Schema;
use exocore_schema::serialization::with_schema;

use crate::js::into_js_error;

#[wasm_bindgen]
pub struct QueryBuilder {
    schema: Arc<Schema>,
    store_handle: Arc<ClientHandle>,
    inner: Option<Query>,
}

#[wasm_bindgen]
impl QueryBuilder {
    pub(crate) fn new(schema: Arc<Schema>, store_handle: Arc<ClientHandle>) -> QueryBuilder {
        QueryBuilder {
            schema,
            store_handle,
            inner: None,
        }
    }

    #[wasm_bindgen]
    pub fn match_text(mut self, query: String) -> Self {
        self.inner = Some(Query::match_text(query));
        self
    }

    #[wasm_bindgen]
    pub fn with_trait(mut self, name: String) -> Self {
        self.inner = Some(Query::with_trait(name));
        self
    }

    #[wasm_bindgen]
    pub fn with_count(mut self, count: u32) -> Self {
        self.inner = Some(
            self.inner
                .expect("Query was not initialized")
                .with_count(count),
        );
        self
    }

    #[wasm_bindgen]
    pub fn execute(self) -> crate::query::QueryResult {
        let query = self.inner.expect("Query was not initialized");

        let results_cell = Rc::new(RefCell::new(None));
        let fut_results = {
            let results_cell1 = results_cell.clone();
            let results_cell2 = results_cell.clone();
            self.store_handle
                .query(query)
                .map(move |result| {
                    let mut res_cell = results_cell1.borrow_mut();
                    *res_cell = Some(Ok(result));
                    true.into()
                })
                .map_err(move |err| {
                    let mut res_cell = results_cell2.borrow_mut();
                    *res_cell = Some(Err(err.clone()));
                    into_js_error(err)
                })
        };

        crate::query::QueryResult {
            schema: self.schema.clone(),
            _store_handle: self.store_handle.clone(),

            promise: wasm_bindgen_futures::future_to_promise(fut_results),
            inner: results_cell,
        }
    }
}

#[wasm_bindgen]
pub struct QueryResult {
    schema: Arc<Schema>,
    _store_handle: Arc<ClientHandle>,

    promise: js_sys::Promise,
    inner:
        Rc<RefCell<Option<Result<exocore_index::query::QueryResult, exocore_index::error::Error>>>>,
}

#[wasm_bindgen]
impl QueryResult {
    #[wasm_bindgen]
    pub fn ready(&self) -> js_sys::Promise {
        self.promise.clone()
    }

    #[wasm_bindgen]
    pub fn is_ready(&self) -> bool {
        let res = self.inner.borrow();
        res.is_some()
    }

    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        let res = self.inner.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        res.results.len()
    }

    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        let res = self.inner.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        res.results.is_empty()
    }

    #[wasm_bindgen]
    pub fn get(&self, index: usize) -> JsValue {
        let res = self.inner.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        with_schema(&self.schema, || JsValue::from_serde(&res.results[index]))
            .unwrap_or_else(into_js_error)
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> JsValue {
        let res = self.inner.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        with_schema(&self.schema, || JsValue::from_serde(res)).unwrap_or_else(into_js_error)
    }
}
