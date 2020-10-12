#![allow(clippy::unnecessary_mut_passed)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use futures::channel::oneshot;
use futures::prelude::*;
use wasm_bindgen::prelude::*;

use exocore_core::futures::spawn_future_non_send;
use exocore_core::protos::generated::exocore_store::{EntityQuery, EntityResults};
use exocore_core::protos::prost::ProstMessageExt;
use exocore_store::remote::ClientHandle;

type ResultCell = Rc<RefCell<Option<Result<EntityResults, exocore_store::error::Error>>>>;
type CallbackCell = Rc<RefCell<Option<js_sys::Function>>>;

#[wasm_bindgen]
pub struct WatchedQuery {
    result_cell: ResultCell,
    callback_cell: CallbackCell,
    _drop_sender: oneshot::Sender<()>,
}

#[wasm_bindgen]
impl WatchedQuery {
    pub(crate) fn new(store_handle: Arc<ClientHandle>, query: EntityQuery) -> WatchedQuery {
        let result_cell = Rc::new(RefCell::new(None));
        let callback_cell = Rc::new(RefCell::<Option<js_sys::Function>>::new(None));

        let results_cell1 = result_cell.clone();
        let callback_cell1 = callback_cell.clone();
        let report_result = move |result: Result<EntityResults, exocore_store::error::Error>| {
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

        Self {
            result_cell,
            callback_cell,
            _drop_sender: drop_sender,
        }
    }

    #[wasm_bindgen]
    pub fn on_change(&self, promise: js_sys::Function) {
        let mut cb = self.callback_cell.borrow_mut();
        *cb = Some(promise);
    }

    #[wasm_bindgen]
    pub fn get(&self) -> js_sys::Uint8Array {
        let res = self.result_cell.borrow();
        let res = res.as_ref().unwrap();
        let res = res.as_ref().unwrap();
        let results_data = res.encode_to_vec().unwrap();
        js_sys::Uint8Array::from(results_data.as_ref())
    }
}

impl Drop for WatchedQuery {
    fn drop(&mut self) {
        debug!("WatchedQuery got dropped");
    }
}
