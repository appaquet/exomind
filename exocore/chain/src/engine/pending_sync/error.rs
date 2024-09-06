/// Pending Synchronization Error
#[derive(Debug, thiserror::Error)]
pub enum PendingSyncError {
    #[error("Got into an invalid synchronization state: {0}")]
    InvalidSyncState(#[source] anyhow::Error),

    #[error("Got an invalid sync request: {0}")]
    InvalidSyncRequest(#[source] anyhow::Error),
}
