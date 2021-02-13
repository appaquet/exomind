use std::sync::Arc;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error of kind: {0}")]
    IO(#[from] Arc<std::io::Error>),

    #[error("Destination buffer too small (needed={0} actual={1})")]
    DestinationTooSmall(usize, usize),

    #[error("Source buffer too small (needed={0} actual={1})")]
    SourceTooSmall(usize, usize),

    #[error("Invalid offset subtraction ({0} - {1} < 0)")]
    OffsetSubtract(usize, usize),

    #[error("Capnp serialization error: {0}")]
    Capnp(#[from] exocore_protos::capnp::Error),

    #[error("Multihash error: {0:?}")]
    Multihash(#[from] Arc<multihash::Error>),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(Arc::new(err))
    }
}
impl From<multihash::Error> for Error {
    fn from(err: multihash::Error) -> Self {
        Error::Multihash(Arc::new(err))
    }
}
