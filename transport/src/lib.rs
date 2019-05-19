#![deny(bare_trait_objects)]

extern crate exocore_common;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
pub mod lp2p;
pub mod errors;
pub mod transport;
pub mod messages;

#[cfg(feature = "websocket_transport")]
pub mod ws;

#[cfg(any(test, feature = "tests_utils"))]
pub mod mock;

pub use errors::Error;
pub use transport::{TransportHandle, TransportLayer};
pub use lp2p::{Libp2pTransport, Libp2pTransportHandle};
pub use messages::{InMessage, OutMessage};
