/// Pending Synchronization Error
#[derive(Clone, Debug, thiserror::Error)]
pub enum PendingSyncError {
    #[error("Got into an invalid synchronization state: {0}")]
    InvalidSyncState(String),

    #[error("Got an invalid sync request: {0}")]
    InvalidSyncRequest(String),
}
