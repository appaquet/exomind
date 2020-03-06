use crate::operation::OperationId;

/// CommitManager related error
#[derive(Clone, Debug, Fail)]
pub enum CommitManagerError {
    #[fail(display = "Invalid signature in commit manager: {}", _0)]
    InvalidSignature(String),
    #[fail(display = "A referenced operation is missing from local store: {}", _0)]
    MissingOperation(OperationId),
}
