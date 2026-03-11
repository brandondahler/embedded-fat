#[cfg(test)]
mod tests;

use crate::allocation_table::AllocationTableError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::{ErrorKind, ReadExactError};

#[derive(Clone, Debug)]
pub enum FileError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    DeviceError(DE),
    SeekPositionBeyondLimits(u64),
    SeekPositionImpossible(i64),
    StreamEndReached,
    StreamError(SE),
    UnexpectedAllocationTableEntryEncountered,
}

impl<DE, SE> Error for FileError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
}

impl<DE, SE> Display for FileError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            FileError::DeviceError(e) => write!(f, "device error occurred: {}", e),
            FileError::SeekPositionBeyondLimits(desired_address) => write!(
                f,
                "seek position provided results in address beyond allowed limits: {}",
                desired_address
            ),
            FileError::SeekPositionImpossible(desired_address) => write!(
                f,
                "seek position provided results in an invalid address {}",
                desired_address
            ),
            FileError::StreamEndReached => write!(f, "stream end was reached when not expected"),
            FileError::StreamError(e) => write!(f, "stream error occurred: {}", e),
            FileError::UnexpectedAllocationTableEntryEncountered => write!(
                f,
                "an unexpected allocation table entry type was encountered"
            ),
        }
    }
}

impl<DE, SE> embedded_io::Error for FileError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn kind(&self) -> ErrorKind {
        match self {
            FileError::StreamError(error) => error.kind(),
            _ => ErrorKind::Other,
        }
    }
}

impl<DE, SE> From<SE> for FileError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: SE) -> Self {
        Self::StreamError(value)
    }
}

impl<DE, SE> From<ReadExactError<SE>> for FileError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: ReadExactError<SE>) -> Self {
        match value {
            ReadExactError::Other(stream_error) => stream_error.into(),
            ReadExactError::UnexpectedEof => FileError::StreamEndReached,
        }
    }
}

impl<DE, SE> From<AllocationTableError<SE>> for FileError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: AllocationTableError<SE>) -> Self {
        match value {
            AllocationTableError::StreamEndReached => FileError::StreamEndReached,
            AllocationTableError::StreamError(stream_error) => stream_error.into(),
        }
    }
}
