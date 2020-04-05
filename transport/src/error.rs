use exocore_core::capnp;

#[cfg(any(feature = "lp2p"))]
use std::sync::Arc;

/// Transport related error
#[derive(Debug, Fail, Clone)]
pub enum Error {
    #[cfg(feature = "lp2p")]
    #[fail(display = "libp2p transport error: {:?}", _0)]
    Libp2pTransport(Arc<dyn std::error::Error + Send + Sync + 'static>),

    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),

    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),

    #[fail(display = "IO error: {}", _0)]
    IO(String),

    #[fail(display = "Could not upgrade a weak reference")]
    Upgrade,

    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,

    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

#[cfg(feature = "lp2p")]
impl<Terr> From<libp2p::core::transport::TransportError<Terr>> for Error
where
    Terr: std::error::Error + Send + Sync + 'static,
{
    fn from(err: libp2p::core::transport::TransportError<Terr>) -> Self {
        Error::Libp2pTransport(Arc::new(err))
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err.to_string())
    }
}
