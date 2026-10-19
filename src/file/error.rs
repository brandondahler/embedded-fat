#[cfg(test)]
mod tests;

use crate::allocation_table::AllocationTableReadError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::{ErrorKind, ReadExactError};

#[derive(Clone, Debug)]
pub enum FileError<SE>
where
    SE: embedded_io::Error,
{
    SeekPositionBeyondLimits(u64),
    SeekPositionImpossible(i64),
    StreamEndReached,
    StreamError(SE),
    UnexpectedAllocationTableEntryEncountered,
}

impl<SE> Error for FileError<SE> where SE: embedded_io::Error {}

impl<SE> Display for FileError<SE>
where
    SE: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
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

impl<SE> embedded_io::Error for FileError<SE>
where
    SE: embedded_io::Error,
{
    fn kind(&self) -> ErrorKind {
        match self {
            FileError::StreamError(error) => error.kind(),
            _ => ErrorKind::Other,
        }
    }
}

impl<SE> From<SE> for FileError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: SE) -> Self {
        Self::StreamError(value)
    }
}

impl<SE> From<ReadExactError<SE>> for FileError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: ReadExactError<SE>) -> Self {
        match value {
            ReadExactError::Other(stream_error) => stream_error.into(),
            ReadExactError::UnexpectedEof => FileError::StreamEndReached,
        }
    }
}

impl<SE> From<AllocationTableReadError<SE>> for FileError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: AllocationTableReadError<SE>) -> Self {
        match value {
            AllocationTableReadError::StreamEndReached => FileError::StreamEndReached,
            AllocationTableReadError::StreamError(stream_error) => stream_error.into(),
        }
    }
}
