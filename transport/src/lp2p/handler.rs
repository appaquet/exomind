use std::collections::VecDeque;
use std::{io, iter};

use byteorder::{ByteOrder, LittleEndian};
use failure::_core::task::{Context, Poll};
use futures::future::BoxFuture;
use futures::prelude::*;
use futures::{AsyncReadExt, AsyncWriteExt};
use libp2p::core::UpgradeInfo;
use libp2p::core::{InboundUpgrade, OutboundUpgrade};
use libp2p::swarm::protocols_handler::{
    KeepAlive, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr, SubstreamProtocol,
};
use libp2p::swarm::NegotiatedSubstream;

///
///
///
pub struct ProtoHandler {
    listen_protocol: SubstreamProtocol<ProtoConfig>,
    inbound_stream_futures: Vec<
        BoxFuture<'static, Result<(ProtoMessage, WrappedSocket<NegotiatedSubstream>), io::Error>>,
    >,
    outbound_dialing: bool,
    outbound_stream_futures:
        Vec<BoxFuture<'static, Result<WrappedSocket<NegotiatedSubstream>, io::Error>>>,
    idle_outbound_stream: Option<WrappedSocket<NegotiatedSubstream>>,
    send_queue: VecDeque<ProtoMessage>,
    keep_alive: KeepAlive,
}

impl Default for ProtoHandler {
    fn default() -> Self {
        ProtoHandler {
            listen_protocol: SubstreamProtocol::new(ProtoConfig::default()),
            inbound_stream_futures: Vec::new(),
            outbound_dialing: false,
            outbound_stream_futures: Vec::new(),
            idle_outbound_stream: None,
            send_queue: VecDeque::new(),
            keep_alive: KeepAlive::Yes,
        }
    }
}

impl ProtocolsHandler for ProtoHandler {
    type InEvent = ProtoMessage;
    type OutEvent = ProtoMessage;
    type Error = io::Error;
    type InboundProtocol = ProtoConfig;
    type OutboundProtocol = ProtoConfig;
    type OutboundOpenInfo = ProtoMessage;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol> {
        self.listen_protocol.clone()
    }

    fn inject_fully_negotiated_inbound(&mut self, substream: WrappedSocket<NegotiatedSubstream>) {
        trace!("Inbound negotiated");
        self.inbound_stream_futures
            .push(Box::pin(substream.read_message()));
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        substream: WrappedSocket<NegotiatedSubstream>,
        message: ProtoMessage,
    ) {
        trace!("Outbound negotiated. Sending message.");
        self.outbound_dialing = false;
        self.outbound_stream_futures
            .push(Box::pin(substream.send_message(message)));
    }

    fn inject_event(&mut self, message: ProtoMessage) {
        self.send_queue.push_back(message);
    }

    fn inject_dial_upgrade_error(
        &mut self,
        _message: ProtoMessage,
        _err: ProtocolsHandlerUpgrErr<io::Error>,
    ) {
        self.outbound_dialing = false;
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    fn poll(
        &mut self,
        cx: &mut Context,
    ) -> Poll<
        ProtocolsHandlerEvent<
            Self::OutboundProtocol,
            Self::OutboundOpenInfo,
            Self::OutEvent,
            Self::Error,
        >,
    > {
        // we have a message to send, but no outbound stream available. we ask for one.
        if !self.send_queue.is_empty()
            && self.idle_outbound_stream.is_none()
            && self.outbound_stream_futures.is_empty()
            && !self.outbound_dialing
        {
            trace!("Asking to open outbound stream");
            self.outbound_dialing = true;
            let message = self.send_queue.pop_front().unwrap();
            return Poll::Ready(ProtocolsHandlerEvent::OutboundSubstreamRequest {
                protocol: self.listen_protocol.clone(),
                info: message,
            });
        }

        if self.idle_outbound_stream.is_some() && !self.send_queue.is_empty() {
            trace!("Sending message to idle output stream");
            let message = self.send_queue.pop_front().unwrap();
            let stream = self.idle_outbound_stream.take().unwrap();
            self.outbound_stream_futures
                .push(Box::pin(stream.send_message(message)));
        }

        if !self.outbound_stream_futures.is_empty() {
            let futures = std::mem::replace(&mut self.outbound_stream_futures, Vec::new());
            for mut fut in futures {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(substream)) => {
                        if self.idle_outbound_stream.is_some() {
                            trace!("Successfully sent message. One stream already opening / ongoing. Closing this one");
                        } else if let Some(message) = self.send_queue.pop_front() {
                            trace!("Successfully sent message. Sending a new one");
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
                        self.keep_alive = KeepAlive::No;
                        // TODO: Gosip doesn't ask to close??
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

///
///
///
#[derive(Clone, Default)]
pub struct ProtoConfig;

type UpgradeInfoData = &'static [u8];

impl UpgradeInfo for ProtoConfig {
    type Info = UpgradeInfoData;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/exocore/1.0.0")
    }
}

impl<TSocket> InboundUpgrade<TSocket> for ProtoConfig
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = WrappedSocket<TSocket>;
    type Error = io::Error;
    type Future = future::Ready<Result<WrappedSocket<TSocket>, io::Error>>;

    fn upgrade_inbound(self, socket: TSocket, _: Self::Info) -> Self::Future {
        future::ok(WrappedSocket { socket })
    }
}

impl<TSocket> OutboundUpgrade<TSocket> for ProtoConfig
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = WrappedSocket<TSocket>;
    type Error = io::Error;
    type Future = future::Ready<Result<WrappedSocket<TSocket>, io::Error>>;

    #[inline]
    fn upgrade_outbound(self, socket: TSocket, _: Self::Info) -> Self::Future {
        future::ok(WrappedSocket { socket })
    }
}

///
///
///
pub struct ProtoMessage {
    pub(crate) data: Vec<u8>,
}

///
///
///
pub struct WrappedSocket<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    socket: TSocket,
}

impl<TSocket> WrappedSocket<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    async fn send_message(mut self, message: ProtoMessage) -> Result<Self, io::Error> {
        // TODO: Use SizedFrame and should be in one shot

        let mut size_buf = [0; 4];
        LittleEndian::write_u32(&mut size_buf, message.data.len() as u32);

        self.socket.write_all(&size_buf).await?;
        self.socket.write_all(&message.data).await?;
        self.socket.flush().await?;
        Ok(self)
    }

    async fn read_message(mut self) -> Result<(ProtoMessage, Self), io::Error> {
        let mut size_buf = vec![0; 4];
        self.socket.read_exact(&mut size_buf).await?;
        let size = LittleEndian::read_u32(&size_buf) as usize;

        // TODO: Should limit size
        let mut msg = ProtoMessage {
            data: vec![0; size],
        };
        self.socket.read_exact(&mut msg.data).await?;

        Ok((msg, self))
    }
}

///
///
///
#[derive(Debug)]
pub enum ProtoError {
    IO(std::io::Error),
}

impl From<std::io::Error> for ProtoError {
    #[inline]
    fn from(err: std::io::Error) -> Self {
        ProtoError::IO(err)
    }
}
