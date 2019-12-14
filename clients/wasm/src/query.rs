#![allow(clippy::unnecessary_mut_passed)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use wasm_bindgen::prelude::*;

use exocore_index::query::Query;
use exocore_index::store::remote::ClientHandle;
use exocore_schema::schema::Schema;
use exocore_schema::serialization::with_schema;

use crate::js::into_js_error;
use exocore_common::utils::futures::spawn_future_non_send;
use futures::channel::oneshot;
use futures::prelude::*;

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

        let store_handle = self.store_handle;
        let result_cell = Rc::new(RefCell::new(None));
        let result_cell1 = result_cell.clone();
        let fut_results = async move {
            let result = store_handle.query(query).await;
            let js_result = match &result {
                Ok(_res) => Ok(true.into()),
                Err(err) => Err(into_js_error(err)),
            };

            let mut res_cell = result_cell1.borrow_mut();
            *res_cell = Some(result);

            js_result
        };

        crate::query::QueryResult {
            schema: self.schema.clone(),
            promise: wasm_bindgen_futures::future_to_promise(fut_results),
            result_cell,
        }
    }

    #[wasm_bindgen]
    pub fn execute_and_watch(self) -> crate::query::WatchedQuery {
        let query = self.inner.expect("Query was not initialized");

        let result_cell = Rc::new(RefCell::new(None));
        let callback_cell = Rc::new(RefCell::<Option<js_sys::Function>>::new(None));

        let results_cell1 = result_cell.clone();
        let callback_cell1 = callback_cell.clone();
        let report_result = move |result: Result<
            exocore_index::query::QueryResult,
            exocore_index::error::Error,
        >| {
            if let Err(err) = &result {
                error!("Error in watched query: {}", err);
            }

            {
                let mut res_cell = results_cell1.borrow_mut();
                *res_cell = Some(result);
            }

            {
                let callback = callback_cell1.borrow();
                if let Some(func) = &*callback {
                    func.call0(&JsValue::null()).unwrap();
                }
            }
        };

        let (drop_sender, drop_receiver) = oneshot::channel();
        let store_handle = self.store_handle;
        spawn_future_non_send(async move {
            let mut results = store_handle.watched_query(query);
            let mut drop_receiver = drop_receiver.fuse();

            loop {
                futures::select! {
                    result = results.next().fuse() => {
                        let result = if let Some(result) = result {
                            result
                        } else {
                            return Ok(());
                        };

                        report_result(result);
                    }
                    _ = drop_receiver => {
                        return Ok(());
                    }
                };
            }
        });

        crate::query::WatchedQuery {
            schema: self.schema.clone(),
            result_cell,
            callback_cell,
            _drop_sender: drop_sender,
        }
    }
}

type ResultCell =
    Rc<RefCell<Option<Result<exocore_index::query::QueryResult, exocore_index::error::Error>>>>;
type CallbackCell = Rc<RefCell<Option<js_sys::Function>>>;

#[wasm_bindgen]
pub struct QueryResult {
    schema: Arc<Schema>,
    promise: js_sys::Promise,
    result_cell: ResultCell,
}

#[wasm_bindgen]
impl QueryResult {
    #[wasm_bindgen]
    pub fn ready(&self) -> js_sys::Promise {
        self.promise.clone()
    }

    #[wasm_bindgen]
    pub fn is_ready(&self) -> bool {
        let res = self.result_cell.borrow();
        res.is_some()
    }

    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        let res = self.result_cell.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        res.results.len()
    }

    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        let res = self.result_cell.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        res.results.is_empty()
    }

    #[wasm_bindgen]
    pub fn get(&self, index: usize) -> JsValue {
        let res = self.result_cell.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        with_schema(&self.schema, || JsValue::from_serde(&res.results[index]))
            .unwrap_or_else(into_js_error)
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> JsValue {
        let res = self.result_cell.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        with_schema(&self.schema, || JsValue::from_serde(res)).unwrap_or_else(into_js_error)
    }
}

#[wasm_bindgen]
pub struct WatchedQuery {
    schema: Arc<Schema>,
    result_cell: ResultCell,
    callback_cell: CallbackCell,
    _drop_sender: oneshot::Sender<()>,
}

#[wasm_bindgen]
impl WatchedQuery {
    #[wasm_bindgen]
    pub fn on_change(&self, promise: js_sys::Function) {
        let mut cb = self.callback_cell.borrow_mut();
        *cb = Some(promise);
    }

    #[wasm_bindgen]
    pub fn get(&self, index: usize) -> JsValue {
        let res = self.result_cell.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        with_schema(&self.schema, || JsValue::from_serde(&res.results[index]))
            .unwrap_or_else(into_js_error)
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> JsValue {
        let res = self.result_cell.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();

        with_schema(&self.schema, || JsValue::from_serde(res)).unwrap_or_else(into_js_error)
    }
}
