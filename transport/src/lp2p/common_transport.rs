///
/// Extracted from https://github.com/libp2p/rust-libp2p/blob/master/src/lib.rs to prevent
/// pulling libp2p completly
///
use futures::future::Future;
use libp2p_core::transport::TransportError;
use libp2p_core::upgrade::{InboundUpgradeExt, OutboundUpgradeExt};
use libp2p_core::{identity, Multiaddr, PeerId, Transport};
use std::time::Duration;
use std::{error, io};

pub fn build_tcp_ws_secio_mplex_yamux(
    keypair: identity::Keypair,
) -> impl Transport<
    Output = (
        PeerId,
        impl libp2p_core::muxing::StreamMuxer<
                OutboundSubstream = impl Send,
                Substream = impl Send,
                Error = impl Into<io::Error>,
            > + Send
            + Sync,
    ),
    Error = impl error::Error + Send,
    Listener = impl Send,
    Dial = impl Send,
    ListenerUpgrade = impl Send,
> + Clone {
    CommonTransport::new()
        .with_upgrade(libp2p_secio::SecioConfig::new(keypair))
        .and_then(move |output, endpoint| {
            let peer_id = output.remote_key.into_peer_id();
            let peer_id2 = peer_id.clone();
            let upgrade = libp2p_core::upgrade::SelectUpgrade::new(
                libp2p_yamux::Config::default(),
                libp2p_mplex::MplexConfig::new(),
            )
            // TODO: use a single `.map` instead of two maps
            .map_inbound(move |muxer| (peer_id, muxer))
            .map_outbound(move |muxer| (peer_id2, muxer));
            libp2p_core::upgrade::apply(output.stream, upgrade, endpoint)
                .map(|(id, muxer)| (id, libp2p_core::muxing::StreamMuxerBox::new(muxer)))
        })
        .with_timeout(Duration::from_secs(20))
}

/// Implementation of `Transport` that supports the most common protocols.
///
/// The list currently is TCP/IP, DNS, and WebSockets. However this list could change in the
/// future to get new transports.
#[derive(Debug, Clone)]
struct CommonTransport {
    // The actual implementation of everything.
    inner: CommonTransportInner,
}

type InnerImplementation = libp2p_dns::DnsConfig<libp2p_tcp::TcpConfig>;

#[derive(Debug, Clone)]
struct CommonTransportInner {
    inner: InnerImplementation,
}

impl CommonTransport {
    /// Initializes the `CommonTransport`.
    pub fn new() -> CommonTransport {
        let tcp = libp2p_tcp::TcpConfig::new().nodelay(true);
        let transport = libp2p_dns::DnsConfig::new(tcp);

        CommonTransport {
            inner: CommonTransportInner { inner: transport },
        }
    }
}

impl Transport for CommonTransport {
    type Output = <InnerImplementation as Transport>::Output;
    type Error = <InnerImplementation as Transport>::Error;
    type Listener = <InnerImplementation as Transport>::Listener;
    type ListenerUpgrade = <InnerImplementation as Transport>::ListenerUpgrade;
    type Dial = <InnerImplementation as Transport>::Dial;

    fn listen_on(self, addr: Multiaddr) -> Result<Self::Listener, TransportError<Self::Error>> {
        self.inner.inner.listen_on(addr)
    }

    fn dial(self, addr: Multiaddr) -> Result<Self::Dial, TransportError<Self::Error>> {
        self.inner.inner.dial(addr)
    }
}
