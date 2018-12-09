use std::result::Result as StdResult;
use std::io::Error as IoError;

use crate::chars::CharsError;

#[must_use]
pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    EmptyStack,
    OutOfBounds,
    InvalidAssignArg,
    InvalidIncludeArg,
    InvalidApplyArg,
    InvalidSplitArg,
    InvalidGetArg,
    InvalidMoveArg,
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
