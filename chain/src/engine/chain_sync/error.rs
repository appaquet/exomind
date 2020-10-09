/// Chain synchronizer specific error
#[derive(Clone, Debug, thiserror::Error)]
pub enum ChainSyncError {
    #[error("Got an invalid sync request: {0}")]
    InvalidSyncRequest(String),

    #[error("Got an invalid sync response: {0}")]
    InvalidSyncResponse(String),

    #[error("Our local chain has diverged from leader node: {0}")]
    Diverged(String),

    #[error("Got an error: {0}")]
    Other(String),
}

impl ChainSyncError {
    pub fn is_fatal(&self) -> bool {
        matches!(self, ChainSyncError::Diverged(_))
    }
}
