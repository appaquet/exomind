/// Chain synchronizer specific error
#[derive(Clone, Debug, Fail)]
pub enum ChainSyncError {
    #[fail(display = "Got an invalid sync request: {}", _0)]
    InvalidSyncRequest(String),
    #[fail(display = "Got an invalid sync response: {}", _0)]
    InvalidSyncResponse(String),
    #[fail(display = "Our local chain has diverged from leader node: {}", _0)]
    Diverged(String),
    #[fail(display = "Got an error: {}", _0)]
    Other(String),
}

impl ChainSyncError {
    pub fn is_fatal(&self) -> bool {
        match *self {
            ChainSyncError::Diverged(_) => true,
            _ => false,
        }
    }
}
