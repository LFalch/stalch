use std::result::Result as StdResult;
use std::io::{Error as IoError, CharsError};

#[must_use]
pub type Result<T> = StdResult<T, Error>;

pub enum Error {
    EmptyStack,
    OutOfBounds,
    InvalidAssignArg,
    InvalidIncludeArg,
    InvalidApplyArg,
    InvalidGrabArg,
    NoBlockStarted,
    IoError(IoError),
    CharsError(CharsError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::IoError(e)
    }
}
