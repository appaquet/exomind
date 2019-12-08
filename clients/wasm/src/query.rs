use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use futures::Future;
use wasm_bindgen::prelude::*;

use exocore_index::query::Query;
use exocore_index::store::remote::ClientHandle;
use exocore_schema::schema::Schema;
use exocore_schema::serialization::with_schema;
use futures::prelude::*;
use futures::unsync::oneshot;

use crate::js::into_js_error;
use exocore_common::utils::futures::spawn_future_non_send;

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

        let result_cell = Rc::new(RefCell::new(None));
        let fut_results = {
            let results_cell1 = result_cell.clone();
            let results_cell2 = result_cell.clone();
            self.store_handle
                .query(query)
                .expect("Couldn't query store")
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
        let (drop_sender, drop_receiver) = oneshot::channel();
        let stream = self
            .store_handle
            .watched_query(query)
            .expect("Couldn't watched query store")
            .then(move |result| {
                let ret = result.as_ref().map(|_| ()).map_err(|_| ());

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

                ret
            })
            .for_each(|_| Ok(()));

        spawn_future_non_send(stream.select(drop_receiver.map_err(|_| ())).then(|_| {
            info!("Watch query stream is done");
            Ok(())
        }));

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
