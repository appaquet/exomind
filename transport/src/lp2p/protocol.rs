use futures::future::BoxFuture;
use futures::prelude::*;
use futures::{AsyncReadExt, AsyncWriteExt, FutureExt};
use libp2p::core::UpgradeInfo;
use libp2p::core::{InboundUpgrade, OutboundUpgrade};
use std::{io, iter};

/// Represents the Exocore protocol upgrade handle. It receives an incoming `WireMessage` once the underlying
/// socket has been upgraded.
#[derive(Clone, Default)]
pub struct ExocoreProtocol;

type UpgradeInfoData = &'static [u8];

impl UpgradeInfo for ExocoreProtocol {
    type Info = UpgradeInfoData;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/exocore/1.0.0")
    }
}

impl<TSocket> InboundUpgrade<TSocket> for ExocoreProtocol
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = WireMessage;
    type Error = WireMessageError;
    type Future = BoxFuture<'static, Result<WireMessage, WireMessageError>>;

    fn upgrade_inbound(self, mut socket: TSocket, _: Self::Info) -> Self::Future {
        async move {
            let mut buf = Vec::new();
            socket
                .read_to_end(&mut buf)
                .await
                .map_err(WireMessageError::IO)?;
            Ok(WireMessage { data: buf })
        }
        .boxed()
    }
}

/// Message transmitted over the wire.
///
/// Also implements `OutputUpgrade` to handle transmission of the message once the underlying socket
/// has been upgraded.
pub struct WireMessage {
    pub(crate) data: Vec<u8>,
}

impl UpgradeInfo for WireMessage {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    #[inline]
    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/exocore/1.0.0")
    }
}

impl<TSocket> OutboundUpgrade<TSocket> for WireMessage
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = ();
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<(), io::Error>>;

    #[inline]
    fn upgrade_outbound(self, mut socket: TSocket, _: Self::Info) -> Self::Future {
        async move {
            socket.write_all(&self.data).await?;
            socket.close().await?;
            Ok(())
        }
        .boxed()
    }
}

/// Error related to serialization / deserialization of wire message.
#[derive(Debug)]
pub enum WireMessageError {
    IO(std::io::Error),
}

impl From<std::io::Error> for WireMessageError {
    #[inline]
    fn from(err: std::io::Error) -> Self {
        WireMessageError::IO(err)
    }
}
