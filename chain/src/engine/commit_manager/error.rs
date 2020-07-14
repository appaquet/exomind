use crate::operation::OperationId;

/// CommitManager related error
#[derive(Clone, Debug, thiserror::Error)]
pub enum CommitManagerError {
    #[error("Invalid signature in commit manager: {0}")]
    InvalidSignature(String),

    #[error("A referenced operation is missing from local store: {0}")]
    MissingOperation(OperationId),
}
