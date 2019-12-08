use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use futures::prelude::*;
use futures::sync::mpsc;
use futures::MapErr;
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_timer::Instant;
use web_sys::{BinaryType, ErrorEvent, MessageEvent, WebSocket};

use exocore_common::cell::{Cell, CellId};
use exocore_common::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_common::node::Node;
use exocore_common::protos::common_capnp::envelope;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_common::utils::futures::spawn_future_non_send;
use exocore_transport::transport::{ConnectionStatus, MpscHandleSink, MpscHandleStream};
use exocore_transport::{Error, InEvent, InMessage, OutEvent, TransportHandle, TransportLayer};

const OUT_CHANNEL_SIZE: usize = 100;
const IN_CHANNEL_SIZE: usize = 100;

///
/// Client for `exocore_transport` WebSocket transport for browser
///
pub struct BrowserTransportClient {
    start_notifier: CompletionNotifier<(), Error>,
    inner: Arc<Mutex<Inner>>,
}

impl BrowserTransportClient {
    pub fn new(url: &str, remote_node: Node) -> BrowserTransportClient {
        let start_notifier = CompletionNotifier::new();
        let stop_notifier = CompletionNotifier::new();

        let inner = Arc::new(Mutex::new(Inner {
            remote_node,
            url: url.to_owned(),
            socket: None,
            connection_status: ConnectionStatus::Disconnected,
            last_connect_attempt: None,
            handles: HashMap::new(),
            stop_notifier,
        }));

        BrowserTransportClient {
            start_notifier,
            inner,
        }
    }

    pub fn get_handle(&mut self, cell: Cell, layer: TransportLayer) -> BrowserTransportHandle {
        let mut inner = self.inner.lock().expect("Couldn't get inner lock");

        let (out_sender, out_receiver) = mpsc::channel(OUT_CHANNEL_SIZE);
        let (in_sender, in_receiver) = mpsc::channel(IN_CHANNEL_SIZE);

        let inner_handle = InnerHandle {
            out_receiver: Some(out_receiver),
            in_sender,
        };
        inner
            .handles
            .insert((cell.id().clone(), layer), inner_handle);

        let start_listener = self
            .start_notifier
            .get_listener()
            .expect("Couldn't get a listener on start notifier");
        let stop_listener = inner
            .stop_notifier
            .get_listener()
            .expect("Couldn't get a listener on stop notifier");

        BrowserTransportHandle {
            start_listener,
            in_receiver: Some(MpscHandleStream::new(in_receiver)),
            out_sender: Some(MpscHandleSink::new(out_sender)),
            stop_listener,
        }
    }

    pub fn start(&mut self) {
        let mut inner = self.inner.lock().expect("Couldn't get inner lock");

        // get stream for outgoing messages from each handle, and pipe it to websocket
        for ((_cell_id, _layer), ref mut inner_handle) in &mut inner.handles {
            let weak_inner = Arc::downgrade(&self.inner);
            let out_receiver = inner_handle
                .out_receiver
                .take()
                .expect("InnerHandle's out_receiver was already consumed");

            spawn_future_non_send(
                out_receiver
                    .map_err(|_err| Error::Other("Handle out stream error err".to_string()))
                    .for_each(move |event| {
                        match event {
                            OutEvent::Message(msg) => {
                                let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
                                let inner = inner.lock()?;
                                if let Some(ws) = &inner.socket {
                                    let mut message_bytes = msg.envelope_builder.as_bytes();
                                    if let Err(_err) = ws.ws.send_with_u8_array(&mut message_bytes)
                                    {
                                        error!("Error sending a message to websocket");
                                    }
                                }
                            }
                        }

                        Ok(())
                    })
                    .map_err(|err| {
                        error!("Got an error in out stream of handle: {}", err);
                    }),
            );
        }

        // start websocket
        let weak_inner = Arc::downgrade(&self.inner);
        inner.create_websocket(&weak_inner);

        // start management timer
        let weak_inner = Arc::downgrade(&self.inner);
        let check_connections = wasm_timer::Interval::new_interval(Duration::from_millis(1000))
            .map_err(|err| Error::Other(format!("Timer error: {}", err)))
            .for_each(move |_| {
                let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
                let mut inner = inner.lock()?;
                inner.check_connection_status(&weak_inner)?;
                Ok(())
            })
            .map_err(|err| {
                error!("Error in timer: {}", err.to_string());
            });
        spawn_future_non_send(check_connections);
    }
}

///
/// Inner instance of the transport
///
struct Inner {
    remote_node: Node,
    url: String,
    socket: Option<WSInstance>,
    connection_status: ConnectionStatus,
    last_connect_attempt: Option<Instant>,
    handles: HashMap<(CellId, TransportLayer), InnerHandle>,
    stop_notifier: CompletionNotifier<(), Error>,
}

struct InnerHandle {
    out_receiver: Option<mpsc::Receiver<OutEvent>>,
    in_sender: mpsc::Sender<InEvent>,
}

struct WSInstance {
    ws: WebSocket,
    _onmessage_callback: Closure<dyn FnMut(MessageEvent)>,
    _onerror_callback: Closure<dyn FnMut(ErrorEvent)>,
    _onopen_callback: Closure<dyn FnMut(JsValue)>,
}

impl Inner {
    fn create_websocket(&mut self, weak_self: &Weak<Mutex<Inner>>) {
        // See https://rustwasm.github.io/docs/wasm-bindgen/examples/websockets.html
        // See https://docs.rs/web-sys/0.3.28/web_sys/struct.WebSocket.html#method.send_with_u8_array

        let ws = WebSocket::new_with_str(&self.url, "exocore_websocket").unwrap();
        ws.set_binary_type(BinaryType::Arraybuffer);

        let remote_node = self.remote_node.clone();
        let weak_self1 = weak_self.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            let bin_data = ArrayBuffer::from(e.data());
            let uint8_data = Uint8Array::new(&bin_data);
            let mut bytes = vec![0; uint8_data.length() as usize];
            uint8_data.copy_to(&mut bytes);

            let frame = TypedCapnpFrame::<_, envelope::Owned>::new(bytes.as_slice()).unwrap();
            let in_message = InMessage::from_node_and_frame(remote_node.clone(), frame);

            match in_message {
                Ok(msg) => {
                    if let Err(err) = Self::handle_incoming_message(&weak_self1, msg) {
                        error!("Error handling incoming message: {}", err);
                    }
                }
                Err(err) => {
                    error!("Error parsing incoming message: {}", err);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        let weak_self2 = weak_self.clone();
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            error!("Error in WebSocket: {:?}", e);
            let _ = Self::check_connection_status_weak(&weak_self2);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));

        let weak_self3 = weak_self.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            info!("WebSocket opened");
            let _ = Self::check_connection_status_weak(&weak_self3);
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));

        self.last_connect_attempt = Some(Instant::now());
        self.socket = Some(WSInstance {
            ws,
            _onmessage_callback: onmessage_callback,
            _onerror_callback: onerror_callback,
            _onopen_callback: onopen_callback,
        })
    }

    fn check_connection_status_weak(weak_self: &Weak<Mutex<Inner>>) -> Result<(), Error> {
        let inner = weak_self.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.lock()?;
        inner.check_connection_status(weak_self)
    }

    fn check_connection_status(&mut self, weak_self: &Weak<Mutex<Inner>>) -> Result<(), Error> {
        let status_before = self.connection_status;

        if let Some(ws) = &self.socket {
            // see states here https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState
            match ws.ws.ready_state() {
                0 /*connecting*/ => {
                    self.connection_status = ConnectionStatus::Connecting;
                }
                1 /*open*/ => {
                    self.connection_status = ConnectionStatus::Connected;
                }
                socket_status => {
                    self.connection_status = ConnectionStatus::Disconnected;

                    let should_reconnect = self
                        .last_connect_attempt
                        .map_or(true, |attempt| attempt.elapsed() > Duration::from_secs(5));

                    if should_reconnect {
                        let last_connect_elapsed = self.last_connect_attempt.map(|i| i.elapsed());
                        info!("WebSocket was not connected. Reconnecting... (socket_status={:?} last_connect_attempt={:?})", socket_status, last_connect_elapsed);
                        self.create_websocket(weak_self);
                    }
                }
            }
        } else {
            warn!("No WebSocket was started. Reconnecting...");
            self.connection_status = ConnectionStatus::Disconnected;
            self.create_websocket(weak_self);
        }

        if status_before != self.connection_status {
            info!(
                "Dispatching new status: before={:?} after={:?}",
                status_before, self.connection_status
            );
            let _ = self.dispatch_node_status();
        }

        Ok(())
    }

    fn handle_incoming_message(
        weak_self: &Weak<Mutex<Inner>>,
        in_message: Box<InMessage>,
    ) -> Result<(), Error> {
        let inner = weak_self.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.lock()?;

        let key = (in_message.cell_id.clone(), in_message.layer);
        let handle = inner
            .handles
            .get_mut(&key)
            .ok_or_else(|| Error::Other(format!("Got message for unknown handle: {:?}", key)))?;

        handle
            .in_sender
            .try_send(InEvent::Message(in_message))
            .map_err(|err| {
                Error::Other(format!("Got error sending to handle {:?}: {}", key, err))
            })?;

        Ok(())
    }

    fn dispatch_node_status(&mut self) -> Result<(), Error> {
        for (key, handle) in self.handles.iter_mut() {
            let event = InEvent::NodeStatus(self.remote_node.id().clone(), self.connection_status);
            handle.in_sender.try_send(event).map_err(|err| {
                Error::Other(format!("Got error sending to handle {:?}: {}", key, err))
            })?;
        }

        Ok(())
    }
}

///
/// Handle to the websocket transport
///
pub struct BrowserTransportHandle {
    start_listener: CompletionListener<(), Error>,
    in_receiver: Option<MpscHandleStream>,
    out_sender: Option<MpscHandleSink>,
    stop_listener: CompletionListener<(), Error>,
}

impl Future for BrowserTransportHandle {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.stop_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            _ => Error::Other("Error in completion error".to_string()),
        })
    }
}

type StartFutureType = MapErr<CompletionListener<(), Error>, fn(CompletionError<Error>) -> Error>;

impl TransportHandle for BrowserTransportHandle {
    type StartFuture = StartFutureType;
    type Sink = MpscHandleSink;
    type Stream = MpscHandleStream;

    fn on_start(&self) -> Self::StartFuture {
        self.start_listener
            .try_clone()
            .expect("Couldn't clone start listener")
            .map_err(|err| match err {
                CompletionError::UserError(err) => err,
                _ => Error::Other("Error in completion error".to_string()),
            })
    }

    fn get_sink(&mut self) -> Self::Sink {
        self.out_sender
            .take()
            .expect("OutMessage sink was already taken")
    }

    fn get_stream(&mut self) -> Self::Stream {
        self.in_receiver
            .take()
            .expect("InMessage stream was already taken")
    }
}
