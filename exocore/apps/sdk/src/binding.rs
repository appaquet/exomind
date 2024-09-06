use exocore_protos::{
    apps::{in_message::InMessageType, InMessage, MessageStatus},
    prost::Message,
};

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "exocore")]
extern "C" {
    pub(crate) fn __exocore_host_log(level: u8, bytes: *const u8, len: usize);
    pub(crate) fn __exocore_host_now() -> u64;
    pub(crate) fn __exocore_host_out_message(bytes: *const u8, len: usize) -> u32;
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) unsafe fn __exocore_host_log(_level: u8, _bytes: *const u8, _len: usize) {
    panic!("Not implemented in outside of wasm environment");
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) unsafe fn __exocore_host_now() -> u64 {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) unsafe fn __exocore_host_out_message(_bytes: *const u8, _len: usize) -> u32 {
    panic!("Not implemented in outside of wasm environment");
}

/* Added by the macro
#[no_mangle]
pub extern "C" fn __exocore_app_init() {}
*/

#[no_mangle]
pub extern "C" fn __exocore_init() {
    crate::logging::init().expect("Couldn't setup logging");

    let exocore = crate::client::Exocore::get();
    exocore.store.start();
}

/// Ticks timer, executor and returns the next timestamp at which we should
/// minimally be polled again.
#[no_mangle]
pub extern "C" fn __exocore_tick() -> u64 {
    crate::time::poll_timers();
    crate::executor::poll_executor();

    // returns time at which we want to be tick
    crate::time::next_timer_time()
        .map(|t| -> u64 { t.into() })
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __exocore_app_boot() {
    crate::app::boot_app();
}

#[no_mangle]
pub extern "C" fn __exocore_in_message(ptr: *const u8, size: usize) -> u32 {
    let exomind = crate::client::Exocore::get();

    let bytes = unsafe { std::slice::from_raw_parts(ptr, size) };
    let msg = match InMessage::decode(bytes) {
        Ok(msg) => msg,
        Err(err) => {
            error!("Couldn't decode incoming message: {}", err);
            return MessageStatus::DecodeError as u32;
        }
    };

    let res = match InMessageType::try_from(msg.r#type) {
        Ok(InMessageType::StoreEntityResults) => exomind.store.handle_query_results(msg),
        Ok(InMessageType::StoreMutationResult) => exomind.store.handle_mutation_result(msg),
        Ok(InMessageType::Invalid) => {
            error!("Received an invalid message type: {}", msg.r#type);
            return MessageStatus::Unhandled as u32;
        }
        Err(err) => {
            error!(
                "Received an invalid message type: {}, err: {err}",
                msg.r#type
            );
            return MessageStatus::Unhandled as u32;
        }
    };

    if let Err(err) = res {
        return err as u32;
    }

    MessageStatus::Ok as u32
}

#[no_mangle]
pub unsafe extern "C" fn __exocore_alloc(size: usize) -> *mut u8 {
    let align = std::mem::align_of::<usize>();
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    std::alloc::alloc(layout)
}

#[no_mangle]
pub unsafe extern "C" fn __exocore_free(ptr: *mut u8, size: usize) {
    let align = std::mem::align_of::<usize>();
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    std::alloc::dealloc(ptr, layout);
}
