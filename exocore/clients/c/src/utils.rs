use std::ffi::{CStr, CString};

use exocore_core::utils::id::{generate_id, generate_prefixed_id};

/// Generates a unique identifier, optionally prefixed by the given string.
///
/// # Safety
/// * `prefix` can be null OR a valid string.
/// * `prefix` is owned by caller.
/// * Returned string must be freed using `exocore_free_string`.
#[no_mangle]
pub unsafe extern "C" fn exocore_generate_id(prefix: *const libc::c_char) -> *mut libc::c_char {
    let generated = if prefix.is_null() {
        generate_id()
    } else {
        let prefix = CStr::from_ptr(prefix).to_string_lossy();
        generate_prefixed_id(&prefix)
    };

    CString::new(generated).unwrap().into_raw()
}

/// Frees a string returned by one of the method of this library.
///
/// # Safety
/// * `ptr` should be a valid string returned by one of the method of this
///   library.
/// * This method shall only be called once per string.
#[no_mangle]
pub unsafe extern "C" fn exocore_free_string(ptr: *mut libc::c_char) {
    drop(CString::from_raw(ptr))
}

/// Bytes vector allocated by Rust and returned by methods of this library.
///
/// # Safety
/// * Needs to be freed using `exocore_bytes_free`
#[repr(C)]
pub struct BytesVec {
    // Data
    pub bytes: *mut libc::c_uchar,

    // Number of usable bytes.
    pub size: usize,
}

impl BytesVec {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        // TODO: Use `into_raw_parts` once its stabilized
        let size = bytes.len();
        let boxed = bytes.into_boxed_slice();
        Self {
            bytes: Box::into_raw(boxed) as *mut libc::c_uchar,
            size,
        }
    }
}

/// Frees `BytesVec` return by one of the method of this library.
///
/// # Safety
/// * This method shall only be called once per instance of `BytesVec`
#[no_mangle]
pub unsafe extern "C" fn exocore_bytes_free(bytes: BytesVec) {
    let bytes = Box::from_raw(bytes.bytes);
    drop(bytes)
}

/// Used to wrap the context passed by client to be included in a callback call.
///
/// This wrapping is necessary to make the point Send + Sync since Rust doesn't
/// know if it's safe to do it. In our case, we push the burden to the client to
/// make sure can safely be send and used across threads.
pub(crate) struct CallbackContext {
    pub(crate) ctx: *const libc::c_void,
}

unsafe impl Send for CallbackContext {}

unsafe impl Sync for CallbackContext {}
