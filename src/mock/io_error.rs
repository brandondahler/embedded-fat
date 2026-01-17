use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind};

#[derive(Clone, Debug)]
pub struct IoError(pub ErrorKind);

impl Default for IoError {
    fn default() -> Self {
        IoError(ErrorKind::Other)
    }
}

impl core::error::Error for IoError {}

impl Display for IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "IoError")
    }
}

impl Error for IoError {
    fn kind(&self) -> ErrorKind {
        self.0
    }
}
