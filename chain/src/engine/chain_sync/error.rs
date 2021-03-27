/// Chain synchronizer specific error
#[derive(Debug, thiserror::Error)]
pub enum ChainSyncError {
    #[error("Got an invalid sync request: {0}")]
    InvalidSyncRequest(#[source] anyhow::Error),

    #[error("Got an invalid sync response: {0}")]
    InvalidSyncResponse(#[source] anyhow::Error),

    #[error("Our local chain has diverged from leader node: {0}")]
    Diverged(#[source] anyhow::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ChainSyncError {
    pub fn is_fatal(&self) -> bool {
        matches!(self, ChainSyncError::Diverged(_))
    }
}
