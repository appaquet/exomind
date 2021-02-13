use std::{ffi::CString, os::raw::c_void, sync::Arc, time::Duration};

use exocore_core::{
    cell::Cell,
    futures::Runtime,
    time::{Clock, ConsistentTimestamp},
};
use exocore_protos::prost::Message;
use exocore_protos::{
    generated::exocore_store::EntityQuery, prost::ProstMessageExt, store::MutationRequest,
};
use exocore_store::remote::{Client as StoreClient, ClientConfiguration, ClientHandle};
use exocore_transport::{
    p2p::Libp2pTransportConfig, Libp2pTransport, ServiceType, TransportServiceHandle,
};
use futures::StreamExt;

use crate::exocore_init;
use crate::node::LocalNode;
use crate::utils::CallbackContext;

/// Creates a new exocore client instance of a node that has join a cell.
///
/// The client needs to be freed with `exocore_client_free` once it's not needed
/// anymore. This will trigger runtime and connections to be cleaned up.
///
/// # Safety
/// * `node` should be a valid `LocalNode`.
/// * If return status code is success, a client is returned and needs to be
///   freed with `exocore_client_free`.
#[no_mangle]
pub unsafe extern "C" fn exocore_client_new(node: *mut LocalNode) -> ClientResult {
    exocore_init();

    let client = match Client::new(node) {
        Ok(client) => client,
        Err(err) => {
            return ClientResult {
                status: err,
                client: std::ptr::null_mut(),
            };
        }
    };

    ClientResult {
        status: ClientStatus::Success,
        client: Box::into_raw(Box::new(client)),
    }
}

#[repr(C)]
pub struct ClientResult {
    status: ClientStatus,
    client: *mut Client,
}

#[repr(u8)]
enum ClientStatus {
    Success = 0,
    Error,
}

/// Executes an entity mutation for which results or failure will be reported
/// via the given `callback`.
///
/// `mutation_bytes` and `mutation_size` describes a protobuf encoded
/// `EntityMutation`. It is is still owned by caller after call. Callback's
/// results are owned by the library.
///
/// `callback` is called exactly once (with `callback_ctx` as first argument)
/// when result is received or failed.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
/// * `query_bytes` needs to be a byte array of size `query_size`.
/// * `query_bytes` is owned by the caller.
/// * `callback_ctx` needs to be safe to send and use across threads.
/// * `callback_ctx` is owned by the caller and should be freed when after
///   callback got called.
#[no_mangle]
pub unsafe extern "C" fn exocore_store_mutate(
    client: *mut Client,
    mutation_bytes: *const libc::c_uchar,
    mutation_size: usize,
    callback: extern "C" fn(status: MutationStatus, *const libc::c_uchar, usize, *const c_void),
    callback_ctx: *const c_void,
) -> MutationHandle {
    let client = client.as_mut().unwrap();

    match client.mutate(mutation_bytes, mutation_size, callback, callback_ctx) {
        Ok(res) => res,
        Err(status) => MutationHandle { status },
    }
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

/// Executes an entity query for which results or failure will be reported via
/// the given `callback`.
///
/// `query_bytes` and `query_size` describes a protobuf encoded `EntityQuery`.
/// It is still owned by caller after call. Callback's results are owned by the
/// library.
///
/// `callback` is called exactly once (with `callback_ctx` as first argument)
/// when results are received or failed.
///
/// Unless it has already completed or failed, a query can be cancelled with
/// `exocore_store_query_cancelled`.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
/// * `query_bytes` needs to be a byte array of size `query_size`.
/// * `query_bytes` is owned by the caller.
/// * `callback_ctx` needs to be safe to send and use across threads.
/// * `callback_ctx` is owned by caller and should be freed when after callback
///   got called.
#[no_mangle]
pub unsafe extern "C" fn exocore_store_query(
    ctx: *mut Client,
    query_bytes: *const libc::c_uchar,
    query_size: usize,
    callback: extern "C" fn(status: QueryStatus, *const libc::c_uchar, usize, *const c_void),
    callback_ctx: *const c_void,
) -> QueryHandle {
    let client = ctx.as_mut().unwrap();

    match client.query(query_bytes, query_size, callback, callback_ctx) {
        Ok(res) => res,
        Err(status) => QueryHandle {
            status,
            query_id: 0,
        },
    }
}

/// Cancels a query for which results weren't returned yet.
///
/// If the query is successfully cancelled, the callback will be called with an
/// error status. and the context will need to be freed by caller.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
/// * It is OK to cancel a query even if it may have been cancelled, closed or
///   failed before.
#[no_mangle]
pub unsafe extern "C" fn exocore_store_query_cancel(client: *mut Client, handle: QueryHandle) {
    let client = client.as_mut().unwrap();

    if let Err(err) = client
        .store_handle
        .cancel_query(ConsistentTimestamp(handle.query_id))
    {
        error!("Error cancelling query: {}", err)
    }
}

#[repr(C)]
pub struct QueryHandle {
    status: QueryStatus,
    query_id: u64,
}

#[repr(u8)]
pub enum QueryStatus {
    Success = 0,
    Error,
}

/// Executes a watched entity query, for which a first version of the results
/// will be emitted and then new results will be emitted every time results have
/// changed. Calls are also made when an error occurred, after which no
/// subsequent calls to `callback` will be made.
///
/// `query_bytes` and `query_size` describes a protobuf encoded `EntityQuery`.
/// It is still owned by caller after call.
///
/// `callback` is called (with `callback_ctx` as first argument) when results
/// are received, or when the watched has completed. When a call with a `Done`
/// or `Error` status is made, no results are given and no further calls will be
/// done. Callback's results are owned by the library.
///
/// Unless it has already completed or failed, a watched query needs to be
/// cancelled with `exocore_store_watched_query_cancelled`.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
/// * `query_bytes` needs to be a byte array of size `query_size`.
/// * `query_bytes` are owned by the caller.
/// * `callback_ctx` needs to be safe to send and use across threads.
/// * `callback_ctx` is owned by client and should be freed when receiving a
///   `Done` or `Error` status.
#[no_mangle]
pub unsafe extern "C" fn exocore_store_watched_query(
    client: *mut Client,
    query_bytes: *const libc::c_uchar,
    query_size: usize,
    callback: extern "C" fn(status: WatchedQueryStatus, *const libc::c_uchar, usize, *const c_void),
    callback_ctx: *const c_void,
) -> WatchedQueryHandle {
    let client = client.as_mut().unwrap();

    match client.watched_query(query_bytes, query_size, callback, callback_ctx) {
        Ok(res) => res,
        Err(status) => WatchedQueryHandle {
            status,
            query_id: 0,
        },
    }
}

#[repr(u8)]
pub enum WatchedQueryStatus {
    Success = 0,
    Done,
    Error,
}

#[repr(C)]
pub struct WatchedQueryHandle {
    status: WatchedQueryStatus,
    query_id: u64,
}

/// Cancels a `WatchedQuery` so that no further results can be received.
///
/// It is OK to cancel a query even if it may have already been cancelled,
/// closed or failed. If the query is successfully cancelled, the callback will
/// be called with a `Done` status, and the context will need to be freed by
/// caller.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
#[no_mangle]
pub unsafe extern "C" fn exocore_store_watched_query_cancel(
    client: *mut Client,
    handle: WatchedQueryHandle,
) {
    let client = client.as_mut().unwrap();

    if let Err(err) = client
        .store_handle
        .cancel_query(ConsistentTimestamp(handle.query_id))
    {
        error!("Error cancelling query stream: {}", err)
    }
}

/// Returns a list of HTTP endpoints available on nodes of the cell, returned as
/// a `;` delimited string.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
/// * Returned string must be freed using `exocore_free_string`.
#[no_mangle]
pub unsafe extern "C" fn exocore_store_http_endpoints(client: *mut Client) -> *mut libc::c_char {
    let client = client.as_mut().unwrap();

    let store_node_urls = client
        .store_handle
        .store_node()
        .map(|node| node.http_addresses())
        .unwrap_or_else(Vec::new)
        .into_iter()
        .map(|url| url.to_string())
        .collect::<Vec<_>>();

    let joined = store_node_urls.join(";");

    CString::new(joined).unwrap().into_raw()
}

/// Returns a standalone authentication token that can be used via an HTTP
/// endpoint.
///
/// If a 0 value is given for `expiration_days`, the token will never expire.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
/// * Returned string must be freed using `exocore_free_string`.
#[no_mangle]
pub unsafe extern "C" fn exocore_cell_generate_auth_token(
    client: *mut Client,
    expiration_days: usize,
) -> *mut libc::c_char {
    let client = client.as_mut().unwrap();

    let expiration = if expiration_days > 0 {
        let now = client
            .clock
            .consistent_time(client.cell.local_node().node());
        Some(now + Duration::from_secs(expiration_days as u64 * 86400))
    } else {
        None
    };

    let auth_token =
        exocore_core::sec::auth_token::AuthToken::new(&client.cell, &client.clock, expiration);
    let auth_token = if let Ok(token) = auth_token {
        token
    } else {
        return CString::new("").unwrap().into_raw();
    };

    let auth_token_bs58 = auth_token.encode_base58_string();

    CString::new(auth_token_bs58).unwrap().into_raw()
}

/// Frees an instance of exocore client.
///
/// # Safety
/// * `client` needs to be a valid `Client`.
/// * This method shall only be called once per instance.
#[no_mangle]
pub unsafe extern "C" fn exocore_client_free(client: *mut Client) {
    let client = Box::from_raw(client);
    drop(client);
}

/// Exocore client instance of a bootstrapped node.
///
/// This structure is opaque to the client and is used as context for calls.
pub struct Client {
    _runtime: Runtime,
    clock: Clock,
    cell: Cell,
    store_handle: Arc<ClientHandle>,
}

impl Client {
    unsafe fn new(node: *mut LocalNode) -> Result<Client, ClientStatus> {
        let local_node = node.as_mut().unwrap();

        let (either_cells, local_node) =
            Cell::from_local_node(local_node.node.clone()).map_err(|err| {
                error!("Error creating cell: {}", err);
                ClientStatus::Error
            })?;

        let either_cell = either_cells.first().cloned().ok_or_else(|| {
            error!("Configuration doesn't have any cell config");
            ClientStatus::Error
        })?;

        let cell = either_cell.cell().clone();

        let runtime = Runtime::new().map_err(|err| {
            error!("Couldn't start a tokio Runtime: {}", err);
            ClientStatus::Error
        })?;

        let transport_config = Libp2pTransportConfig::default();
        let mut transport = Libp2pTransport::new(local_node, transport_config);

        let clock = Clock::new();

        let store_transport = transport
            .get_handle(cell.clone(), ServiceType::Store)
            .map_err(|err| {
                error!("Couldn't get transport handle for remote store: {}", err);
                ClientStatus::Error
            })?;
        let remote_store_config = ClientConfiguration::default();
        let remote_store_client = StoreClient::new(
            remote_store_config,
            cell.clone(),
            clock.clone(),
            store_transport,
        )
        .map_err(|err| {
            error!("Couldn't create remote store client: {}", err);
            ClientStatus::Error
        })?;

        let store_handle = Arc::new(remote_store_client.get_handle());
        let management_transport_handle = transport
            .get_handle(cell.clone(), ServiceType::None)
            .map_err(|err| {
                error!("Couldn't get transport handle: {}", err);
                ClientStatus::Error
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

        Ok(Client {
            _runtime: runtime,
            clock,
            cell,
            store_handle,
        })
    }

    unsafe fn mutate(
        &mut self,
        mutation_bytes: *const libc::c_uchar,
        mutation_size: usize,
        callback: extern "C" fn(status: MutationStatus, *const libc::c_uchar, usize, *const c_void),
        callback_ctx: *const c_void,
    ) -> Result<MutationHandle, MutationStatus> {
        let mutation_bytes = std::slice::from_raw_parts(mutation_bytes, mutation_size);
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
                    debug!("Mutation result received");

                    let encoded = res.encode_to_vec();
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

    unsafe fn query(
        &mut self,
        query_bytes: *const libc::c_uchar,
        query_size: usize,
        callback: extern "C" fn(status: QueryStatus, *const libc::c_uchar, usize, *const c_void),
        callback_ctx: *const c_void,
    ) -> Result<QueryHandle, QueryStatus> {
        let query_bytes = std::slice::from_raw_parts(query_bytes, query_size);
        let query = EntityQuery::decode(query_bytes).map_err(|_| QueryStatus::Error)?;

        let future_result = self.store_handle.query(query);
        let query_id = future_result.query_id();

        debug!("Sending a query");
        let callback_ctx = CallbackContext { ctx: callback_ctx };
        self._runtime.spawn(async move {
            let result = future_result.await;
            match result {
                Ok(res) => {
                    debug!("Query results received");

                    let encoded = res.encode_to_vec();
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

    unsafe fn watched_query(
        &mut self,
        query_bytes: *const libc::c_uchar,
        query_size: usize,
        callback: extern "C" fn(
            status: WatchedQueryStatus,
            *const libc::c_uchar,
            usize,
            *const c_void,
        ),
        callback_ctx: *const c_void,
    ) -> Result<WatchedQueryHandle, WatchedQueryStatus> {
        let query_bytes = std::slice::from_raw_parts(query_bytes, query_size);
        let query = EntityQuery::decode(query_bytes).map_err(|_| WatchedQueryStatus::Error)?;

        let result_stream = self.store_handle.watched_query(query);
        let query_id = result_stream.query_id();

        debug!("Sending a watch query");
        let callback_ctx = CallbackContext { ctx: callback_ctx };
        self._runtime.spawn(async move {
            let mut stream = result_stream;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(res) => {
                        debug!("Watched query results received");

                        let encoded = res.encode_to_vec();
                        callback(
                            WatchedQueryStatus::Success,
                            encoded.as_ptr(),
                            encoded.len(),
                            callback_ctx.ctx,
                        );
                    }

                    Err(err) => {
                        warn!("Watched query has failed: {}", err);
                        callback(
                            WatchedQueryStatus::Error,
                            std::ptr::null(),
                            0,
                            callback_ctx.ctx,
                        );
                        return;
                    }
                }
            }

            info!("Watched query done");
            callback(
                WatchedQueryStatus::Done,
                std::ptr::null(),
                0,
                callback_ctx.ctx,
            );
        });

        Ok(WatchedQueryHandle {
            status: WatchedQueryStatus::Success,
            query_id: query_id.0,
        })
    }
}
