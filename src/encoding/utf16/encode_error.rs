use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Utf16EncodeError {
    OutputTooSmall,
}

impl Display for Utf16EncodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Utf16EncodeError::OutputTooSmall => {
                write!(
                    f,
                    "the output buffer did not contain sufficient space to store the requested characters"
                )
            }
        }
    }
}

impl Error for Utf16EncodeError {}
