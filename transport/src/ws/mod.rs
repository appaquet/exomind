extern crate websocket;

use self::websocket::client::r#async::{Framed, TcpStream};
use self::websocket::r#async::MessageCodec;
use self::websocket::OwnedMessage;
pub use self::websocket::WebSocketError;
use crate::transport::{MpscHandleSink, MpscHandleStream, TransportHandleOnStart};
use crate::{Error, InEvent, InMessage, OutEvent, TransportHandle};
use exocore_common::cell::{Cell, CellId};
use exocore_common::framing::{FrameBuilder, TypedCapnpFrame};
use exocore_common::node::{Node, NodeId};
use exocore_common::protos::common_capnp::envelope;
use exocore_common::utils::futures::{spawn_future, spawn_future_01};
use exocore_common::utils::handle_set::{Handle, HandleSet};
use futures::channel::mpsc;
use futures::compat::{Sink01CompatExt, Stream01CompatExt};
use futures::prelude::*;
use futures::FutureExt;
use futures01::stream::Stream as Stream01;
use futures01::Future as Future01;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll};

static WEBSOCKET_PROTOCOL: &str = "exocore_websocket";

// TODO: Handle Cell authentication: https://github.com/appaquet/exocore/issues/73

/// Configuration for WebSocket transport
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

/// WebSocket based transport made for communication from WASM / Web.
///
/// It's not a full transport since it cannot allow sending messages to cluster nodes, but
/// only to inbound connections.
///
/// Each connection is assigned a temporary node that is used internally for communication.
pub struct WebsocketTransport {
    config: WebSocketTransportConfig,
    listen_address: SocketAddr,
    inner: Arc<RwLock<InnerTransport>>,
    handle_set: HandleSet,
}

struct InnerTransport {
    config: WebSocketTransportConfig,
    handles: HashMap<CellId, InnerHandle>,
    connections: HashMap<NodeId, Connection>,
}

impl WebsocketTransport {
    pub fn new(listen_address: SocketAddr, config: WebSocketTransportConfig) -> WebsocketTransport {
        let inner = Arc::new(RwLock::new(InnerTransport {
            config,
            handles: HashMap::new(),
            connections: HashMap::new(),
        }));

        let handle_set = HandleSet::new();
        WebsocketTransport {
            config,
            listen_address,
            inner,
            handle_set,
        }
    }

    pub fn get_handle(&mut self, cell: &Cell) -> Result<WebsocketTransportHandle, Error> {
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
            sink: Some(out_sink),
            stream: Some(in_stream),
            handle: self.handle_set.get_handle(),
        })
    }

    pub async fn run(self) -> Result<(), Error> {
        let outgoing_handler = {
            let mut handles_senders = Vec::new();
            let mut inner = self.inner.write()?;
            for handle in inner.handles.values_mut() {
                let weak_inner = Arc::downgrade(&self.inner);

                let mut stream = handle
                    .out_stream
                    .take()
                    .expect("Out stream for handle was already taken out");

                let stream_future = async move {
                    while let Some(out_event) = stream.next().await {
                        if let Err(err) =
                            WebsocketTransport::dispatch_outgoing_event(&weak_inner, &out_event)
                        {
                            error!("Error dispatching message from handle: {}", err);
                        }
                    }
                };

                handles_senders.push(stream_future);
            }

            futures::future::join_all(handles_senders)
        };

        let reactor_handle = &tokio::reactor::Handle::default();
        let server = websocket::r#async::Server::bind(self.listen_address, reactor_handle)
            .map_err(|err| Error::Other(format!("Cannot start websocket: {}", err)))?;
        let inner = Arc::downgrade(&self.inner);
        let incoming_handler = async move {
            let mut incoming_connections = server.incoming().compat();
            while let Some(connection) = incoming_connections.next().await {
                let (upgrade, addr) = if let Err(err) = connection {
                    error!("Invalid incoming connection: {:?}", err);
                    continue;
                } else {
                    connection.unwrap()
                };

                if !upgrade.protocols().iter().any(|s| s == WEBSOCKET_PROTOCOL) {
                    debug!("Rejecting connection {} with wrong connection", addr);
                    spawn_future_01(upgrade.reject().map(|_| ()).map_err(|_| ()));
                    continue;
                }

                // accept the request to be a ws connection if it does
                debug!("Got a connection from: {}", addr);
                let weak_inner = inner.clone();
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
                spawn_future_01(client_connection);
            }
        };

        info!("Websocket transport server now running");
        futures::select! {
             _ = outgoing_handler.fuse() => (),
             _ = incoming_handler.fuse() => (),
             _ = self.handle_set.on_handles_dropped().fuse() => (),
        }
        info!("Websocket transport server done");

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
        let (connection_sender, mut connection_receiver) =
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
            let mut client_sink = client_sink.sink_compat();
            spawn_future(async move {
                while let Some(data) = connection_receiver.next().await {
                    let message = websocket::OwnedMessage::Binary(data);
                    if let Err(err) = client_sink.send(message).await {
                        error!("Error in websocket connection. Closing it: {}", err);
                        let _ = Self::close_errored_connection(&weak_inner, &temporary_node);
                    }
                }
            });
        }

        // handle incoming messages from connection
        {
            let weak_inner1 = weak_inner.clone();
            let temporary_node1 = temporary_node.clone();
            let weak_inner2 = weak_inner;
            let temporary_node2 = temporary_node;
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
                    error!("Error in websocket connection. Closing it: {}", err);
                    let _ = Self::close_errored_connection(&weak_inner2, &temporary_node2);
                    Error::Other(format!("Error in stream from connection: {}", err))
                });
            spawn_future_01(incoming.map(|_| ()).map_err(|_| ()));
        }

        Ok(())
    }

    fn dispatch_outgoing_event(
        weak_inner: &Weak<RwLock<InnerTransport>>,
        event: &OutEvent,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.write()?;

        let OutEvent::Message(msg) = event;
        let envelope_data = msg.envelope_builder.as_bytes();
        for node in &msg.to {
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

                if let Err(err) = handle.in_sink.try_send(InEvent::Message(in_message)) {
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
    sink: Option<mpsc::Sender<OutEvent>>,
    stream: Option<mpsc::Receiver<InEvent>>,
    handle: Handle,
}

struct InnerHandle {
    out_stream: Option<mpsc::Receiver<OutEvent>>,
    in_sink: mpsc::Sender<InEvent>,
}

impl TransportHandle for WebsocketTransportHandle {
    type Sink = MpscHandleSink;
    type Stream = MpscHandleStream;

    fn on_start(&self) -> TransportHandleOnStart {
        Box::new(self.handle.on_set_started())
    }

    fn get_sink(&mut self) -> Self::Sink {
        MpscHandleSink::new(self.sink.take().expect("Sink was already consumed"))
    }

    fn get_stream(&mut self) -> Self::Stream {
        MpscHandleStream::new(self.stream.take().expect("Stream was already consumed"))
    }
}

impl Future for WebsocketTransportHandle {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.handle.on_set_dropped().poll_unpin(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::websocket::ClientBuilder;
    use super::*;
    use crate::{OutMessage, TransportLayer};
    use exocore_common::cell::FullCell;
    use exocore_common::framing::{CapnpFrameBuilder, FrameBuilder};
    use exocore_common::node::LocalNode;
    use exocore_common::tests_utils::{expect_eventually, setup_logging};
    use exocore_common::utils::futures::Runtime;
    use futures::compat::Future01CompatExt;
    use std::sync::Mutex;

    #[test]
    fn client_send_receive() -> Result<(), failure::Error> {
        setup_logging();

        let node = LocalNode::generate();
        let cell = FullCell::generate(node);
        let mut rt = Runtime::new()?;

        let listen_address = "127.0.0.1:3100".parse()?;
        let config = WebSocketTransportConfig::default();
        let mut server = WebsocketTransport::new(listen_address, config);
        let handle = server.get_handle(&cell)?;

        // start server & wait for it to be started
        rt.spawn_std(async move {
            if let Err(err) = server.run().await {
                error!("Error in server: {}", err);
            }
        });
        let received_messages = schedule_server_handle(&mut rt, handle);

        let client = TestClient::new(&mut rt, "ws://127.0.0.1:3100");
        client.send_str("hello world");

        // server should have received message
        expect_eventually(|| {
            let received_messages = received_messages.lock().unwrap();
            received_messages
                .iter()
                .filter_map(extract_message_event)
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

    fn extract_message_event(event: &InEvent) -> Option<String> {
        match event {
            InEvent::Message(msg) => {
                let message_data = msg.get_data().unwrap();
                let data = String::from_utf8_lossy(message_data).to_string();
                Some(data)
            }
            _ => None,
        }
    }

    fn schedule_server_handle(
        rt: &mut Runtime,
        mut handle: WebsocketTransportHandle,
    ) -> Arc<Mutex<Vec<InEvent>>> {
        rt.block_on_std(handle.on_start());

        let received_events = Arc::new(Mutex::new(Vec::new()));

        // loops back messages
        let received_events1 = received_events.clone();
        let mut stream = handle.get_stream();
        let mut sink = handle.get_sink();
        rt.spawn_std(async move {
            while let Some(event) = stream.next().await {
                {
                    let mut received_events = received_events1.lock().unwrap();
                    received_events.push(event.clone());
                }

                if let InEvent::Message(msg) = event {
                    let mut frame_builder = CapnpFrameBuilder::<envelope::Owned>::new();
                    {
                        let mut message_builder = frame_builder.get_builder();
                        message_builder.set_data(msg.get_data().unwrap());
                        message_builder.set_layer(TransportLayer::Meta.to_code());
                    }

                    let out_message = OutMessage {
                        to: vec![msg.from.clone()],
                        expiration: None,
                        envelope_builder: frame_builder,
                    };

                    let _ = sink.send(OutEvent::Message(out_message)).await;
                }
            }
        });

        rt.spawn_std(handle);

        received_events
    }

    struct TestClient {
        out_sender: mpsc::UnboundedSender<OwnedMessage>,
        received_messages: Arc<Mutex<Vec<String>>>,
    }

    impl TestClient {
        fn new(rt: &mut Runtime, url: &str) -> TestClient {
            let (out_sender, mut out_receiver) = mpsc::unbounded();
            let received_messages = Arc::new(Mutex::new(Vec::new()));

            let url = url.to_string();
            let received_messages1 = received_messages.clone();
            rt.spawn_std(
                async move {
                    let (duplex, _) = ClientBuilder::new(&url)
                        .unwrap()
                        .add_protocol(WEBSOCKET_PROTOCOL)
                        .async_connect_insecure()
                        .compat()
                        .await?;

                    let (sink, stream) = duplex.split();
                    let sender = async move {
                        let mut sink = sink.sink_compat();
                        while let Some(msg) = out_receiver.next().await {
                            debug!("Sending message to server");
                            if let Err(err) = sink.send(msg).await {
                                error!("Error sending to server: {}", err);
                            }
                        }
                    };

                    let receiver = async move {
                        let mut stream = stream.compat();
                        while let Some(msg) = stream.next().await {
                            match msg {
                                Ok(OwnedMessage::Binary(data)) => {
                                    let envelope_frame =
                                        TypedCapnpFrame::<_, envelope::Owned>::new(data).unwrap();
                                    let envelope_reader = envelope_frame.get_reader().unwrap();
                                    let message_data = String::from_utf8_lossy(
                                        envelope_reader.get_data().unwrap(),
                                    )
                                    .to_string();

                                    let mut received_messages = received_messages1.lock().unwrap();
                                    received_messages.push(message_data);
                                }
                                _ => {
                                    error!("Receiver an invalid message through websocket client");
                                    return;
                                }
                            }
                        }
                    };

                    info!("Test client started");
                    futures::select! {
                        _ = sender.fuse() => (),
                        _ = receiver.fuse() => (),
                    }
                    info!("Test client done");

                    Ok::<_, failure::Error>(())
                }
                .map_err(|err| {
                    error!("Error in client: {}", err);
                })
                .map(|_| ()),
            );

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
