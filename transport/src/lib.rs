#![deny(bare_trait_objects)]

extern crate exocore_common;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

pub mod errors;
pub mod messages;
pub mod transport;

#[cfg(feature = "libp2p_transport")]
#[macro_use]
pub mod lp2p;

#[cfg(feature = "libp2p_transport")]
pub use lp2p::{Libp2pTransport, Libp2pTransportHandle};

#[cfg(feature = "websocket_transport")]
pub mod ws;

#[cfg(any(test, feature = "tests_utils"))]
pub mod mock;

pub use errors::Error;
pub use messages::{InMessage, OutMessage};
pub use transport::{TransportHandle, TransportLayer};
