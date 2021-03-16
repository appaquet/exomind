use std::{fs::File, io::Write, path::PathBuf, sync::Arc, time::Duration};

use anyhow::anyhow;
use exocore_core::{
    cell::Cell,
    futures::{block_on, owned_spawn, sleep, spawn_blocking, spawn_future, BatchingStream},
    sec::hash::{multihash_decode_bs58, MultihashDigestExt, MultihashExt, Sha3_256},
    time::Clock,
    utils::backoff::BackoffCalculator,
};
use exocore_protos::{
    apps::{in_message::InMessageType, out_message::OutMessageType, InMessage, OutMessage},
    prost::{Message, ProstMessageExt},
    store::{EntityMutation, EntityQuery},
};
use exocore_store::store::Store;
use futures::{
    channel::mpsc,
    future::{pending, select_all, FutureExt},
    lock::Mutex,
    Future, SinkExt, StreamExt,
};

use crate::{runtime::AppRuntime, Config, Error};

const MSG_BUFFER_SIZE: usize = 5000;
const RUNTIME_MSG_BATCH_SIZE: usize = 1000;
const APP_MIN_TICK_TIME: Duration = Duration::from_millis(100);

/// Exocore applications host.
///
/// Executes applications that have a WASM module in a background thread per
/// applications and handles incoming and outgoing messages to the module for
/// store and communication.
pub struct Applications<S: Store> {
    config: Config,
    cell: Cell,
    clock: Clock,
    store: S,
    apps: Vec<Application>,
}

impl<S: Store> Applications<S> {
    pub async fn new(
        config: Config,
        clock: Clock,
        cell: Cell,
        store: S,
    ) -> Result<Applications<S>, Error> {
        let apps_directory = cell
            .apps_directory()
            .ok_or_else(|| anyhow!("Cell {}: No apps directory configured", cell))?;

        let mut apps = Vec::new();
        for app in cell.applications().applications() {
            let cell_app = app.application();
            let app_manifest = cell_app.manifest();

            let module_manifest = if let Some(module) = &app_manifest.module {
                module.clone()
            } else {
                continue;
            };

            let app = Application {
                cell: cell.clone(),
                cell_app: cell_app.clone(),
                module_manifest,
                module_path: apps_directory.join(format!(
                    "{}_{}/module.wasm",
                    app_manifest.name, app_manifest.version
                )),
            };
            app.ensure_downloaded().await?;

            apps.push(app);
        }

        Ok(Applications {
            config,
            cell,
            clock,
            store,
            apps,
        })
    }

    /// Starts and runs applications.
    pub async fn run(self) -> Result<(), Error> {
        if self.apps.is_empty() {
            info!("{}: No apps to start. Blocking forever.", self.cell);
            pending::<()>().await;
            return Ok(());
        }

        let mut spawned_apps = Vec::new();
        for app in self.apps {
            spawned_apps.push(owned_spawn(Self::start_app_loop(
                self.clock.clone(),
                self.config,
                app,
                self.store.clone(),
            )));
        }

        // wait for any applications to terminate
        let _ = select_all(spawned_apps).await;

        Ok(())
    }

    async fn start_app_loop(clock: Clock, config: Config, app: Application, store: S) {
        let mut backoff = BackoffCalculator::new(clock, config.restart_backoff);
        loop {
            info!("{}: Starting application", app);

            let store = store.clone();
            Self::start_app(&app, store).await;

            backoff.increment_failure();

            let restart_delay = backoff.backoff_duration();
            error!(
                "{}: Application has quit. Restarting in {:?}...",
                app, restart_delay
            );
            sleep(restart_delay).await;
        }
    }

    async fn start_app(app: &Application, store: S) {
        let (in_sender, in_receiver) = mpsc::channel(MSG_BUFFER_SIZE);
        let (out_sender, mut out_receiver) = mpsc::channel(MSG_BUFFER_SIZE);

        // Spawn the application module runtime on a separate thread.
        let runtime_spawn = {
            let env = Arc::new(WiredEnvironment {
                log_prefix: app.to_string(),
                sender: std::sync::Mutex::new(out_sender),
            });

            let app_module_path = app.module_path.clone();
            let app_prefix = app.to_string();
            spawn_blocking(move || -> Result<(), Error> {
                let app_runtime = AppRuntime::from_file(app_module_path, env)?;
                let mut batch_receiver = BatchingStream::new(in_receiver, RUNTIME_MSG_BATCH_SIZE);

                let mut next_tick = sleep(APP_MIN_TICK_TIME);
                loop {
                    let in_messages: Option<Vec<InMessage>> = block_on(async {
                        futures::select! {
                            _ = next_tick.fuse() => Some(vec![]),
                            msgs = batch_receiver.next().fuse() => msgs,
                        }
                    });

                    let in_messages = if let Some(in_messages) = in_messages {
                        in_messages
                    } else {
                        info!(
                            "{}: In message receiver returned none. Stopping app runtime",
                            app_prefix
                        );
                        return Ok(());
                    };

                    for in_message in in_messages {
                        app_runtime.send_message(in_message)?;
                    }

                    let next_tick_duration = app_runtime.tick()?.unwrap_or(APP_MIN_TICK_TIME);
                    next_tick = sleep(next_tick_duration);
                }
            })
        };

        // Spawn a task to handle store requests coming from the application
        let store_worker = {
            let store = store.clone();
            let app_prefix = app.to_string();
            async move {
                let in_sender = Arc::new(Mutex::new(in_sender));
                while let Some(message) = out_receiver.next().await {
                    match OutMessageType::from_i32(message.r#type) {
                        Some(OutMessageType::StoreEntityQuery) => {
                            let store = store.clone();
                            handle_store_message(
                                message.rendez_vous_id,
                                InMessageType::StoreEntityResults,
                                in_sender.clone(),
                                move || handle_entity_query(message, store),
                            )
                        }
                        Some(OutMessageType::StoreMutationRequest) => {
                            let store = store.clone();
                            handle_store_message(
                                message.rendez_vous_id,
                                InMessageType::StoreMutationResult,
                                in_sender.clone(),
                                move || handle_entity_mutation(message, store),
                            )
                        }
                        other => {
                            error!(
                                "{}: Got an unknown message type {:?} with id {}",
                                app_prefix, other, message.r#type
                            );
                        }
                    }
                }

                Ok::<(), Error>(())
            }
        };

        futures::select! {
            res = runtime_spawn.fuse() => {
                info!("{}: App runtime spawn has stopped: {:?}", app, res);
            }
            _ = store_worker.fuse() => {
                info!("{}: Store worker task has stopped", app);
            }
        };
    }
}

fn handle_store_message<F, O>(
    rendez_vous_id: u32,
    reply_type: InMessageType,
    in_sender: Arc<Mutex<mpsc::Sender<InMessage>>>,
    func: F,
) where
    F: (FnOnce() -> O) + Send + 'static,
    O: Future<Output = Result<Vec<u8>, Error>> + Send + 'static,
{
    spawn_future(async move {
        let mut msg = InMessage {
            r#type: reply_type.into(),
            rendez_vous_id,
            ..Default::default()
        };

        let res = func();
        match res.await {
            Ok(res) => msg.data = res,
            Err(err) => msg.error = err.to_string(),
        }

        let mut in_sender = in_sender.lock().await;
        let _ = in_sender.send(msg).await;
    });
}

async fn handle_entity_query<S: Store>(
    out_message: OutMessage,
    store: S,
) -> Result<Vec<u8>, Error> {
    let query = EntityQuery::decode(out_message.data.as_ref())?;
    let res = store.query(query);
    let res = res.await?;

    Ok(res.encode_to_vec())
}

async fn handle_entity_mutation<S: Store>(
    out_message: OutMessage,
    store: S,
) -> Result<Vec<u8>, Error> {
    let mutation = EntityMutation::decode(out_message.data.as_ref())?;
    let res = store.mutate(mutation);
    let res = res.await?;

    Ok(res.encode_to_vec())
}

struct WiredEnvironment {
    log_prefix: String,
    sender: std::sync::Mutex<mpsc::Sender<exocore_protos::apps::OutMessage>>,
}

impl crate::runtime::HostEnvironment for WiredEnvironment {
    fn handle_message(&self, msg: exocore_protos::apps::OutMessage) {
        let mut sender = self.sender.lock().unwrap();
        if let Err(err) = sender.try_send(msg) {
            error!("Couldn't send message via channel: {}", err)
        }
    }

    fn handle_log(&self, level: log::Level, msg: &str) {
        log!(level, "{}: WASM - {}", self.log_prefix, msg);
    }
}

struct Application {
    cell: Cell,
    cell_app: exocore_core::cell::Application,
    module_manifest: exocore_protos::apps::ManifestModule,
    module_path: PathBuf,
}

impl Application {
    async fn ensure_downloaded(&self) -> Result<(), Error> {
        if self.is_module_downloaded() {
            return Ok(());
        }

        if let Some(parent) = self.module_path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                anyhow!(
                    "{}: Couldn't create module directory '{:?}': {}",
                    self,
                    parent,
                    err
                )
            })?;
        }

        if self.module_manifest.url.starts_with("file://") {
            let source_path = self.module_manifest.url.strip_prefix("file://").unwrap();
            std::fs::copy(source_path, &self.module_path).map_err(|err| {
                anyhow!(
                    "{}: Couldn't copy app module from '{:?}' to path '{:?}': {}",
                    self,
                    source_path,
                    self.module_path,
                    err
                )
            })?;
        } else {
            let body = reqwest::get(&self.module_manifest.url)
                .await
                .map_err(|err| {
                    anyhow!(
                        "{}: Couldn't download app module from {}: {}",
                        self,
                        self.module_manifest.url,
                        err
                    )
                })?
                .bytes()
                .await
                .map_err(|err| {
                    anyhow!(
                        "{}: Couldn't download app module from {}: {}",
                        self,
                        self.module_manifest.url,
                        err
                    )
                })?;

            let mut file = File::create(&self.module_path)
                .map_err(|err| anyhow!("Couldn't create module app file: {}", err))?;
            file.write_all(body.as_ref())
                .map_err(|err| anyhow!("Couldn't write app file: {}", err))?;
        }

        if !self.is_module_downloaded() {
            return Err(anyhow!(
                "{}: Module file not downloaded or not matching multihash",
                self
            )
            .into());
        }

        Ok(())
    }

    fn is_module_downloaded(&self) -> bool {
        let module_exists = std::fs::metadata(&self.module_path).is_ok();
        if !module_exists {
            debug!("{}: Module doesn't exist at {:?}", self, self.module_path);
            return false;
        }

        let file = match File::open(&self.module_path) {
            Ok(file) => file,
            Err(err) => {
                debug!(
                    "{}: Couldn't open module from disk at {:?}: {}",
                    self, self.module_path, err
                );
                return false;
            }
        };

        let expected_multihash = match multihash_decode_bs58(&self.module_manifest.multihash) {
            Ok(mh) => mh,
            Err(err) => {
                warn!(
                    "{}: Couldn't decode expected module multihash in manifest: {}",
                    self, err
                );
                return false;
            }
        };

        let mut hasher = Sha3_256::default();
        match hasher.update_from_reader(file) {
            Ok(mh) if mh == expected_multihash => true,
            Ok(mh) => {
                let mh_bs58 = mh.encode_bs58();

                warn!(
                    "{}: Module multihash in manifest doesn't match module file (expected={} module={})",
                    self,
                    self.module_manifest.multihash,
                    mh_bs58,
                );
                false
            }
            Err(err) => {
                warn!(
                    "{}: Couldn't compute multihash of {:?}: {}",
                    self, self.module_path, err
                );
                false
            }
        }
    }
}

impl std::fmt::Display for Application {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} App{{{}}}", self.cell, self.cell_app.name())
    }
}
