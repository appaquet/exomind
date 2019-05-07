#![deny(bare_trait_objects)]

extern crate exocore_common;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
pub mod lp2p;
mod completion;
pub mod layer;
pub mod messages;

#[cfg(any(test, feature = "tests_utils"))]
pub mod mock;

pub use layer::{LayerStreams, Layer};
pub use messages::{InMessage, OutMessage};

///
/// Transport related error
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "libp2p transport error: {:?}", _0)]
    TransportError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl<Terr> From<libp2p::TransportError<Terr>> for Error
where
    Terr: std::error::Error + Send + Sync + 'static,
{
    fn from(err: libp2p::TransportError<Terr>) -> Self {
        Error::TransportError(Box::new(err))
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}
