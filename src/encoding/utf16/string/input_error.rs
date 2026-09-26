#[cfg(test)]
mod tests;

use crate::encoding::Utf16EncodeError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use core::str::EncodeUtf16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Utf16StringInputError {
    TooLong,
}

impl Display for Utf16StringInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Utf16StringInputError::TooLong => {
                write!(f, "input string requires too many UTF-16 code units")
            }
        }
    }
}

impl Error for Utf16StringInputError {}

impl From<Utf16EncodeError> for Utf16StringInputError {
    fn from(value: Utf16EncodeError) -> Self {
        match value {
            Utf16EncodeError::OutputTooSmall => Utf16StringInputError::TooLong,
        }
    }
}
