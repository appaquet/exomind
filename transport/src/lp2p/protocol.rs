use libp2p_core::upgrade;
use libp2p_core::{InboundUpgrade, OutboundUpgrade};
use libp2p_core::{Negotiated, UpgradeInfo};

use std::{io, iter};
use tokio::io::{AsyncRead, AsyncWrite};

static PACKET_MAX_SIZE: usize = 10 * 1024 * 1024; // 1MB

/// Exocore protocol configuration. This exposes the information about protocol, and how
/// message are serialized / deserialized.
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

type InboundUpgradeFuture<TSocket> = upgrade::ReadOneThen<
    upgrade::Negotiated<TSocket>,
    (),
    fn(Vec<u8>, ()) -> Result<WireMessage, WireMessageError>,
>;

impl<TSocket> InboundUpgrade<TSocket> for ExocoreProtoConfig
where
    TSocket: AsyncRead + AsyncWrite,
{
    type Output = WireMessage;
    type Error = WireMessageError;
    type Future = InboundUpgradeFuture<TSocket>;

    #[inline]
    fn upgrade_inbound(self, socket: Negotiated<TSocket>, _info: UpgradeInfoData) -> Self::Future {
        upgrade::read_one_then(socket, PACKET_MAX_SIZE, (), |packet, ()| {
            Ok(WireMessage { data: packet })
        })
    }
}

/// Message transmitted over the wire.
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
    TSocket: AsyncWrite,
{
    type Output = ();
    type Error = io::Error;
    type Future = upgrade::WriteOne<upgrade::Negotiated<TSocket>>;

    #[inline]
    fn upgrade_outbound(self, socket: upgrade::Negotiated<TSocket>, _: Self::Info) -> Self::Future {
        upgrade::write_one(socket, self.data)
    }
}

/// Error related to serialization / deserialization of wire message.
#[derive(Debug)]
pub enum WireMessageError {
    ReadError(upgrade::ReadOneError),
}

impl From<upgrade::ReadOneError> for WireMessageError {
    #[inline]
    fn from(err: upgrade::ReadOneError) -> Self {
        WireMessageError::ReadError(err)
    }
}
