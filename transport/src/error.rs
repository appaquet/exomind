/// Transport related error
#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[cfg(feature = "p2p-base")]
    #[error("libp2p transport error: {0:?}")]
    Libp2pTransport(#[from] std::sync::Arc<dyn std::error::Error + Send + Sync + 'static>),

    #[error("Error in capnp serialization: {0}")]
    Serialization(#[from] exocore_core::capnp::Error),

    #[error("Field is not in capnp schema: code={0}")]
    SerializationNotInSchema(u16),

    #[error("IO error: {0}")]
    IO(#[from] std::sync::Arc<std::io::Error>),

    #[error("Could not upgrade a weak reference")]
    Upgrade,

    #[error("Try to lock a mutex that was poisoned")]
    Poisoned,

    #[error("An error occurred: {0}")]
    Other(String),
}

#[cfg(feature = "p2p-base")]
impl<Terr> From<libp2p::core::transport::TransportError<Terr>> for Error
where
    Terr: std::error::Error + Send + Sync + 'static,
{
    fn from(err: libp2p::core::transport::TransportError<Terr>) -> Self {
        Error::Libp2pTransport(std::sync::Arc::new(err))
    }
}

impl From<exocore_core::capnp::NotInSchema> for Error {
    fn from(err: exocore_core::capnp::NotInSchema) -> Self {
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
        Error::IO(std::sync::Arc::new(err))
    }
}
