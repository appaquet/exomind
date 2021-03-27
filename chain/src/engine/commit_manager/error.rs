use crate::operation::OperationId;

/// CommitManager related error
#[derive(Debug, thiserror::Error)]
pub enum CommitManagerError {
    #[error("A referenced operation is missing from local store: {0}")]
    MissingOperation(OperationId),
}
