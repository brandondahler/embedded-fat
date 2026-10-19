use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Utf16DecodeError {
    HighSurrogateOrphaned,
    LowSurrogateMissing,
    LowSurrogateUnexpected,
}

impl Display for Utf16DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Utf16DecodeError::LowSurrogateUnexpected => {
                write!(f, "unexpected low surrogate code unit encountered")
            }
            Utf16DecodeError::HighSurrogateOrphaned => {
                write!(f, "high surrogate code unit encountered as final code unit")
            }
            Utf16DecodeError::LowSurrogateMissing => write!(
                f,
                "expected low surrogate code unit to follow previous high surrogate code unit"
            ),
        }
    }
}

impl Error for Utf16DecodeError {}
