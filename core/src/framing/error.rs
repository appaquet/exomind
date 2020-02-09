#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error of kind {:?}: {}", _0, _1)]
    IO(std::io::ErrorKind, String),
    #[fail(display = "Destination buffer too small (needed={} actual={})", _0, _1)]
    DestinationTooSmall(usize, usize),
    #[fail(display = "Source buffer too small (needed={} actual={})", _0, _1)]
    SourceTooSmall(usize, usize),
    #[fail(display = "Invalid offset subtraction ({} - {} < 0)", _0, _1)]
    OffsetSubtract(usize, usize),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err.kind(), err.to_string())
    }
}
