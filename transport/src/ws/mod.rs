extern crate websocket;

use self::websocket::client::r#async::{Framed, TcpStream};
use self::websocket::r#async::MessageCodec;
use self::websocket::OwnedMessage;
pub use self::websocket::WebSocketError;
use crate::transport::{MpscHandleSink, MpscHandleStream};
use crate::{Error, InMessage, OutMessage, TransportHandle};
use exocore_common::cell::{Cell, CellId};
use exocore_common::node::{Node, NodeId};
use exocore_common::serialization::framed::{
    FrameBuilder, OwnedTypedFrame, TypedFrame, TypedSliceFrame,
};
use exocore_common::serialization::protos::data_transport_capnp::envelope;
use futures::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock, Weak};

///
///
///
#[derive(Clone, Copy)]
pub struct Config {
    pub handle_in_channel_size: usize,
    pub handle_out_channel_size: usize,
    pub handle_out_to_websocket_channel_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            handle_in_channel_size: 1000,
            handle_out_channel_size: 1000,
            handle_out_to_websocket_channel_size: 1000,
        }
    }
}

///
///
///
pub struct WebsocketTransport {
    config: Config,
    listen_address: SocketAddr,
    inner: Arc<RwLock<InnerTransport>>,
}

struct InnerTransport {
    config: Config,
    handles: HashMap<CellId, InnerHandle>,
    connections: HashMap<NodeId, Connection>,
}

impl WebsocketTransport {
    pub fn new(listen_address: SocketAddr, config: Config) -> WebsocketTransport {
        let inner = Arc::new(RwLock::new(InnerTransport {
            config,
            handles: HashMap::new(),
            connections: HashMap::new(),
        }));

        WebsocketTransport {
            config,
            listen_address,
            inner,
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
            inner_transport: Arc::downgrade(&self.inner),
            sink: Some(out_sink),
            stream: Some(in_stream),
        })
    }

    fn start(&mut self) -> Result<(), Error> {
        self.schedule_handles_streams()?;
        self.start_websocket_server()?;

        Ok(())
    }

    fn schedule_handles_streams(&mut self) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        for (_cell, handle) in &mut inner.handles {
            let weak_inner = Arc::downgrade(&self.inner);
            let stream_future = handle
                .out_stream
                .take()
                .expect("Out stream for handle was already taken out")
                .for_each(move |out_message| {
                    WebsocketTransport::dispatch_outgoing_message(&weak_inner, &out_message)
                        .map_err(|err| ())
                })
                .map_err(|_| {});

            tokio_executor::spawn(stream_future);
        }

        Ok(())
    }

    fn start_websocket_server(&mut self) -> Result<(), Error> {
        let reactor_handle = &tokio_reactor::Handle::default();
        let server = websocket::r#async::Server::bind(self.listen_address, reactor_handle)
            .map_err(|err| Error::Other(format!("Cannot start websocket: {}", err)))?;

        // the server will own the strong ref on inner. if it get killed, the transport is killed
        let inner = Arc::clone(&self.inner);
        let incoming_stream = server
            .incoming()
            .map_err(|err| Error::Other("Invalid connection".to_string()))
            .for_each(move |(upgrade, addr)| {
                {
                    // check if we should still be running
                    let inner = inner.read()?;
                    if inner.handles.is_empty() {
                        info!("All handles have been dropped. Stopping transport.");
                        return Err(Error::Other("Stopped".to_string()));
                    }
                }

                if !upgrade.protocols().iter().any(|s| s == "exocore_websocket") {
                    debug!("Rejecting connection {} with wrong connection", addr);
                    tokio_executor::spawn(upgrade.reject().map(|_| ()).map_err(|_| ()));
                    return Ok(());
                }

                // accept the request to be a ws connection if it does
                debug!("Got a connection from: {}", addr);
                let weak_inner = Arc::downgrade(&inner);
                let client_connection = upgrade
                    .use_protocol("exocore_websocket")
                    .accept()
                    .map_err(|err| Error::WebsocketTransport(err))
                    .and_then(move |(connection, _)| {
                        Self::schedule_incoming_connection(weak_inner, connection)
                    })
                    .map(|_| ())
                    .map_err(|err| {
                        error!("Error in incoming connection accept: {}", err);
                    });
                tokio_executor::spawn(client_connection);

                Ok(())
            })
            .map_err(|_| {
                // TODO: Should kill inner
                ()
            });

        tokio_executor::spawn(incoming_stream);

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
            temporary_node: temporary_node.clone(),
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
                .map(|out_msg| websocket::OwnedMessage::Binary(out_msg.frame_data().to_vec()))
                .forward(client_sink.sink_map_err(|_| ()))
                .map(|_| ())
                .map_err(move |_| {
                    let _ = Self::close_errored_connection(&weak_inner, &temporary_node);
                    Error::Other(format!("Error in sink forward to connection"))
                });
            tokio_executor::spawn(outgoing.map(|_| ()).map_err(|_| ()));
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
                    debug!("Message from client: {:?}", message);
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
            tokio_executor::spawn(incoming.map(|_| ()).map_err(|_| ()));
        }

        Ok(())
    }

    fn dispatch_outgoing_message(
        weak_inner: &Weak<RwLock<InnerTransport>>,
        out_message: &OutMessage,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
        let mut inner = inner.write()?;

        // TODO: Should sign?
        let frame = out_message.envelope.as_owned_unsigned_framed()?;
        for node in &out_message.to {
            if let Some(connection) = inner.connections.get_mut(node.id()) {
                let send_result = connection.out_sink.try_send(frame.clone());
                if let Err(err) = send_result {
                    error!("Couldn't send message to node {}: {}", node.id(), err);
                }
            } else {
                warn!(
                    "Couldn't find a connection for node {}. Probably got closed.",
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
        if let websocket::OwnedMessage::Binary(data) = &message {
            let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
            let mut inner = inner.write()?;

            let envelope_frame = TypedSliceFrame::<envelope::Owned>::new(data)?;
            for (_cell_id, handle) in &mut inner.handles {
                let in_message = InMessage {
                    from: connection_node.clone(),
                    envelope: envelope_frame.to_owned(),
                };

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
        self.start()?;

        Ok(Async::Ready(()))
    }
}

///
///
///
struct Connection {
    temporary_node: Node,
    out_sink: mpsc::Sender<OwnedTypedFrame<envelope::Owned>>,
}

///
///
///
pub struct WebsocketTransportHandle {
    inner_transport: Weak<RwLock<InnerTransport>>,
    sink: Option<mpsc::Sender<OutMessage>>,
    stream: Option<mpsc::Receiver<InMessage>>,
}

struct InnerHandle {
    out_stream: Option<mpsc::Receiver<OutMessage>>,
    in_sink: mpsc::Sender<InMessage>,
}

impl TransportHandle for WebsocketTransportHandle {
    type Sink = MpscHandleSink;
    type Stream = MpscHandleStream;

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
        Ok(Async::Ready(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exocore_common::cell::FullCell;
    use exocore_common::node::LocalNode;
    use exocore_common::utils::setup_logging;
    use std::time::Duration;

    //#[ignore]
    #[test]
    fn test_server() -> Result<(), failure::Error> {
        setup_logging();

        let node = LocalNode::generate();
        let cell = FullCell::generate(node);
        let mut rt = tokio::runtime::Runtime::new()?;

        let listen_address = "127.0.0.1:3341".parse()?;
        let config = Config::default();
        let mut server = WebsocketTransport::new(listen_address, config);
        let mut handle = server.get_handle(&cell)?;

        // start server
        rt.block_on(server)?;

        // then get connection
        let sink = handle.get_sink();

        rt.spawn(
            handle
                .get_stream()
                .and_then(|message| {
                    let envelope_reader: envelope::Reader = message.envelope.get_typed_reader()?;
                    let data = envelope_reader.get_data()?;
                    info!("Got message: {}", String::from_utf8_lossy(data));

                    let mut frame_builder = FrameBuilder::<envelope::Owned>::new();
                    {
                        let mut builder: envelope::Builder = frame_builder.get_builder_typed();
                        builder.set_data(data);
                    }

                    Ok(OutMessage {
                        to: vec![message.from],
                        envelope: frame_builder,
                    })
                })
                .forward(sink)
                .map(|_| ())
                .map_err(|_| ()),
        );

        std::thread::sleep(Duration::from_secs(1000));

        Ok(())
    }
}
