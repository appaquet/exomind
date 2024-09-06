use std::{ffi::CStr, sync::Arc, time::Duration};

use exocore_core::{
    cell::{CellConfigExt, CellNodeConfigExt, LocalNodeConfigExt},
    futures::Runtime,
};
use exocore_discovery::{Client, Pin, DEFAULT_DISCO_SERVER};
use exocore_protos::core::{node_cell_config, CellConfig, LocalNodeConfig, NodeCellConfig};

use crate::{node::LocalNode, utils::CallbackContext};

pub struct Discovery {
    runtime: Runtime,
    client: Arc<Client>,
}

/// Creates a new discovery service client. This client can be used to join a
/// cell by using the discovery service to exchange node and cell configs.
///
/// If null is passed to `service_url`, the default service will be used.
///
/// # Safety
/// * `service_url` needs to be a valid \0 delimited string or null.
/// * Returns a `DiscoveryResult` in which the `status` field indicates if the
///   client was successfully created. In case it wasn't, the `client` field
///   will be null.
/// * If the method succeeds, the `client` in result needs to be freed using
///   `exocore_discovery_free`.
#[no_mangle]
pub unsafe extern "C" fn exocore_discovery_new(
    service_url: *const libc::c_char,
) -> DiscoveryResult {
    let service_url = if service_url.is_null() {
        DEFAULT_DISCO_SERVER.to_string()
    } else {
        CStr::from_ptr(service_url).to_string_lossy().to_string()
    };

    let client = match Client::new(service_url) {
        Ok(cli) => cli,
        Err(err) => {
            error!("Couldn't create discovery client: {}", err);
            return DiscoveryResult::err();
        }
    };

    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(err) => {
            error!("Couldn't start a Tokio runtime: {}", err);
            return DiscoveryResult::err();
        }
    };

    DiscoveryResult {
        status: DiscoveryStatus::Success,
        discovery: Box::into_raw(Box::new(Discovery {
            runtime,
            client: Arc::new(client),
        })),
    }
}

/// Requests for `LocalNode` to join a cell by using the discovery service.
///
/// The process of joining a node to a cell is as follow:
///
///   1) The node config will be pushed to the discovery service, which will
///      return a discovery pin and a reply pin.
///
///   2) The `callback` is called with the `InProgress` status and the discovery
///      pin to be displayed to the user.
///
///   3) The discovery pin is entered on a node currently in the cell, which
///      will get the config of the joining node. The joining node will be added
///      to the cell and the cell's config will be pushed back to discovery
///      service on the  reply pin.
///
///   4) The cell configuration is then fetched from the discovery service using
///      the reply pin. The cell is added to local node configuration and the
///      `callback` is called with a `Success` status and the new `LocalNode`
///      config.
///
///   *) If any errors occurred during the process, the `callback` is called
///      with a  `Error` status. No pin and LocalNode will be specified. The
///      discovery client still needs to be freed.
///
/// # Safety
/// * `disco` needs to be a valid client created with `exocore_disco_new`.
/// * `node` needs to be a valid LocalNode created with `exocore_local_node_*`
///   and is still owned by caller after.
/// * `callback_ctx` needs to be safe to send and use across threads.
/// * `callback_ctx` is owned by the caller and should be freed when after
///   callback got called.
/// * On success, the `LocalNode` passed is owned by client and will need to be
///   freed.
#[no_mangle]
#[allow(clippy::redundant_locals)] // because of redefinition of callback_ctx
pub unsafe extern "C" fn exocore_discovery_join_cell(
    disco: *mut Discovery,
    node: *mut LocalNode,
    callback: extern "C" fn(status: DiscoveryStatus, u32, *mut LocalNode, *const libc::c_void),
    callback_ctx: *const libc::c_void,
) {
    let disco = disco.as_mut().unwrap();
    let node = node.as_mut().unwrap();
    let node_config = node.node.config().clone();

    let callback_ctx = CallbackContext { ctx: callback_ctx };
    let client = disco.client.clone();
    let fut = async move {
        let callback_ctx = callback_ctx; // required since the struct is send + sync, not the field
        let reply_pin = match push_config(&node_config, client.clone()).await {
            Ok((disco_pin, reply_pin)) => {
                info!(
                    "Node config pushed to discovery service. Pin: {} Reply pin: {}",
                    disco_pin.to_formatted_string(),
                    reply_pin.to_formatted_string(),
                );
                callback(
                    DiscoveryStatus::InProgress,
                    disco_pin.into(),
                    std::ptr::null_mut(),
                    callback_ctx.ctx,
                );

                reply_pin
            }
            Err(_) => {
                error!("Couldn't join cell. Failed to push config.");
                callback(
                    DiscoveryStatus::Error,
                    0,
                    std::ptr::null_mut(),
                    callback_ctx.ctx,
                );
                return;
            }
        };

        match join_cell(&node_config, client, reply_pin).await {
            Ok(node) => {
                info!("Successfully joined cell");
                let node_ptr = Box::into_raw(Box::new(node));
                callback(DiscoveryStatus::Success, 0, node_ptr, callback_ctx.ctx);
            }
            Err(_) => {
                error!("Couldn't join cell. Failed to receive cell.");
                callback(
                    DiscoveryStatus::Error,
                    0,
                    std::ptr::null_mut(),
                    callback_ctx.ctx,
                );
            }
        }
    };
    disco.runtime.spawn(fut);
}

/// Frees an instance of `Discovery`.
///
/// # Safety
/// * `node` needs to be a valid node created by `exocore_discovery_new`.
/// * This method shall only be called once per instance.
#[no_mangle]
pub unsafe extern "C" fn exocore_discovery_free(disco: *mut Discovery) {
    let disco = Box::from_raw(disco);
    drop(disco);
}

/// Parts of the cell joining process. Pushes the node's config to the discovery
/// service and returns a discovery pin and reply pin.
async fn push_config(node_config: &LocalNodeConfig, client: Arc<Client>) -> Result<(Pin, Pin), ()> {
    let roles = Vec::new(); // thin client, no roles for now
    let cell_node = node_config.create_cell_node_config(roles);
    let cell_node_yml = cell_node.to_yaml_string().map_err(|err| {
        error!("Error converting config to yaml: {}", err);
    })?;

    let create_resp = client
        .create(cell_node_yml.as_bytes(), true)
        .await
        .map_err(|err| error!("Error pushing node config to discovery service: {}", err))?;

    let disco_pin = create_resp.pin;
    let reply_pin = create_resp
        .reply_pin
        .ok_or_else(|| error!("Expected reply pin from discovery service, not none"))?;

    Ok((disco_pin, reply_pin))
}

/// Parts of the cell joining process. Waits for the cell's config to be pushed
/// to the discovery service on the reply pin.
async fn join_cell(
    node_config: &LocalNodeConfig,
    client: Arc<Client>,
    reply_pin: Pin,
) -> Result<LocalNode, ()> {
    let mut node_config = node_config.clone();

    let get_cell_resp = client
        .get_loop(reply_pin, Duration::from_secs(60))
        .await
        .map_err(|err| error!("Error getting config to discovery service: {}", err))?;

    let get_cell_payload = get_cell_resp
        .decode_payload()
        .map_err(|err| error!("Couldn't decode payload from discovery service: {}", err))?;
    let cell_config = CellConfig::read_yaml(get_cell_payload.as_slice())
        .map_err(|err| error!("Couldn't decode config retrieved from discovery: {}", err))?;

    node_config.add_cell(NodeCellConfig {
        location: Some(node_cell_config::Location::Inline(cell_config)),
        ..Default::default()
    });

    let local_node = LocalNode::from_config(node_config)
        .map_err(|_| error!("Couldn't create local node from config"))?;

    Ok(local_node)
}

#[repr(C)]
pub struct DiscoveryResult {
    status: DiscoveryStatus,
    discovery: *mut Discovery,
}

impl DiscoveryResult {
    fn err() -> DiscoveryResult {
        DiscoveryResult {
            status: DiscoveryStatus::Error,
            discovery: std::ptr::null_mut(),
        }
    }
}

#[repr(u8)]
pub enum DiscoveryStatus {
    Success = 0,
    Error,
    InProgress,
}
