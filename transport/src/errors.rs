use exocore_common::serialization::capnp;

#[cfg(any(feature = "libp2p_transport", feature = "websocket_transport"))]
use std::sync::Arc;

/// Transport related error
#[derive(Debug, Fail, Clone)]
pub enum Error {
    #[cfg(feature = "libp2p_transport")]
    #[fail(display = "libp2p transport error: {:?}", _0)]
    Libp2pTransport(Arc<dyn std::error::Error + Send + Sync + 'static>),

    #[cfg(feature = "websocket_transport")]
    #[fail(display = "Websocket transport error: {:?}", _0)]
    WebsocketTransport(Arc<crate::ws::WebSocketError>),

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

#[cfg(feature = "libp2p_transport")]
impl<Terr> From<libp2p_core::transport::TransportError<Terr>> for Error
where
    Terr: std::error::Error + Send + Sync + 'static,
{
    fn from(err: libp2p_core::transport::TransportError<Terr>) -> Self {
        Error::Libp2pTransport(Arc::new(err))
    }
}

#[cfg(feature = "websocket_transport")]
impl From<crate::ws::WebSocketError> for Error {
    fn from(err: crate::ws::WebSocketError) -> Self {
        Error::WebsocketTransport(Arc::new(err))
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
