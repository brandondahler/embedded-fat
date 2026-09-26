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
            Utf16DecodeError::LowSurrogateUnexpected => {}
            Utf16DecodeError::HighSurrogateOrphaned => {}
            Utf16DecodeError::LowSurrogateMissing => {}
        }

        todo!()
    }
}

impl Error for Utf16DecodeError {}
