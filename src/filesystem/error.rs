#[cfg(test)]
mod tests;

use crate::BiosParameterBlockError;
use crate::queued_resource::QueuedResourceBusyError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::{ErrorType, ReadExactError};

#[derive(Clone, Debug)]
pub enum FilesystemError<SE>
where
    SE: embedded_io::Error,
{
    FilesystemBusy,
    InvalidBiosParameterBlock(BiosParameterBlockError),
    InvalidFatSignature,
    StreamEndReached,
    StreamError(SE),
}

impl<SE> Error for FilesystemError<SE> where SE: embedded_io::Error {}

impl<SE> Display for FilesystemError<SE>
where
    SE: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FilesystemError::FilesystemBusy => write!(f, "the filesystem is currently busy"),
            FilesystemError::InvalidBiosParameterBlock(e) => {
                write!(f, "the bios parameter block is invalid: {}", e)
            }
            FilesystemError::InvalidFatSignature => {
                write!(
                    f,
                    "the FAT signature at offsets 0xFE and 0xFF were incorrect"
                )
            }
            FilesystemError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            FilesystemError::StreamError(e) => write!(f, "stream error occurred: {}", e),
        }
    }
}

impl<SE> From<BiosParameterBlockError> for FilesystemError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: BiosParameterBlockError) -> Self {
        FilesystemError::InvalidBiosParameterBlock(value)
    }
}

impl<SE> From<SE> for FilesystemError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: SE) -> Self {
        FilesystemError::StreamError(value)
    }
}

impl<SE> From<QueuedResourceBusyError> for FilesystemError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: QueuedResourceBusyError) -> Self {
        Self::FilesystemBusy
    }
}

impl<SE> From<ReadExactError<SE>> for FilesystemError<SE>
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
