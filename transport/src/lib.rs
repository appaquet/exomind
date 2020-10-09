#![deny(bare_trait_objects)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod error;
pub mod messages;
pub mod streams;
pub mod transport;

#[cfg(feature = "p2p-base")]
pub mod p2p;

#[cfg(feature = "p2p-base")]
pub use p2p::{Libp2pTransport, Libp2pTransportServiceHandle};

#[cfg(feature = "http-server")]
pub mod http;

#[cfg(any(feature = "http-server", feature = "p2p-full"))]
pub mod either;

#[cfg(any(test, feature = "tests-utils"))]
pub mod testing;

pub use error::Error;
pub use messages::{InMessage, OutMessage};
pub use transport::{InEvent, OutEvent, ServiceType, TransportServiceHandle};
