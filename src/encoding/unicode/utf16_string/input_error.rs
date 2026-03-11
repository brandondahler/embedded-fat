#[cfg(test)]
mod tests;

use core::error::Error;
use core::fmt::{Display, Formatter};

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
