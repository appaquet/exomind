use exocore_common::cell::{Cell, CellId};
use exocore_common::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_common::node::Node;
use exocore_common::protos::common_capnp::envelope;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_transport::transport::{MpscHandleSink, MpscHandleStream};
use exocore_transport::{Error, InMessage, OutMessage, TransportHandle, TransportLayer};
use futures::prelude::*;
use futures::sync::mpsc;
use futures::MapErr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use stdweb::traits::{IEventTarget, IMessageEvent};
use stdweb::web::event::{SocketMessageEvent, SocketOpenEvent};
use stdweb::web::{SocketBinaryType, SocketReadyState, WebSocket};
use wasm_timer::Instant;

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

        let (out_sender, out_receiver) = mpsc::channel(1000); // TODO: Config
        let (in_sender, in_receiver) = mpsc::channel(1000); // TODO: Config

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
            .expect("Couldn't get a listener on start notifier");

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
            wasm_bindgen_futures::spawn_local(
                out_receiver
                    .map_err(|_err| Error::Other("Handle out stream error err".to_string()))
                    .for_each(move |out_message| {
                        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
                        let inner = inner.lock()?;
                        if let Some(ws) = &inner.socket {
                            let message_bytes = out_message.envelope_builder.as_bytes();
                            if let Err(_err) = ws.send_bytes(&message_bytes) {
                                error!("Error sending a message to websocket");
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
                inner.check_should_reconnect(&weak_inner)?;
                Ok(())
            })
            .map_err(|err| {
                error!("Error in timer: {}", err.to_string());
            });
        wasm_bindgen_futures::spawn_local(check_connections);
    }
}

///
/// Inner instance of the transport
///
struct Inner {
    remote_node: Node,
    url: String,
    socket: Option<WebSocket>,
    last_connect_attempt: Option<Instant>,
    handles: HashMap<(CellId, TransportLayer), InnerHandle>,
    stop_notifier: CompletionNotifier<(), Error>,
}

struct InnerHandle {
    out_receiver: Option<mpsc::Receiver<OutMessage>>,
    in_sender: mpsc::Sender<InMessage>,
}

impl Inner {
    fn create_websocket(&mut self, weak_self: &Weak<Mutex<Inner>>) {
        let ws = WebSocket::new_with_protocols(&self.url, &["exocore_websocket"]).unwrap();
        ws.set_binary_type(SocketBinaryType::ArrayBuffer);

        // SEE: https://github.com/koute/stdweb/blob/dff1e06086124fe79e3393a99ae8e2d424f5b2f1/examples/echo/src/main.rs
        let remote_node = self.remote_node.clone();
        let weak_self = weak_self.clone();
        ws.add_event_listener(move |event: SocketMessageEvent| {
            let data = Vec::from(event.data().into_array_buffer().unwrap());
            let frame = TypedCapnpFrame::<_, envelope::Owned>::new(data.as_slice()).unwrap();
            let in_message = InMessage::from_node_and_frame(remote_node.clone(), frame);

            match in_message {
                Ok(msg) => {
                    if let Err(err) = Self::handle_incoming_message(&weak_self, msg) {
                        error!("Error handling incoming message: {}", err);
                    }
                }
                Err(err) => {
                    error!("Error parsing incoming message: {}", err);
                }
            }
        });

        ws.add_event_listener(move |_event: SocketOpenEvent| {
            info!("Connected");
        });

        self.last_connect_attempt = Some(Instant::now());
        self.socket = Some(ws);
    }

    fn check_should_reconnect(&mut self, weak_self: &Weak<Mutex<Inner>>) -> Result<(), Error> {
        if let Some(ws) = &self.socket {
            match ws.ready_state() {
                SocketReadyState::Connecting | SocketReadyState::Open => {
                    // nothing to do
                }
                socket_status => {
                    let should_reconnect = self
                        .last_connect_attempt
                        .map_or(true, |attempt| attempt.elapsed() > Duration::from_secs(5));

                    if should_reconnect {
                        let last_connect_elapsed = self.last_connect_attempt.map(|i| i.elapsed());
                        info!("WebSocket was not connected. Reconnecting... (socket_status={:?} last_connect_attempt={:?})", socket_status, last_connect_elapsed );
                        self.create_websocket(weak_self);
                    }
                }
            }
        } else {
            warn!("No WebSocket was started. Reconnecting...");
            self.create_websocket(weak_self);
        }

        Ok(())
    }

    fn handle_incoming_message(
        weak_self: &Weak<Mutex<Inner>>,
        in_message: InMessage,
    ) -> Result<(), Error> {
        let inner = weak_self.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.lock()?;

        let key = (in_message.cell_id.clone(), in_message.layer);
        let handle = inner
            .handles
            .get_mut(&key)
            .ok_or_else(|| Error::Other(format!("Got message for unknown handle: {:?}", key)))?;

        handle.in_sender.try_send(in_message).map_err(|err| {
            Error::Other(format!("Got error sending to handle {:?}: {}", key, err))
        })?;

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
