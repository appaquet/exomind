use std::{path::Path, sync::Arc, time::Duration};

use exocore_protos::{
    apps::{InMessage, MessageStatus, OutMessage},
    prost::{self, Message},
};
use log::Level;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

use crate::error::Error;

type FuncSendMessage = TypedFunc<(i32, i32), u32>;
type FuncTick = TypedFunc<(), u64>;

/// Runtime for an application WASM module.
pub struct WasmTimeRuntime<E: HostEnvironment> {
    instance: Instance,
    send_message_func: FuncSendMessage,
    tick_func: FuncTick,
    store: Store<WasiCtx>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: HostEnvironment> WasmTimeRuntime<E> {
    pub fn from_file<P>(file: P, env: Arc<E>) -> Result<WasmTimeRuntime<E>, Error>
    where
        P: AsRef<Path>,
    {
        let engine = Engine::default();

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        Self::setup_host_module(&mut linker, &env)?;

        let module = Module::from_file(&engine, file)?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stderr()
            .inherit_stdout()
            .build();

        let mut store = Store::new(&engine, wasi);

        let instance = linker.instantiate(&mut store, &module)?;
        let (tick_func, send_message_func) = bootstrap_instance(&mut store, &instance)?;

        Ok(WasmTimeRuntime {
            instance,
            send_message_func,
            tick_func,
            store,
            _phantom: std::marker::PhantomData,
        })
    }

    // Runs an iteration on the WASM module.
    pub fn tick(&mut self) -> Result<Option<Duration>, Error> {
        let now = unix_timestamp();
        let next_tick_time = self.tick_func.call(&mut self.store, ())?;

        if next_tick_time > now {
            Ok(Some(Duration::from_nanos(next_tick_time - now)))
        } else {
            Ok(None)
        }
    }

    // Send a message to the WASM module.
    pub fn send_message(&mut self, message: InMessage) -> Result<(), Error> {
        let message_bytes = message.encode_to_vec();

        let (message_ptr, message_size) =
            wasm_alloc(&mut self.store, &self.instance, &message_bytes)?;
        let res = self
            .send_message_func
            .call(&mut self.store, (message_ptr, message_size));
        wasm_free(&mut self.store, &self.instance, message_ptr, message_size)?;

        match MessageStatus::try_from(res? as i32) {
            Ok(MessageStatus::Ok) => {}
            other => return Err(Error::MessageStatus(other.ok())),
        }

        Ok(())
    }

    fn setup_host_module(linker: &mut Linker<WasiCtx>, env: &Arc<E>) -> Result<(), Error> {
        let env_clone = env.clone();

        linker.func_wrap(
            "exocore",
            "__exocore_host_log",
            move |mut caller: Caller<'_, WasiCtx>, level: i32, ptr: i32, len: i32| {
                let log_level = log_level_from_i32(level);
                read_wasm_str(&mut caller, ptr, len, |msg| {
                    env_clone.handle_log(log_level, msg);
                })?;

                Ok(())
            },
        )?;

        linker.func_wrap(
            "exocore",
            "__exocore_host_now",
            |_caller: Caller<'_, WasiCtx>| -> u64 { unix_timestamp() },
        )?;

        let env = env.clone();
        linker.func_wrap(
            "exocore",
            "__exocore_host_out_message",
            move |mut caller: Caller<'_, WasiCtx>, ptr: i32, len: i32| -> u32 {
                let status = match read_wasm_message::<OutMessage>(&mut caller, ptr, len) {
                    Ok(msg) => {
                        env.as_ref().handle_message(msg);
                        MessageStatus::Ok
                    }
                    Err(err) => {
                        error!("Couldn't decode message sent from application: {}", err);
                        MessageStatus::DecodeError
                    }
                };

                status as u32
            },
        )?;

        Ok(())
    }
}

fn log_level_from_i32(level: i32) -> Level {
    match level {
        1 => Level::Error,
        2 => Level::Warn,
        3 => Level::Info,
        4 => Level::Debug,
        5 => Level::Trace,
        _ => Level::Error,
    }
}

/// Environment to which messages and logs from the WASM application are sent.
pub trait HostEnvironment: Send + Sync + 'static {
    fn handle_message(&self, msg: OutMessage);
    fn handle_log(&self, level: log::Level, msg: &str);
}

fn bootstrap_instance(
    mut store: &mut Store<WasiCtx>,
    instance: &Instance,
) -> Result<(FuncTick, FuncSendMessage), Error> {
    // Initialize environment
    let exocore_init = instance
        .get_typed_func::<(), ()>(&mut store, "__exocore_init")
        .map_err(|err| Error::MissingFunction(err, "__exocore_init"))?;
    exocore_init.call(&mut store, ())?;

    // Create application instance
    let exocore_app_init = instance
        .get_typed_func::<(), ()>(&mut store, "__exocore_app_init")
        .map_err(|err| Error::MissingFunction(err, "__exocore_app_init"))?;
    exocore_app_init.call(&mut store, ())?;

    // Boot the application
    let exocore_app_boot = instance
        .get_typed_func::<(), ()>(&mut store, "__exocore_app_boot")
        .map_err(|err| Error::MissingFunction(err, "__exocore_app_boot"))?;
    exocore_app_boot.call(&mut store, ())?;

    // Extract tick & message sending functions
    let exocore_tick = instance
        .get_typed_func::<(), u64>(&mut store, "__exocore_tick")
        .map_err(|err| Error::MissingFunction(err, "__exocore_tick"))?;
    let exocore_send_message = instance
        .get_typed_func::<(i32, i32), u32>(&mut store, "__exocore_in_message")
        .map_err(|err| Error::MissingFunction(err, "__exocore_in_message"))?;

    Ok((exocore_tick, exocore_send_message))
}

/// Reads a bytes from a wasm pointer and len.
///
/// Mostly copied from wasmtime::Func comments.
fn read_wasm_message<M: prost::Message + Default>(
    caller: &mut Caller<'_, WasiCtx>,
    ptr: i32,
    len: i32,
) -> Result<M, Error> {
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(Error::Runtime("failed to find host memory")),
    };

    let data = mem
        .data(caller)
        .get(ptr as u32 as usize..)
        .and_then(|arr| arr.get(..len as u32 as usize));

    match data {
        Some(data) => Ok(M::decode(data)?),
        None => Err(Error::Runtime("pointer/length out of bounds")),
    }
}

/// Reads a str from a wasm pointer and len.
///
/// Mostly copied from wasmtime::Func comments.
fn read_wasm_str<F: FnOnce(&str)>(
    caller: &mut Caller<'_, WasiCtx>,
    ptr: i32,
    len: i32,
    f: F,
) -> Result<(), Error> {
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(Error::Runtime("failed to find host memory")),
    };

    let data = mem
        .data(caller)
        .get(ptr as u32 as usize..)
        .and_then(|arr| arr.get(..len as u32 as usize));
    match data {
        Some(data) => match std::str::from_utf8(data) {
            Ok(s) => f(s),
            Err(_) => return Err(Error::Runtime("invalid utf-8")),
        },
        None => return Err(Error::Runtime("pointer/length out of bounds")),
    };

    Ok(())
}

// Inspired from https://radu-matei.com/blog/practical-guide-to-wasm-memory/#passing-arrays-to-modules-using-wasmtime
fn wasm_alloc(
    mut store: &mut Store<WasiCtx>,
    instance: &Instance,
    bytes: &[u8],
) -> Result<(i32, i32), Error> {
    let mem = match instance.get_export(&mut store, "memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(Error::Runtime("failed to find host memory")),
    };

    let alloc = instance
        .get_typed_func::<i32, i32>(&mut store, "__exocore_alloc")
        .map_err(|err| Error::MissingFunction(err, "__exocore_alloc"))?;
    let ptr = alloc.call(&mut store, bytes.len() as i32)?;

    let data = mem.data_mut(store);
    let ptr_usize = ptr as usize;
    data[ptr_usize..ptr_usize + bytes.len()].copy_from_slice(bytes);

    Ok((ptr, bytes.len() as i32))
}

fn wasm_free(
    mut store: &mut Store<WasiCtx>,
    instance: &Instance,
    ptr: i32,
    size: i32,
) -> Result<(), Error> {
    let alloc = instance
        .get_typed_func::<(i32, i32), ()>(&mut store, "__exocore_free")
        .map_err(|err| Error::MissingFunction(err, "__exocore_free"))?;
    alloc.call(&mut store, (ptr, size))?;

    Ok(())
}

fn unix_timestamp() -> u64 {
    // TODO: Should be consistent timestamp
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use std::{sync::Mutex, thread::sleep};

    use exocore_core::tests_utils::find_test_fixture;
    use exocore_protos::{
        apps::{in_message::InMessageType, out_message::OutMessageType},
        store::{EntityResults, MutationResult},
    };

    use super::*;

    /// Runs the application defined in `exocore-apps-example`. See its `lib.rs`
    /// to follow the sequence.
    #[test]
    fn example_golden_path() {
        let example_path = find_test_fixture("fixtures/example.wasm");
        let env = Arc::new(TestEnv::new());

        let mut app = WasmTimeRuntime::from_file(example_path, env.clone()).unwrap();

        // first tick should execute up to sleep
        app.tick().unwrap();
        assert!(env.find_log("application initialized").is_some());
        assert!(env.find_log("task starting").is_some());

        // should now be sleeping for 100ms
        let last_log = env.last_log().unwrap();
        assert!(last_log.contains("before sleep"));
        let time_before_sleep = last_log
            .replace("before sleep ", "")
            .parse::<u64>()
            .unwrap();

        // ticking right away shouldn't do anything since app is sleeping for 100ms
        let next_tick_duration = app
            .tick()
            .unwrap()
            .unwrap_or_else(|| Duration::from_nanos(0));
        let last_log = env.last_log().unwrap();
        assert!(last_log.contains("before sleep"));

        // wait for next tick duration
        assert!(next_tick_duration > Duration::from_millis(10));
        sleep(next_tick_duration);

        // ticking after sleep time should now wake up and continue
        app.tick().unwrap();
        let after_sleep_log = env.find_log("after sleep").unwrap();
        let time_after_sleep = after_sleep_log
            .replace("after sleep ", "")
            .parse::<u64>()
            .unwrap();
        assert!((time_after_sleep - time_before_sleep) > 100_000_000); // 100ms in nano

        // should have sent mutation request to host
        let message = env.pop_message().unwrap();
        assert_eq!(message.r#type, OutMessageType::StoreMutationRequest as i32);

        // reply with mutation result, should report that mutation has succeed
        app.send_message(InMessage {
            r#type: InMessageType::StoreMutationResult.into(),
            rendez_vous_id: message.rendez_vous_id,
            data: MutationResult::default().encode_to_vec(),
            error: String::new(),
        })
        .unwrap();
        app.tick().unwrap();
        assert!(env.find_log("mutation success").is_some());

        // should have sent query to host
        let message = env.pop_message().unwrap();
        assert_eq!(message.r#type, OutMessageType::StoreEntityQuery as i32);

        // reply with query result, should report that query has succeed
        app.send_message(InMessage {
            r#type: InMessageType::StoreEntityResults.into(),
            rendez_vous_id: message.rendez_vous_id,
            data: EntityResults::default().encode_to_vec(),
            error: String::new(),
        })
        .unwrap();
        app.tick().unwrap();
        assert!(env.find_log("query success").is_some());

        // should have completed
        assert_eq!(env.last_log(), Some("task done".to_string()));
    }

    struct TestEnv {
        logs: Mutex<Vec<String>>,
        messages: Mutex<Vec<OutMessage>>,
    }

    impl TestEnv {
        fn new() -> TestEnv {
            TestEnv {
                logs: Mutex::new(Vec::new()),
                messages: Mutex::new(Vec::new()),
            }
        }

        fn find_log(&self, needle: &str) -> Option<String> {
            let logs = self.logs.lock().unwrap();
            for log in logs.iter() {
                if log.contains(needle) {
                    return Some(log.clone());
                }
            }

            None
        }

        fn last_log(&self) -> Option<String> {
            let logs = self.logs.lock().unwrap();
            logs.last().cloned()
        }

        fn pop_message(&self) -> Option<OutMessage> {
            let mut msgs = self.messages.lock().unwrap();
            msgs.pop()
        }
    }

    impl HostEnvironment for TestEnv {
        fn handle_message(&self, msg: OutMessage) {
            let mut messages = self.messages.lock().unwrap();
            messages.push(msg);
        }

        fn handle_log(&self, level: log::Level, msg: &str) {
            log!(level, "WASM APP: {}", msg);
            let mut logs = self.logs.lock().unwrap();
            logs.push(msg.to_string());
        }
    }
}
