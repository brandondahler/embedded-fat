use crate::BiosParameterBlockError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::{ErrorType, ReadExactError};

#[derive(Clone, Debug)]
pub enum FileSystemError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    DeviceError(DE),
    InvalidBiosParameterBlock(BiosParameterBlockError),
    InvalidFatSignature,
    StreamEndReached,
    StreamError(SE),
}

impl<DE, SE> Error for FileSystemError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
}

impl<DE, SE> Display for FileSystemError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FileSystemError::DeviceError(e) => write!(f, "device error occurred: {}", e),
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

impl<DE, SE> From<BiosParameterBlockError> for FileSystemError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: BiosParameterBlockError) -> Self {
        FileSystemError::InvalidBiosParameterBlock(value)
    }
}

impl<DE, SE> From<SE> for FileSystemError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: SE) -> Self {
        FileSystemError::StreamError(value)
    }
}

impl<DE, SE> From<ReadExactError<SE>> for FileSystemError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: ReadExactError<SE>) -> Self {
        match value {
            ReadExactError::Other(stream_error) => stream_error.into(),
            ReadExactError::UnexpectedEof => Self::StreamEndReached,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    mod display {
        use super::*;
        use crate::mock::IoError;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                FileSystemError::DeviceError(IoError::default()),
                FileSystemError::InvalidFatSignature,
                FileSystemError::InvalidBiosParameterBlock(
                    BiosParameterBlockError::InvalidFatCount,
                ),
                FileSystemError::StreamEndReached,
                FileSystemError::StreamError(IoError::default()),
            ];

            for value in values {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }
}
