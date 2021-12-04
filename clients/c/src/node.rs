use exocore_core::{cell::LocalNode as CoreLocalNode, dir::ram::RamDirectory};
use exocore_protos::{core::LocalNodeConfig, prost::Message};

use crate::utils::BytesVec;

/// LocalNode that may or may not be part of a cell.
///
/// This structure is opaque to the client and is used as context for calls.
pub struct LocalNode {
    pub(crate) node: CoreLocalNode,
}

impl LocalNode {
    pub(crate) fn from_config(config: LocalNodeConfig) -> Result<LocalNode, ()> {
        let core_local_node = match CoreLocalNode::from_config(RamDirectory::new(), config) {
            Ok(node) => node,
            Err(err) => {
                error!("Couldn't create LocalNode from config: {}", err);
                return Err(());
            }
        };

        Ok(LocalNode {
            node: core_local_node,
        })
    }
}

/// Generates a `LocalNode` with a randomly generated Keypair.
///
/// # Safety
/// * Needs to be freed using `exocore_local_node_free`.
/// * If result's status is not success, the node pointer will be null and
///   shouldn't be freed.
#[no_mangle]
pub unsafe extern "C" fn exocore_local_node_generate() -> LocalNodeResult {
    let core_local_node = CoreLocalNode::generate();
    let config = core_local_node.config().clone();

    match LocalNode::from_config(config) {
        Ok(node) => node.into(),
        Err(_) => LocalNodeResult::err(),
    }
}

/// Creates a new `LocalNode` from a `LocalNodeConfig` protobuf encoded message.
///
/// # Safety
/// * `config_bytes` needs to be a byte array of size `config_bytes_size`.
/// * `config_bytes` are owned by the caller.
/// * Needs to be freed using `exocore_local_node_free`.
/// * If result's status is not success, the node pointer will be null and
///   shouldn't be freed.
#[no_mangle]
pub unsafe extern "C" fn exocore_local_node_new(
    config_bytes: *const libc::c_uchar,
    config_bytes_size: usize,
) -> LocalNodeResult {
    let config_bytes = std::slice::from_raw_parts(config_bytes, config_bytes_size);
    let config = match LocalNodeConfig::decode(config_bytes) {
        Ok(cfg) => cfg,
        Err(err) => {
            error!(
                "Couldn't decode LocalNodeConfig protobuf from bytes: {}",
                err
            );
            return LocalNodeResult::err();
        }
    };

    match LocalNode::from_config(config) {
        Ok(node) => node.into(),
        Err(_) => LocalNodeResult::err(),
    }
}

/// Returns protobuf bytes of a encoded `LocalNodeConfig` message of the node.
///
/// # Safety
/// * `node` needs to be a valid node created by `exocore_local_node_*`.
/// * Returned `BytesVec` will be owned by caller and needs to be freed using
///   `exocore_bytes_free`.
#[no_mangle]
pub unsafe extern "C" fn exocore_local_node_protobuf_config(node: *mut LocalNode) -> BytesVec {
    let node = node.as_mut().unwrap();
    let encoded = node
        .node
        .inlined_config()
        .expect("Couldn't inline config")
        .encode_to_vec();
    BytesVec::from_vec(encoded)
}

/// Frees an instance of `LocalNode`.
///
/// # Safety
/// * `node` needs to be a valid node created by `exocore_local_node_*`.
/// * This method shall only be called once per instance.
#[no_mangle]
pub unsafe extern "C" fn exocore_local_node_free(node: *mut LocalNode) {
    let node = Box::from_raw(node);
    drop(node);
}

#[repr(C)]
pub struct LocalNodeResult {
    status: LocalNodeStatus,
    node: *mut LocalNode,
}

impl From<LocalNode> for LocalNodeResult {
    fn from(node: LocalNode) -> Self {
        LocalNodeResult {
            status: LocalNodeStatus::Success,
            node: Box::into_raw(Box::new(node)),
        }
    }
}

impl LocalNodeResult {
    fn err() -> LocalNodeResult {
        LocalNodeResult {
            status: LocalNodeStatus::Error,
            node: std::ptr::null_mut(),
        }
    }
}

#[repr(u8)]
pub enum LocalNodeStatus {
    Success = 0,
    Error,
}
