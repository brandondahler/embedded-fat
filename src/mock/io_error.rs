use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind};

#[derive(Clone, Copy, Debug)]
pub struct IoError;

impl core::error::Error for IoError {}

impl Display for IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "MockError")
    }
}

impl Error for IoError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}
