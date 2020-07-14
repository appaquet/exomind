use std::collections::VecDeque;
use std::{io, iter};

use byteorder::{ByteOrder, LittleEndian};
use futures::future::BoxFuture;
use futures::prelude::*;
use futures::{AsyncReadExt, AsyncWriteExt};
use libp2p::core::UpgradeInfo;
use libp2p::core::{InboundUpgrade, OutboundUpgrade};
use libp2p::swarm::protocols_handler::{
    KeepAlive, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr, SubstreamProtocol,
};
use libp2p::swarm::NegotiatedSubstream;
use std::task::{Context, Poll};

const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB

type HandlerEvent =
    ProtocolsHandlerEvent<ExocoreProtoConfig, ExocoreProtoMessage, ExocoreProtoMessage, io::Error>;

// TODO: Remove dyn dispatched future once type_alias_impl_trait lands: https://github.com/rust-lang/rust/issues/63063
type InboundStreamFuture = BoxFuture<
    'static,
    Result<(ExocoreProtoMessage, WrappedStream<NegotiatedSubstream>), io::Error>,
>;
type OutboundStreamFuture =
    BoxFuture<'static, Result<WrappedStream<NegotiatedSubstream>, io::Error>>;

/// Protocol handler for Exocore protocol.
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
///
/// Note:
///   * Streams are not mapped 1:1 to sockets as the transport may be
///     multiplexed.
pub struct ExocoreProtoHandler {
    listen_protocol: SubstreamProtocol<ExocoreProtoConfig>,
    inbound_stream_futures: Vec<InboundStreamFuture>,
    outbound_dialing: bool,
    outbound_stream_futures: Vec<OutboundStreamFuture>,
    idle_outbound_stream: Option<WrappedStream<NegotiatedSubstream>>,
    send_queue: VecDeque<ExocoreProtoMessage>,
    keep_alive: KeepAlive,
}

impl Default for ExocoreProtoHandler {
    fn default() -> Self {
        ExocoreProtoHandler {
            listen_protocol: SubstreamProtocol::new(ExocoreProtoConfig::default()),
            inbound_stream_futures: Vec::new(),
            outbound_dialing: false,
            outbound_stream_futures: Vec::new(),
            idle_outbound_stream: None,
            send_queue: VecDeque::new(),
            keep_alive: KeepAlive::Yes,
        }
    }
}

impl ProtocolsHandler for ExocoreProtoHandler {
    type InEvent = ExocoreProtoMessage;
    type OutEvent = ExocoreProtoMessage;
    type Error = io::Error;
    type InboundProtocol = ExocoreProtoConfig;
    type OutboundProtocol = ExocoreProtoConfig;
    type OutboundOpenInfo = ExocoreProtoMessage;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol> {
        self.listen_protocol.clone()
    }

    fn inject_fully_negotiated_inbound(&mut self, substream: WrappedStream<NegotiatedSubstream>) {
        trace!("Inbound negotiated");
        self.inbound_stream_futures
            .push(Box::pin(substream.read_message()));
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        substream: WrappedStream<NegotiatedSubstream>,
        message: ExocoreProtoMessage,
    ) {
        trace!("Outbound negotiated. Sending message.");
        self.outbound_dialing = false;
        self.outbound_stream_futures
            .push(Box::pin(substream.send_message(message)));
    }

    fn inject_event(&mut self, message: ExocoreProtoMessage) {
        self.send_queue.push_back(message);
    }

    fn inject_dial_upgrade_error(
        &mut self,
        _message: ExocoreProtoMessage,
        _err: ProtocolsHandlerUpgrErr<io::Error>,
    ) {
        debug!("Upgrade error. Dropping stream.");
        self.outbound_dialing = false;
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

            let message = self.send_queue.pop_front().unwrap();
            return Poll::Ready(ProtocolsHandlerEvent::OutboundSubstreamRequest {
                protocol: self.listen_protocol.clone(),
                info: message,
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
            let futures = std::mem::replace(&mut self.outbound_stream_futures, Vec::new());
            for mut fut in futures {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(substream)) => {
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
                    Poll::Ready(Err(err)) => {
                        debug!("Error sending message: {}", err);
                        return Poll::Ready(ProtocolsHandlerEvent::Close(err));
                    }
                    Poll::Pending => {
                        self.outbound_stream_futures.push(fut);
                    }
                }
            }
        }

        // we poll all futures that reads messages to completion.
        if !self.inbound_stream_futures.is_empty() {
            let futures = std::mem::replace(&mut self.inbound_stream_futures, Vec::new());
            for mut fut in futures {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok((message, substream))) => {
                        trace!("Successfully received message");
                        self.inbound_stream_futures
                            .push(Box::pin(substream.read_message()));
                        return Poll::Ready(ProtocolsHandlerEvent::Custom(message));
                    }
                    Poll::Ready(Err(err)) => {
                        debug!("Error receiving message: {}", err);
                        return Poll::Ready(ProtocolsHandlerEvent::Close(err));
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

type UpgradeInfoData = &'static [u8];

impl UpgradeInfo for ExocoreProtoConfig {
    type Info = UpgradeInfoData;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/exocore/1.0.0")
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
        future::ok(WrappedStream { socket })
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
        future::ok(WrappedStream { socket })
    }
}

/// Wire message sent and receive over the streams managed by
/// `ExocoreProtoHandler`
#[derive(Clone)]
pub struct ExocoreProtoMessage {
    pub(crate) data: Vec<u8>,
}

/// Wraps a stream to expose reading and writing message capability.
pub struct WrappedStream<TStream>
where
    TStream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    socket: TStream,
}

impl<TStream> WrappedStream<TStream>
where
    TStream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    async fn send_message(mut self, message: ExocoreProtoMessage) -> Result<Self, io::Error> {
        let mut size_buf = [0; 4];
        LittleEndian::write_u32(&mut size_buf, message.data.len() as u32);

        self.socket.write_all(&size_buf).await?;
        self.socket.write_all(&message.data).await?;
        self.socket.flush().await?;

        Ok(self)
    }

    async fn read_message(mut self) -> Result<(ExocoreProtoMessage, Self), io::Error> {
        let mut size_buf = vec![0; 4];
        self.socket.read_exact(&mut size_buf).await?;
        let size = LittleEndian::read_u32(&size_buf) as usize;

        if size > MAX_MESSAGE_SIZE {
            warn!(
                "Got a message on stream that exceeds maximum size. Dropping stream. ({}>{})",
                size, MAX_MESSAGE_SIZE
            );
            return Err(io::ErrorKind::InvalidData.into());
        }

        let mut msg = ExocoreProtoMessage {
            data: vec![0; size],
        };
        self.socket.read_exact(&mut msg.data).await?;

        Ok((msg, self))
    }
}
