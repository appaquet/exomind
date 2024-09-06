use std::{cell::RefCell, rc::Rc};

use exocore_core::futures::spawn_future_non_send;
use exocore_protos::{
    generated::exocore_store::{EntityQuery, EntityResults},
    prost::Message,
};
use exocore_store::{remote::ClientHandle, store::Store};
use futures::{channel::oneshot, prelude::*};
use wasm_bindgen::prelude::*;

use crate::js::into_js_error;

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
    pub(crate) fn new(store_handle: Rc<ClientHandle>, query: EntityQuery) -> WatchedQuery {
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
                    if let Err(err) = func.call0(&JsValue::null()) {
                        error!("Error calling watch query callback: {:?}", err);
                    }
                }
            }
        };

        let (drop_sender, drop_receiver) = oneshot::channel();
        spawn_future_non_send(async move {
            let mut results = match store_handle.watched_query(query) {
                Ok(ok) => ok,
                Err(err) => {
                    report_result(Err(err));
                    return Ok(());
                }
            };
            let mut drop_receiver = drop_receiver.fuse();

            loop {
                futures::select! {
                    result = results.next().fuse() => {
                        let Some(result) = result else {
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
    pub fn get(&self) -> Result<js_sys::Uint8Array, JsValue> {
        let res = self.result_cell.borrow();
        let res = res.as_ref().ok_or("couldn't borrow inner results")?;
        let res = res
            .as_ref()
            .map_err(|err| into_js_error("query is an error", err))?;
        let results_data = res.encode_to_vec();
        Ok(js_sys::Uint8Array::from(results_data.as_ref()))
    }
}

impl Drop for WatchedQuery {
    fn drop(&mut self) {
        debug!("WatchedQuery got dropped");
    }
}
