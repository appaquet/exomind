use std::{
    collections::VecDeque,
    io, iter,
    task::{Context, Poll},
};

use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use futures::{future::BoxFuture, prelude::*, AsyncReadExt, AsyncWriteExt};
use libp2p::{
    core::{InboundUpgrade, OutboundUpgrade, UpgradeInfo},
    swarm::{
        handler::{
            ConnectionEvent, ConnectionHandler, ConnectionHandlerEvent, DialUpgradeError,
            KeepAlive, SubstreamProtocol,
        },
        Stream,
    },
};

use super::bytes_channel::BytesChannelSender;

const MAX_MESSAGE_SIZE: usize = 20 * 1024 * 1024; // 20MB
const STREAM_BUFFER_SIZE: usize = 1024;

type HandlerEvent = ConnectionHandlerEvent<ExocoreProtoConfig, (), MessageData, io::Error>;

// TODO: Remove dyn dispatched future once type_alias_impl_trait lands: https://github.com/rust-lang/rust/issues/63063
type InboundStreamFuture =
    BoxFuture<'static, Result<(Option<MessageData>, WrappedStream<Stream>), io::Error>>;
type OutboundStreamFuture = BoxFuture<'static, Result<Option<WrappedStream<Stream>>, io::Error>>;

/// Protocol handler for Exocore protocol. This handles protocols and substreams
/// to with a connection with a remote peer. This sends and receives messages
/// from the substreams for the behaviour.
///
/// It handles:
///   * Outgoing message requests from the behaviour.
///   * If we don't have any outgoing streams, we request one from libp2p, which
///     then upgrade a stream for us using `ExocoreProtoConfig`
///   * When an outgoing stream is open, it writes the outgoing messages to it.
///     Since this is asynchronous, we keep the futures and poll to completion.
///   * When an incoming stream is open to us, it reads the incoming message
///     from it. Since this is asynchronous, we keep the futures and poll to
///     completion.
pub struct ExocoreProtoHandler {
    listen_protocol: SubstreamProtocol<ExocoreProtoConfig, ()>,
    inbound_stream_futures: Vec<InboundStreamFuture>,
    outbound_dialing: bool,
    outbound_stream_futures: Vec<OutboundStreamFuture>,
    idle_outbound_stream: Option<WrappedStream<Stream>>,
    send_queue: VecDeque<MessageData>,
    keep_alive: KeepAlive,
}

impl ExocoreProtoHandler {
    fn handle_dial_upgrade_error(&mut self, event: DialUpgradeError<(), ExocoreProtoConfig>) {
        debug!("Upgrade error. Dropping stream. {err}", err = event.error);
        self.outbound_dialing = false;
    }

    fn handle_full_negotiated_inbound(
        &mut self,
        event: libp2p::swarm::handler::FullyNegotiatedInbound<ExocoreProtoConfig, ()>,
    ) {
        trace!("Inbound negotiated");
        self.inbound_stream_futures
            .push(Box::pin(event.protocol.read_next()));
    }

    fn handle_full_negotiated_outbound(
        &mut self,
        event: libp2p::swarm::handler::FullyNegotiatedOutbound<ExocoreProtoConfig, ()>,
    ) {
        trace!("Outbound negotiated. Sending message.");
        self.outbound_dialing = false;
        self.idle_outbound_stream = Some(event.protocol);
    }
}

impl Default for ExocoreProtoHandler {
    fn default() -> Self {
        ExocoreProtoHandler {
            listen_protocol: SubstreamProtocol::new(ExocoreProtoConfig, ()),
            inbound_stream_futures: Vec::new(),
            outbound_dialing: false,
            outbound_stream_futures: Vec::new(),
            idle_outbound_stream: None,
            send_queue: VecDeque::new(),
            keep_alive: KeepAlive::Yes,
        }
    }
}

impl ConnectionHandler for ExocoreProtoHandler {
    type FromBehaviour = MessageData;
    type ToBehaviour = MessageData;
    type Error = io::Error;
    type InboundProtocol = ExocoreProtoConfig;
    type InboundOpenInfo = ();
    type OutboundProtocol = ExocoreProtoConfig;
    type OutboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, ()> {
        self.listen_protocol.clone()
    }

    fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
        self.send_queue.push_back(event);
    }

    fn on_connection_event(
        &mut self,
        event: ConnectionEvent<
            Self::InboundProtocol,
            Self::OutboundProtocol,
            Self::InboundOpenInfo,
            Self::OutboundOpenInfo,
        >,
    ) {
        match event {
            ConnectionEvent::FullyNegotiatedInbound(event) => {
                self.handle_full_negotiated_inbound(event);
            }
            ConnectionEvent::FullyNegotiatedOutbound(event) => {
                self.handle_full_negotiated_outbound(event);
            }
            ConnectionEvent::DialUpgradeError(event) => {
                self.handle_dial_upgrade_error(event);
            }
            ConnectionEvent::AddressChange(event) => {
                debug!("Address change: {addr}", addr = event.new_address);
            }
            ConnectionEvent::ListenUpgradeError(event) => {
                error!("Listen upgrade error: {err}", err = event.error);
            }
            ConnectionEvent::LocalProtocolsChange(_event) => {
                debug!("Local protocols change");
            }
            ConnectionEvent::RemoteProtocolsChange(_event) => {
                debug!("Remote protocols change");
            }
        }
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    fn poll(&mut self, cx: &mut Context) -> Poll<HandlerEvent> {
        // if we have a message to send, but no outgoing streams via which to send it,
        // we request one
        if !self.send_queue.is_empty()
            && self.idle_outbound_stream.is_none()
            && self.outbound_stream_futures.is_empty()
            && !self.outbound_dialing
        {
            trace!("Asking to open outbound stream");

            self.outbound_dialing = true; // only one dialing at the time
            return Poll::Ready(ConnectionHandlerEvent::OutboundSubstreamRequest {
                protocol: self.listen_protocol.clone(),
            });
        }

        // if we have a message to send, and a stream it available, we write the message
        // to it and keep the future to poll to completion
        if self.idle_outbound_stream.is_some() && !self.send_queue.is_empty() {
            trace!("Sending message to idle output stream");
            let message = self.send_queue.pop_front().unwrap();
            let stream = self.idle_outbound_stream.take().unwrap();
            self.outbound_stream_futures
                .push(Box::pin(stream.send_message(message)));
        }

        // we poll all futures that writes messages to completion. once completed, we
        // take back the stream for next message.
        if !self.outbound_stream_futures.is_empty() {
            let futures = std::mem::take(&mut self.outbound_stream_futures);
            for mut fut in futures {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(Some(substream))) => {
                        if self.idle_outbound_stream.is_some() {
                            trace!("Successfully sent message. One stream already opening / ongoing. Closing this one");
                        } else if let Some(message) = self.send_queue.pop_front() {
                            trace!("Successfully sent message. Sending a new one from queue.");
                            self.outbound_stream_futures
                                .push(Box::pin(substream.send_message(message)));
                        } else if self.idle_outbound_stream.is_none() {
                            trace!("Successfully sent message. None in queue. Idling");
                            self.idle_outbound_stream = Some(substream);
                        }
                    }
                    Poll::Ready(Ok(None)) => {
                        trace!(
                            "Successfully sent message. Substream was consumed (had streaming)."
                        );
                    }
                    Poll::Ready(Err(err)) => {
                        debug!("Error sending message: {}", err);
                        return Poll::Ready(ConnectionHandlerEvent::Close(err));
                    }
                    Poll::Pending => {
                        self.outbound_stream_futures.push(fut);
                    }
                }
            }
        }

        // we poll all futures that reads messages to completion.
        if !self.inbound_stream_futures.is_empty() {
            let futures = std::mem::take(&mut self.inbound_stream_futures);
            for mut fut in futures {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok((opt_msg, substream))) => {
                        self.inbound_stream_futures
                            .push(Box::pin(substream.read_next()));

                        // we may not always have a message if the substream was currently handling
                        // copying data to a stream consumed by the application
                        if let Some(message) = opt_msg {
                            trace!("Successfully read a message on substream");
                            return Poll::Ready(ConnectionHandlerEvent::NotifyBehaviour(message));
                        }
                    }
                    Poll::Ready(Err(err)) => {
                        debug!("Error receiving on substream (it may have closed): {}", err);
                        return Poll::Ready(ConnectionHandlerEvent::Close(err));
                    }
                    Poll::Pending => {
                        self.inbound_stream_futures.push(fut);
                    }
                }
            }
        }

        Poll::Pending
    }
}

/// Protocol configuration that defines the protocol identification string and
/// stream upgrading capabilities.
///
/// Stream protocol negotiation and upgrading is entirely managed by libp2p.
/// Once an incoming stream or outgoing stream is upgraded, we wrap it into a
/// `WrappedStream` that will then be used by `ExocoreProtoHandler`.
#[derive(Clone, Default)]
pub struct ExocoreProtoConfig;

type UpgradeInfoData = &'static str;

impl UpgradeInfo for ExocoreProtoConfig {
    type Info = UpgradeInfoData;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once("/exocore/0.1.0")
    }
}

impl<TStream> InboundUpgrade<TStream> for ExocoreProtoConfig
where
    TStream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = WrappedStream<TStream>;
    type Error = io::Error;
    type Future = future::Ready<Result<WrappedStream<TStream>, io::Error>>;

    fn upgrade_inbound(self, socket: TStream, _: Self::Info) -> Self::Future {
        future::ok(WrappedStream::new(socket))
    }
}

impl<TStream> OutboundUpgrade<TStream> for ExocoreProtoConfig
where
    TStream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = WrappedStream<TStream>;
    type Error = io::Error;
    type Future = future::Ready<Result<WrappedStream<TStream>, io::Error>>;

    #[inline]
    fn upgrade_outbound(self, socket: TStream, _: Self::Info) -> Self::Future {
        future::ok(WrappedStream::new(socket))
    }
}

/// Wire message sent and receive over the streams managed by
/// `ExocoreProtoHandler`
pub struct MessageData {
    pub(crate) message: Bytes,
    pub(crate) stream: Option<Box<dyn AsyncRead + Send + Unpin>>,
}

impl std::fmt::Debug for MessageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageData")
            .field("message_len", &self.message.len())
            .field("with_stream", &self.stream.is_some())
            .finish()
    }
}

/// Wraps a stream to expose reading and writing message capability.
pub struct WrappedStream<TStream>
where
    TStream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    socket: TStream,
    out_stream: Option<BytesChannelSender>,
}

impl<TStream> WrappedStream<TStream>
where
    TStream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    fn new(socket: TStream) -> Self {
        WrappedStream {
            socket,
            out_stream: None,
        }
    }
}

impl<TStream> WrappedStream<TStream>
where
    TStream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    async fn send_message(mut self, message: MessageData) -> Result<Option<Self>, io::Error> {
        let mut size_buf = [0; 4];
        encode_msg_len(
            message.message.len(),
            message.stream.is_some(),
            &mut size_buf,
        );

        // write msg size & msg data
        self.socket.write_all(&size_buf).await?;
        self.socket.write_all(&message.message).await?;

        // if we have a stream, copy the stream to socket then drop the substream to
        // notify the end of stream
        if let Some(stream) = message.stream {
            futures::io::copy(stream, &mut self.socket).await?;
            self.socket.flush().await?;

            // closing the socket is necessary to prevent the socket from being dropped and
            // a reset control message sent to destination resulting in an error
            // reading the socket, even if it has still data to be read.
            self.socket.close().await?;

            Ok(None)
        } else {
            self.socket.flush().await?;
            Ok(Some(self))
        }
    }

    async fn read_next(mut self) -> Result<(Option<MessageData>, Self), io::Error> {
        if let Some(mut out_stream) = self.out_stream.take() {
            futures::io::copy(&mut self.socket, &mut out_stream).await?;
            Ok((None, self))
        } else {
            let msg = self.read_new_message().await?;
            Ok((Some(msg), self))
        }
    }

    async fn read_new_message(&mut self) -> Result<MessageData, io::Error> {
        let mut size_buf = [0; 4];
        self.socket.read_exact(&mut size_buf).await?;
        let (msg_len, has_stream) = decode_msg_len(&size_buf);

        let message_data = {
            if msg_len > MAX_MESSAGE_SIZE {
                warn!(
                    "Got a message on stream that exceeds maximum size. Dropping stream. ({}>{})",
                    msg_len, MAX_MESSAGE_SIZE
                );
                return Err(io::ErrorKind::InvalidData.into());
            }

            let mut message_data = vec![0; msg_len];
            self.socket.read_exact(&mut message_data).await?;
            message_data
        };

        if has_stream {
            let (sender, receiver) = super::bytes_channel::new(STREAM_BUFFER_SIZE);
            self.out_stream = Some(sender);

            Ok(MessageData {
                message: message_data.into(),
                stream: Some(Box::new(receiver)),
            })
        } else {
            Ok(MessageData {
                message: message_data.into(),
                stream: None,
            })
        }
    }
}

const STREAM_MASK: usize = 1 << 31;

/// Encodes message size and uses high bit to indicate that there is a stream
/// after message.
fn encode_msg_len(mut len: usize, has_stream: bool, into: &mut [u8]) {
    if has_stream {
        len |= STREAM_MASK;
    }

    LittleEndian::write_u32(into, len as u32);
}

/// Decodes message size.
fn decode_msg_len(bytes: &[u8]) -> (usize, bool) {
    let mut msg_len = LittleEndian::read_u32(bytes) as usize;
    let has_stream = if msg_len & STREAM_MASK == STREAM_MASK {
        msg_len &= STREAM_MASK - 1;
        true
    } else {
        false
    };

    (msg_len, has_stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_msg_len() {
        let tv = vec![
            (7, true),
            (7, false),
            (2 << 9, true),
            (2 << 9, false),
            (2 << 18, true),
            (2 << 18, false),
            (2 << 26, true),
            (2 << 26, false),
        ];

        for (len, stream) in tv {
            let mut size_buf = vec![0; 4];
            encode_msg_len(len, stream, &mut size_buf);

            let (decoded_len, decoded_stream) = decode_msg_len(&size_buf);
            assert_eq!(decoded_len, len, "{} {}", len, stream);
            assert_eq!(stream, decoded_stream, "{} {}", len, stream);
        }
    }
}
