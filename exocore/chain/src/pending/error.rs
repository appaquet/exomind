use crate::operation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation related error: {0:?}")]
    Operation(#[from] operation::Error),

    #[error("Operation cannot be found")]
    NotFound,
}
