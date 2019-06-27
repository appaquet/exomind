use crate::operation;

#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Operation related error: {:?}", _0)]
    Operation(#[fail(source)] operation::Error),
    #[fail(display = "Operation cannot be found")]
    NotFound,
}

impl From<operation::Error> for Error {
    fn from(err: operation::Error) -> Self {
        Error::Operation(err)
    }
}
