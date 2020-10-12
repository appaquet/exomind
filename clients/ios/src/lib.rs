#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[macro_use]
extern crate log;

use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::sync::Arc;

use futures::StreamExt;
use prost::Message;

use exocore_core::cell::Cell;
use exocore_core::futures::Runtime;
use exocore_core::protos::generated::exocore_core::LocalNodeConfig;
use exocore_core::protos::generated::exocore_store::EntityQuery;
use exocore_core::protos::{prost::ProstMessageExt, store::MutationRequest};
use exocore_core::time::{Clock, ConsistentTimestamp};
use exocore_core::utils::id::{generate_id, generate_prefixed_id};
use exocore_store::remote::{Client, ClientConfiguration, ClientHandle};
use exocore_transport::p2p::Libp2pTransportConfig;
use exocore_transport::{Libp2pTransport, ServiceType, TransportServiceHandle};

pub struct Context {
    _runtime: Runtime,
    store_handle: Arc<ClientHandle>,
}

impl Context {
    fn new(
        config_bytes: *const libc::c_uchar,
        config_size: usize,
        config_format: ConfigFormat,
    ) -> Result<Context, ContextStatus> {
        exocore_core::logging::setup(Some(log::LevelFilter::Debug));

        let config_bytes = unsafe { std::slice::from_raw_parts(config_bytes, config_size) };
        let config = match config_format {
            ConfigFormat::Protobuf => LocalNodeConfig::decode(config_bytes).map_err(|err| {
                error!("Couldn't decode node config from Protobuf: {}", err);
                ContextStatus::Error
            })?,
            ConfigFormat::Yaml => {
                exocore_core::cell::node_config_from_yaml(config_bytes).map_err(|err| {
                    error!("Couldn't parse node config from YAML: {}", err);
                    ContextStatus::Error
                })?
            }
        };

        let (either_cells, local_node) =
            Cell::new_from_local_node_config(config).map_err(|err| {
                error!("Error creating cell: {}", err);
                ContextStatus::Error
            })?;

        let either_cell = either_cells.first().cloned().ok_or_else(|| {
            error!("Configuration doesn't have any cell config");
            ContextStatus::Error
        })?;

        let cell = either_cell.cell().clone();

        let mut runtime = Runtime::new().map_err(|err| {
            error!("Couldn't start Runtime: {}", err);
            ContextStatus::Error
        })?;

        let transport_config = Libp2pTransportConfig::default();
        let mut transport = Libp2pTransport::new(local_node, transport_config);

        let clock = Clock::new();

        let store_transport = transport
            .get_handle(cell.clone(), ServiceType::Store)
            .map_err(|err| {
                error!("Couldn't get transport handle for remote store: {}", err);
                ContextStatus::Error
            })?;
        let remote_store_config = ClientConfiguration::default();
        let remote_store_client =
            Client::new(remote_store_config, cell.clone(), clock, store_transport).map_err(
                |err| {
                    error!("Couldn't create remote store client: {}", err);
                    ContextStatus::Error
                },
            )?;

        let store_handle = Arc::new(remote_store_client.get_handle());
        let management_transport_handle =
            transport
                .get_handle(cell, ServiceType::None)
                .map_err(|err| {
                    error!("Couldn't get transport handle: {}", err);
                    ContextStatus::Error
                })?;

        runtime.spawn(async move {
            let res = transport.run().await;
            info!("Transport is done: {:?}", res);
        });

        runtime.block_on(management_transport_handle.on_started());

        runtime.spawn(async move {
            let _ = remote_store_client.run().await;
            info!("Remote store is done");
        });

        Ok(Context {
            _runtime: runtime,
            store_handle,
        })
    }

    pub fn mutate(
        &mut self,
        mutation_bytes: *const libc::c_uchar,
        mutation_size: usize,
        callback: extern "C" fn(status: MutationStatus, *const libc::c_uchar, usize, *const c_void),
        callback_ctx: *const c_void,
    ) -> Result<MutationHandle, MutationStatus> {
        let mutation_bytes = unsafe { std::slice::from_raw_parts(mutation_bytes, mutation_size) };
        let mutation =
            MutationRequest::decode(mutation_bytes).map_err(|_| MutationStatus::Error)?;

        let store_handle = self.store_handle.clone();

        debug!("Sending a mutation");
        let callback_ctx = CallbackContext { ctx: callback_ctx };
        self._runtime.spawn(async move {
            let future_result = store_handle.mutate(mutation);

            let result = future_result.await;
            match result {
                Ok(res) => {
                    let encoded = match res.encode_to_vec() {
                        Ok(res) => res,
                        Err(err) => {
                            error!("Error decoding mutation result: {}", err);
                            callback(MutationStatus::Error, std::ptr::null(), 0, callback_ctx.ctx);
                            return;
                        }
                    };

                    debug!("Mutation result received");
                    callback(
                        MutationStatus::Success,
                        encoded.as_ptr(),
                        encoded.len(),
                        callback_ctx.ctx,
                    );
                }

                Err(err) => {
                    warn!("Mutation future has failed: {}", err);
                    callback(MutationStatus::Error, std::ptr::null(), 0, callback_ctx.ctx);
                }
            }
        });

        Ok(MutationHandle {
            status: MutationStatus::Success,
        })
    }

    pub fn query(
        &mut self,
        query_bytes: *const libc::c_uchar,
        query_size: usize,
        callback: extern "C" fn(status: QueryStatus, *const libc::c_uchar, usize, *const c_void),
        callback_ctx: *const c_void,
    ) -> Result<QueryHandle, QueryStatus> {
        let query_bytes = unsafe { std::slice::from_raw_parts(query_bytes, query_size) };
        let query = EntityQuery::decode(query_bytes).map_err(|_| QueryStatus::Error)?;

        let future_result = self.store_handle.query(query);
        let query_id = future_result.query_id();

        debug!("Sending a query");
        let callback_ctx = CallbackContext { ctx: callback_ctx };
        self._runtime.spawn(async move {
            let result = future_result.await;
            match result {
                Ok(res) => {
                    let encoded = match res.encode_to_vec() {
                        Ok(res) => res,
                        Err(err) => {
                            error!("Error decoding query result: {}", err);
                            callback(QueryStatus::Error, std::ptr::null(), 0, callback_ctx.ctx);
                            return;
                        }
                    };

                    debug!("Query results received");
                    callback(
                        QueryStatus::Success,
                        encoded.as_ptr(),
                        encoded.len(),
                        callback_ctx.ctx,
                    );
                }

                Err(err) => {
                    warn!("Query future has failed: {}", err);
                    callback(QueryStatus::Error, std::ptr::null(), 0, callback_ctx.ctx);
                }
            }
        });

        Ok(QueryHandle {
            status: QueryStatus::Success,
            query_id: query_id.0,
        })
    }

    pub fn watched_query(
        &mut self,
        query_bytes: *const libc::c_uchar,
        query_size: usize,
        callback: extern "C" fn(status: QueryStatus, *const libc::c_uchar, usize, *const c_void),
        callback_ctx: *const c_void,
    ) -> Result<QueryStreamHandle, QueryStreamStatus> {
        let query_bytes = unsafe { std::slice::from_raw_parts(query_bytes, query_size) };
        let query = EntityQuery::decode(query_bytes).map_err(|_| QueryStreamStatus::Error)?;

        let result_stream = self.store_handle.watched_query(query);
        let query_id = result_stream.query_id();

        debug!("Sending a watch query");
        let callback_ctx = CallbackContext { ctx: callback_ctx };
        self._runtime.spawn(async move {
            let mut stream = result_stream;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(res) => {
                        let encoded = match res.encode_to_vec() {
                            Ok(res) => res,
                            Err(err) => {
                                error!("Error decoding watched query result: {}", err);
                                callback(QueryStatus::Error, std::ptr::null(), 0, callback_ctx.ctx);
                                return;
                            }
                        };

                        debug!("Watched query results received");
                        callback(
                            QueryStatus::Success,
                            encoded.as_ptr(),
                            encoded.len(),
                            callback_ctx.ctx,
                        );
                    }

                    Err(err) => {
                        warn!("Watched query has failed: {}", err);
                        callback(QueryStatus::Error, std::ptr::null(), 0, callback_ctx.ctx);
                        return;
                    }
                }
            }

            info!("Watched query done");
            callback(QueryStatus::Done, std::ptr::null(), 0, callback_ctx.ctx);
        });

        Ok(QueryStreamHandle {
            status: QueryStreamStatus::Success,
            query_id: query_id.0,
        })
    }
}

#[repr(C)]
pub struct ContextResult {
    status: ContextStatus,
    context: *mut Context,
}

#[repr(u8)]
enum ContextStatus {
    Success = 0,
    Error,
}

struct CallbackContext {
    ctx: *const c_void,
}

unsafe impl Send for CallbackContext {}

unsafe impl Sync for CallbackContext {}

#[repr(u8)]
pub enum ConfigFormat {
    Protobuf = 0,
    Yaml,
}

#[repr(u8)]
pub enum QueryStatus {
    Success = 0,
    Done = 1,
    Error,
}

#[repr(C)]
pub struct QueryHandle {
    status: QueryStatus,
    query_id: u64,
}

#[repr(u8)]
pub enum QueryStreamStatus {
    Success = 0,
    Done,
    Error,
}

#[repr(C)]
pub struct QueryStreamHandle {
    status: QueryStreamStatus,
    query_id: u64,
}

#[repr(u8)]
pub enum MutationStatus {
    Success = 0,
    Error,
}

#[repr(C)]
pub struct MutationHandle {
    status: MutationStatus,
}

#[no_mangle]
pub extern "C" fn exocore_context_new(
    config_bytes: *const libc::c_uchar,
    config_size: usize,
    config_format: ConfigFormat,
) -> ContextResult {
    let context = match Context::new(config_bytes, config_size, config_format) {
        Ok(context) => context,
        Err(err) => {
            return ContextResult {
                status: err,
                context: std::ptr::null_mut(),
            };
        }
    };

    ContextResult {
        status: ContextStatus::Success,
        context: Box::into_raw(Box::new(context)),
    }
}

#[no_mangle]
pub extern "C" fn exocore_context_free(ctx: *mut Context) {
    let context = unsafe { Box::from_raw(ctx) };
    drop(context);
}

#[no_mangle]
pub extern "C" fn exocore_mutate(
    ctx: *mut Context,
    mutation_bytes: *const libc::c_uchar,
    mutation_size: usize,
    callback: extern "C" fn(status: MutationStatus, *const libc::c_uchar, usize, *const c_void),
    callback_ctx: *const c_void,
) -> MutationHandle {
    let context = unsafe { ctx.as_mut().unwrap() };

    match context.mutate(mutation_bytes, mutation_size, callback, callback_ctx) {
        Ok(res) => res,
        Err(status) => MutationHandle { status },
    }
}

#[no_mangle]
pub extern "C" fn exocore_query(
    ctx: *mut Context,
    query_bytes: *const libc::c_uchar,
    query_size: usize,
    callback: extern "C" fn(status: QueryStatus, *const libc::c_uchar, usize, *const c_void),
    callback_ctx: *const c_void,
) -> QueryHandle {
    let context = unsafe { ctx.as_mut().unwrap() };

    match context.query(query_bytes, query_size, callback, callback_ctx) {
        Ok(res) => res,
        Err(status) => QueryHandle {
            status,
            query_id: 0,
        },
    }
}

#[no_mangle]
pub extern "C" fn exocore_query_cancel(ctx: *mut Context, handle: QueryHandle) {
    let context = unsafe { ctx.as_mut().unwrap() };

    if let Err(err) = context
        .store_handle
        .cancel_query(ConsistentTimestamp(handle.query_id))
    {
        error!("Error cancelling query: {}", err)
    }
}

#[no_mangle]
pub extern "C" fn exocore_watched_query(
    ctx: *mut Context,
    query_bytes: *const libc::c_uchar,
    query_size: usize,
    callback: extern "C" fn(status: QueryStatus, *const libc::c_uchar, usize, *const c_void),
    callback_ctx: *const c_void,
) -> QueryStreamHandle {
    let context = unsafe { ctx.as_mut().unwrap() };

    match context.watched_query(query_bytes, query_size, callback, callback_ctx) {
        Ok(res) => res,
        Err(status) => QueryStreamHandle {
            status,
            query_id: 0,
        },
    }
}

#[no_mangle]
pub extern "C" fn exocore_watched_query_cancel(ctx: *mut Context, handle: QueryStreamHandle) {
    let context = unsafe { ctx.as_mut().unwrap() };

    if let Err(err) = context
        .store_handle
        .cancel_query(ConsistentTimestamp(handle.query_id))
    {
        error!("Error cancelling query stream: {}", err)
    }
}

#[no_mangle]
pub extern "C" fn exocore_generate_id(prefix: *const libc::c_char) -> *mut libc::c_char {
    let generated = if prefix.is_null() {
        generate_id()
    } else {
        let prefix = unsafe { CStr::from_ptr(prefix).to_string_lossy() };
        generate_prefixed_id(&prefix)
    };

    CString::new(generated).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn exocore_free_string(ptr: *mut libc::c_char) {
    unsafe { drop(CString::from_raw(ptr)) }
}
