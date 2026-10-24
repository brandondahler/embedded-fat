#[cfg(test)]
mod tests;

use crate::BiosParameterBlockError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::{ErrorType, ReadExactError};

#[derive(Clone, Debug)]
pub enum FileSystemError<SE>
where
    SE: embedded_io::Error,
{
    DeviceBusy,
    InvalidBiosParameterBlock(BiosParameterBlockError),
    InvalidFatSignature,
    StreamEndReached,
    StreamError(SE),
}

impl<SE> Error for FileSystemError<SE> where SE: embedded_io::Error {}

impl<SE> Display for FileSystemError<SE>
where
    SE: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FileSystemError::DeviceBusy => write!(f, "the device is currently busy"),
            FileSystemError::InvalidBiosParameterBlock(e) => {
                write!(f, "the bios parameter block is invalid: {}", e)
            }
            FileSystemError::InvalidFatSignature => {
                write!(
                    f,
                    "the FAT signature at offsets 0xFE and 0xFF were incorrect"
                )
            }
            FileSystemError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            FileSystemError::StreamError(e) => write!(f, "stream error occurred: {}", e),
        }
    }
}

impl<SE> From<BiosParameterBlockError> for FileSystemError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: BiosParameterBlockError) -> Self {
        FileSystemError::InvalidBiosParameterBlock(value)
    }
}

impl<SE> From<SE> for FileSystemError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: SE) -> Self {
        FileSystemError::StreamError(value)
    }
}

impl<SE> From<ReadExactError<SE>> for FileSystemError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: ReadExactError<SE>) -> Self {
        match value {
            ReadExactError::Other(stream_error) => stream_error.into(),
            ReadExactError::UnexpectedEof => Self::StreamEndReached,
        }
    }
}
