#![deny(bare_trait_objects)]

extern crate exocore_common;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
pub mod lp2p;
pub mod error;
pub mod layer;
pub mod messages;

#[cfg(any(test, feature = "tests_utils"))]
pub mod mock;

pub use error::Error;
pub use layer::{TransportHandle, TransportLayer};
pub use lp2p::{Libp2pTransport, Libp2pTransportHandle};
pub use messages::{InMessage, OutMessage};
