use exocore_protos::apps::MessageStatus;
use wasmtime::Trap;

/// Application runtime error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The application is missing function '{0}'. Did you include SDK and implement #[exocore_app]?")]
    MissingFunction(#[source] anyhow::Error, &'static str),

    #[error("WASM runtime error '{0}'")]
    Runtime(&'static str),

    #[error("WASM execution aborted: {0}")]
    Trap(#[from] Trap),

    #[error("Message handling error: status={0:?}")]
    MessageStatus(Option<MessageStatus>),

    #[error("Message decoding error: {0}")]
    MessageDecode(#[from] exocore_protos::prost::DecodeError),

    #[error("Entity store error: {0}")]
    Store(#[from] exocore_store::error::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<Error> for Trap {
    fn from(err: Error) -> Self {
        match err {
            Error::Trap(t) => t,
            other => Trap::new(other.to_string()),
        }
    }
}
