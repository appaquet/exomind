extern crate websocket;

use self::websocket::client::r#async::{Framed, TcpStream};
use self::websocket::r#async::MessageCodec;
use self::websocket::OwnedMessage;
pub use self::websocket::WebSocketError;
use crate::transport::{MpscHandleSink, MpscHandleStream};
use crate::{Error, InMessage, OutMessage, TransportHandle};
use exocore_common::cell::{Cell, CellId};
use exocore_common::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_common::node::{Node, NodeId};
use exocore_common::protos::common_capnp::envelope;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use futures::prelude::*;
use futures::sync::mpsc;
use futures::MapErr;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock, Weak};

static WEBSOCKET_PROTOCOL: &str = "exocore_websocket";

// TODO: Handle Cell authentication: https://github.com/appaquet/exocore/issues/73

///
/// Configuration for WebSocket transport
///
#[derive(Clone, Copy)]
pub struct WebSocketTransportConfig {
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub handle_out_to_websocket_channel_size: usize,
}

impl Default for WebSocketTransportConfig {
    fn default() -> Self {
        WebSocketTransportConfig {
            handle_in_channel_size: 1000,
            handle_out_channel_size: 1000,
            handle_out_to_websocket_channel_size: 1000,
        }
    }
}

///
/// WebSocket based transport made for communication from WASM / Web.
///
/// It's not a full transport since it cannot allow sending messages to cluster nodes, but
/// only to inbound connections.
///
/// Each connection is assigned a temporary node that is used internally for communication.
///
pub struct WebsocketTransport {
    config: WebSocketTransportConfig,
    listen_address: SocketAddr,
    start_notifier: CompletionNotifier<(), Error>,
    inner: Arc<RwLock<InnerTransport>>,
    stop_listener: CompletionListener<(), Error>,
}

struct InnerTransport {
    config: WebSocketTransportConfig,
    handles: HashMap<CellId, InnerHandle>,
    connections: HashMap<NodeId, Connection>,
    stop_notifier: CompletionNotifier<(), Error>,
}

impl InnerTransport {
    fn remove_handle(&mut self, cell_id: &CellId) {
        self.handles.remove(cell_id);
        if self.handles.is_empty() {
            info!("No more handles, killing transport");
            self.stop_notifier.complete(Ok(()));
        }
    }
}

impl WebsocketTransport {
    pub fn new(listen_address: SocketAddr, config: WebSocketTransportConfig) -> WebsocketTransport {
        let start_notifier = CompletionNotifier::new();
        let (stop_notifier, stop_listener) = CompletionNotifier::new_with_listener();

        let inner = Arc::new(RwLock::new(InnerTransport {
            config,
            handles: HashMap::new(),
            connections: HashMap::new(),
            stop_notifier,
        }));

        WebsocketTransport {
            config,
            start_notifier,
            listen_address,
            inner,
            stop_listener,
        }
    }

    pub fn get_handle(&mut self, cell: &Cell) -> Result<WebsocketTransportHandle, Error> {
        let start_listener = self.start_notifier.get_listener().map_err(|err| {
            Error::Other(format!(
                "Couldn't get a listener on start notifier: {}",
                err
            ))
        })?;
        let stop_listener = self.stop_listener.try_clone().map_err(|err| {
            Error::Other(format!("Couldn't clone listener on stop notifier: {}", err))
        })?;

        let (out_sink, out_stream) = mpsc::channel(self.config.handle_in_channel_size);
        let (in_sink, in_stream) = mpsc::channel(self.config.handle_out_channel_size);

        let inner_handle = InnerHandle {
            out_stream: Some(out_stream),
            in_sink,
        };

        {
            let mut inner = self.inner.write()?;
            inner.handles.insert(cell.id().clone(), inner_handle);
        }

        Ok(WebsocketTransportHandle {
            cell_id: cell.id().clone(),
            start_listener,
            inner_transport: Arc::downgrade(&self.inner),
            sink: Some(out_sink),
            stream: Some(in_stream),
            stop_listener,
        })
    }

    fn start(&mut self) -> Result<(), Error> {
        self.schedule_handles_streams()?;
        self.start_websocket_server()?;

        Ok(())
    }

    fn schedule_handles_streams(&mut self) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        for handle in inner.handles.values_mut() {
            let weak_inner = Arc::downgrade(&self.inner);
            let stream_future = handle
                .out_stream
                .take()
                .expect("Out stream for handle was already taken out")
                .for_each(move |out_message| {
                    WebsocketTransport::dispatch_outgoing_message(&weak_inner, &out_message)
                        .map_err(|err| {
                            error!("Error dispatching message from handle: {}", err);
                        })
                });
            tokio::spawn(stream_future);
        }

        Ok(())
    }

    fn start_websocket_server(&mut self) -> Result<(), Error> {
        let reactor_handle = &tokio::reactor::Handle::default();
        let server = websocket::r#async::Server::bind(self.listen_address, reactor_handle)
            .map_err(|err| Error::Other(format!("Cannot start websocket: {}", err)))?;

        let inner1 = Arc::downgrade(&self.inner);
        let inner2 = Arc::downgrade(&self.inner);
        let incoming_stream = server
            .incoming()
            .map_err(|err| Error::Other(format!("Invalid incoming connection: {}", err.error)))
            .for_each(move |(upgrade, addr)| {
                // make sure we should still be running
                if inner1.upgrade().is_none() {
                    return Err(Error::Upgrade);
                }

                if !upgrade.protocols().iter().any(|s| s == WEBSOCKET_PROTOCOL) {
                    debug!("Rejecting connection {} with wrong connection", addr);
                    tokio::spawn(upgrade.reject().map(|_| ()).map_err(|_| ()));
                    return Ok(());
                }

                // accept the request to be a ws connection if it does
                debug!("Got a connection from: {}", addr);
                let weak_inner = inner1.clone();
                let client_connection = upgrade
                    .use_protocol(WEBSOCKET_PROTOCOL)
                    .accept()
                    .map_err(|err| Error::WebsocketTransport(Arc::new(err)))
                    .and_then(move |(connection, _)| {
                        Self::schedule_incoming_connection(weak_inner, connection)
                    })
                    .map(|_| ())
                    .map_err(|err| {
                        error!("Error in incoming connection accept: {}", err);
                    });
                tokio::spawn(client_connection);

                Ok(())
            })
            .map_err(move |err| {
                error!("Error in incoming connections stream: {}", err);
                if let Some(inner) = inner2.upgrade() {
                    if let Ok(inner) = inner.write() {
                        inner.stop_notifier.complete(Err(err));
                    }
                }
            });

        let stop_listener = self
            .stop_listener
            .try_clone()
            .expect("Couldn't clone stop listener");
        tokio::spawn(
            incoming_stream
                .select2(stop_listener)
                .map(|_| ())
                .map_err(|_| ()),
        );

        Ok(())
    }

    fn schedule_incoming_connection(
        weak_inner: Weak<RwLock<InnerTransport>>,
        framed: Framed<TcpStream, MessageCodec<OwnedMessage>>,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.write()?;

        // create a connection struct with temporary node associated with it
        let temporary_node = Node::generate_temporary();
        let (connection_sender, connection_receiver) =
            mpsc::channel(inner.config.handle_out_to_websocket_channel_size);
        let connection = Connection {
            _temporary_node: temporary_node.clone(),
            out_sink: connection_sender,
        };
        inner
            .connections
            .insert(temporary_node.id().clone(), connection);

        let (client_sink, client_stream) = framed.split();

        // handle outgoing messages to connection
        {
            let weak_inner = weak_inner.clone();
            let temporary_node = temporary_node.clone();
            let outgoing = connection_receiver
                .map(websocket::OwnedMessage::Binary)
                .forward(client_sink.sink_map_err(|_| ()))
                .map(|_| ())
                .map_err(move |_| {
                    let _ = Self::close_errored_connection(&weak_inner, &temporary_node);
                    Error::Other("Error in sink forward to connection".to_string())
                });
            tokio::spawn(outgoing.map(|_| ()).map_err(|_| ()));
        }

        // handle incoming messages from connection
        {
            let weak_inner1 = weak_inner.clone();
            let temporary_node1 = temporary_node.clone();
            let weak_inner2 = weak_inner.clone();
            let temporary_node2 = temporary_node.clone();
            let incoming = client_stream
                .take_while(|m| Ok(!m.is_close()))
                .for_each(move |message| {
                    debug!("Message from connection: {:?}", message);
                    if let Err(err) =
                        Self::handle_incoming_message(&weak_inner1, &temporary_node1, message)
                    {
                        error!("Error handling incoming message: {}", err);
                    }

                    Ok(())
                })
                .map_err(move |err| {
                    let _ = Self::close_errored_connection(&weak_inner2, &temporary_node2);
                    Error::Other(format!("Error in stream from connection: {}", err))
                });
            tokio::spawn(incoming.map(|_| ()).map_err(|_| ()));
        }

        Ok(())
    }

    fn dispatch_outgoing_message(
        weak_inner: &Weak<RwLock<InnerTransport>>,
        out_message: &OutMessage,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.write()?;

        let envelope_data = out_message.envelope_builder.as_bytes();
        for node in &out_message.to {
            if let Some(connection) = inner.connections.get_mut(node.id()) {
                let send_result = connection.out_sink.try_send(envelope_data.clone());
                if let Err(err) = send_result {
                    error!("Couldn't send message to node {}: {}", node.id(), err);
                }
            } else {
                warn!(
                    "Could not find a connection for node {}. Probably got closed.",
                    node.id()
                );
            }
        }

        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<InnerTransport>>,
        connection_node: &Node,
        message: OwnedMessage,
    ) -> Result<(), Error> {
        if let websocket::OwnedMessage::Binary(data) = message {
            let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
            let mut inner = inner.write()?;

            let envelope_frame = TypedCapnpFrame::<_, envelope::Owned>::new(data)?;
            for handle in inner.handles.values_mut() {
                let in_message = InMessage::from_node_and_frame(
                    connection_node.clone(),
                    envelope_frame.to_owned(),
                )?;
                if let Err(err) = handle.in_sink.try_send(in_message) {
                    error!("Error sending to handle: {}", err);
                }
            }

            Ok(())
        } else {
            Err(Error::Other(format!("Unhandled message: {:?}", message)))
        }
    }

    fn close_errored_connection(
        weak_inner: &Weak<RwLock<InnerTransport>>,
        connection_node: &Node,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.write()?;
        inner.connections.remove(connection_node.id());

        Ok(())
    }
}

impl Future for WebsocketTransport {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.start_notifier.is_complete() {
            let start_res = self.start();
            self.start_notifier.complete(start_res);
        }

        self.stop_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            _ => Error::Other("Error in completion error".to_string()),
        })
    }
}

///
/// Incoming WebSocket connection, with sink that can be used to send messages to it
///
struct Connection {
    _temporary_node: Node,
    out_sink: mpsc::Sender<Vec<u8>>,
}

///
/// Handle used to send & receive messages from active connections for a given cell
///
pub struct WebsocketTransportHandle {
    cell_id: CellId,
    start_listener: CompletionListener<(), Error>,
    inner_transport: Weak<RwLock<InnerTransport>>,
    sink: Option<mpsc::Sender<OutMessage>>,
    stream: Option<mpsc::Receiver<InMessage>>,
    stop_listener: CompletionListener<(), Error>,
}

struct InnerHandle {
    out_stream: Option<mpsc::Receiver<OutMessage>>,
    in_sink: mpsc::Sender<InMessage>,
}

type StartFutureType = MapErr<CompletionListener<(), Error>, fn(CompletionError<Error>) -> Error>;

impl TransportHandle for WebsocketTransportHandle {
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
        MpscHandleSink::new(self.sink.take().expect("Sink was already consumed"))
    }

    fn get_stream(&mut self) -> Self::Stream {
        MpscHandleStream::new(self.stream.take().expect("Stream was already consumed"))
    }
}

impl Future for WebsocketTransportHandle {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.stop_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            _ => Error::Other("Error in completion error".to_string()),
        })
    }
}

impl Drop for WebsocketTransportHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.inner_transport.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.remove_handle(&self.cell_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::websocket::ClientBuilder;
    use super::*;
    use crate::TransportLayer;
    use exocore_common::cell::FullCell;
    use exocore_common::framing::{CapnpFrameBuilder, FrameBuilder};
    use exocore_common::node::LocalNode;
    use exocore_common::tests_utils::expect_eventually;
    use std::sync::Mutex;
    use tokio::runtime::Runtime;

    #[test]
    fn client_send_receive() -> Result<(), failure::Error> {
        let node = LocalNode::generate();
        let cell = FullCell::generate(node);
        let mut rt = tokio::runtime::Runtime::new()?;

        let listen_address = "127.0.0.1:3100".parse()?;
        let config = WebSocketTransportConfig::default();
        let mut server = WebsocketTransport::new(listen_address, config);
        let handle = server.get_handle(&cell)?;

        // start server & wait for it to be started
        rt.spawn(server.map(|_| ()).map_err(|_| ()));
        let received_messages = schedule_server_handle(&mut rt, handle);

        let client = TestClient::new(&mut rt, "ws://127.0.0.1:3100");
        client.send_str("hello world");

        // server should have received message
        expect_eventually(|| {
            let received_messages = received_messages.lock().unwrap();
            received_messages
                .iter()
                .any(|data| data.as_str() == "hello world")
        });

        // client should have received replied message
        expect_eventually(|| {
            let received_messages = client.received_messages.lock().unwrap();
            received_messages
                .iter()
                .any(|data| data.as_str() == "hello world")
        });

        Ok(())
    }

    fn schedule_server_handle(
        rt: &mut Runtime,
        mut handle: WebsocketTransportHandle,
    ) -> Arc<Mutex<Vec<String>>> {
        rt.block_on(handle.on_start()).unwrap();

        let received_messages = Arc::new(Mutex::new(Vec::new()));

        {
            let received_messages = received_messages.clone();
            rt.spawn(
                handle
                    .get_stream()
                    .and_then(move |message| {
                        let message_data = message.get_data().unwrap();
                        let mut received_messages = received_messages.lock().unwrap();
                        received_messages.push(String::from_utf8_lossy(message_data).to_string());

                        // forward the message back to client
                        let mut frame_builder = CapnpFrameBuilder::<envelope::Owned>::new();
                        {
                            let mut message_builder = frame_builder.get_builder();
                            message_builder.set_data(message_data);
                            message_builder.set_layer(TransportLayer::Meta.to_code());
                        }

                        let out_message = OutMessage {
                            to: vec![message.from.clone()],
                            expiration: None,
                            envelope_builder: frame_builder,
                        };
                        Ok(out_message)
                    })
                    .forward(handle.get_sink())
                    .map(|_| ())
                    .map_err(|_| ()),
            );
        }

        rt.spawn(handle.map(|_| ()).map_err(|_| ()));

        received_messages
    }

    struct TestClient {
        out_sender: mpsc::UnboundedSender<OwnedMessage>,
        received_messages: Arc<Mutex<Vec<String>>>,
    }

    impl TestClient {
        fn new(rt: &mut Runtime, url: &str) -> TestClient {
            let (out_sender, out_receiver) = mpsc::unbounded();
            let received_messages = Arc::new(Mutex::new(Vec::new()));

            {
                let received_messages = received_messages.clone();
                let builder = ClientBuilder::new(url)
                    .unwrap()
                    .add_protocol(WEBSOCKET_PROTOCOL)
                    .async_connect_insecure()
                    .and_then(move |(duplex, _)| {
                        let (sink, stream) = duplex.split();

                        let sink_future = out_receiver
                            .map_err(|_err| Error::Other("Error in out receiver".to_string()))
                            .forward(sink.sink_map_err(|err| {
                                Error::Other(format!("Error in sink: {}", err))
                            }))
                            .map(|_| ())
                            .map_err(|_| ());
                        tokio::spawn(sink_future);

                        let stream_future = stream
                            .for_each(move |msg| {
                                match msg {
                                    OwnedMessage::Binary(data) => {
                                        let envelope_frame =
                                            TypedCapnpFrame::<_, envelope::Owned>::new(data)
                                                .unwrap();
                                        let envelope_reader = envelope_frame.get_reader().unwrap();
                                        let message_data = String::from_utf8_lossy(
                                            envelope_reader.get_data().unwrap(),
                                        )
                                        .to_string();

                                        let mut received_messages =
                                            received_messages.lock().unwrap();
                                        received_messages.push(message_data);
                                    }
                                    _ => panic!("Received unexpected message: {:?}", msg),
                                }

                                Ok(())
                            })
                            .map(|_| ())
                            .map_err(|_| ());
                        tokio::spawn(stream_future);

                        Ok(())
                    })
                    .into_future()
                    .map(|_| ())
                    .map_err(|_| ());
                rt.spawn(builder);
            }

            TestClient {
                out_sender,
                received_messages,
            }
        }

        fn send_str(&self, data: &str) {
            let mut frame_builder = CapnpFrameBuilder::<envelope::Owned>::new();
            {
                let mut message_builder: envelope::Builder = frame_builder.get_builder();
                message_builder.set_data(data.as_bytes());
                message_builder.set_layer(TransportLayer::Meta.to_code());
            }

            let frame_data = frame_builder.as_bytes();
            self.out_sender
                .unbounded_send(OwnedMessage::Binary(frame_data))
                .unwrap();
        }
    }
}
