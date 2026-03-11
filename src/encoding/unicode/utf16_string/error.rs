#[cfg(test)]
mod tests;

use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Utf16StringError {
    UnpairedSurrogateEncountered { index: usize },
}

impl Display for Utf16StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Utf16StringError::UnpairedSurrogateEncountered { index } => {
                write!(f, "unpaired surrogate encountered at index {}", index)
            }
        }
    }
}

impl Error for Utf16StringError {}
