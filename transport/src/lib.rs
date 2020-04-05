#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod error;
pub mod messages;
pub mod transport;

#[cfg(feature = "lp2p")]
#[macro_use]
pub mod lp2p;

#[cfg(feature = "lp2p")]
pub use lp2p::{Libp2pTransport, Libp2pTransportHandle};

#[cfg(any(test, feature = "tests_utils"))]
pub mod mock;

#[cfg(any(test, feature = "lp2p"))]
pub mod either;

pub use error::Error;
pub use messages::{InMessage, OutMessage};
pub use transport::{InEvent, OutEvent, TransportHandle, TransportLayer};
