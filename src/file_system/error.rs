use crate::BiosParameterBlockError;
use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind, ErrorType, ReadExactError};

#[derive(Clone, Copy, Debug)]
pub enum FileSystemError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    DeviceError(DE),
    InvalidFatSignature,
    InvalidBiosParameterBlock(BiosParameterBlockError),
    StreamError(SE),
    StreamEndReached,
}

impl<DE, SE> core::error::Error for FileSystemError<DE, SE>
where
    DE: Error,
    SE: Error,
{
}

impl<DE, SE> Display for FileSystemError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FileSystemError::DeviceError(e) => write!(f, "device error occurred: {}", e),
            FileSystemError::InvalidFatSignature => {
                write!(
                    f,
                    "the FAT signature at offsets 0xFE and 0xFF were incorrect"
                )
            }
            FileSystemError::InvalidBiosParameterBlock(e) => {
                write!(f, "the bios parameter block is invalid: {}", e)
            }
            FileSystemError::StreamError(e) => write!(f, "stream error occurred: {}", e),
            FileSystemError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
        }
    }
}

impl<DE, SE> From<BiosParameterBlockError> for FileSystemError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: BiosParameterBlockError) -> Self {
        FileSystemError::InvalidBiosParameterBlock(value)
    }
}

impl<DE, SE> From<SE> for FileSystemError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: SE) -> Self {
        FileSystemError::StreamError(value)
    }
}

impl<DE, SE> From<ReadExactError<SE>> for FileSystemError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: ReadExactError<SE>) -> Self {
        match value {
            ReadExactError::Other(stream_error) => stream_error.into(),
            ReadExactError::UnexpectedEof => Self::StreamEndReached,
        }
    }
}
