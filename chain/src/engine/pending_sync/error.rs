/// Pending Synchronization Error
#[derive(Clone, Debug, Fail)]
pub enum PendingSyncError {
    #[fail(display = "Got into an invalid synchronization state: {}", _0)]
    InvalidSyncState(String),
    #[fail(display = "Got an invalid sync request: {}", _0)]
    InvalidSyncRequest(String),
}
