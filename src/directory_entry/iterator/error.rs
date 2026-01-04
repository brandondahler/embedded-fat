use crate::allocation_table::AllocationTableError;
use crate::directory_entry::DirectoryEntryError;
use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind, ReadExactError};

#[derive(Clone, Copy, Debug)]
pub enum DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    AllocationTableEntryTypeUnexpected,
    EntryInvalid(DirectoryEntryError),
    DeviceError(DE),
    StreamError(SE),
    StreamEndReached,
}

impl<DE, SE> core::error::Error for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
}

impl<DE, SE> Display for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected => {
                write!(f, "the allocation table entry was an unexpected type")
            }
            DirectoryEntryIterationError::DeviceError(e) => {
                write!(f, "device error occurred: {}", e)
            }
            DirectoryEntryIterationError::EntryInvalid(e) => {
                write!(f, "an entry was invalid: {}", e)
            }
            DirectoryEntryIterationError::StreamError(e) => {
                write!(f, "stream error occurred: {}", e)
            }
            DirectoryEntryIterationError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
        }
    }
}

impl<DE, SE> Error for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn kind(&self) -> ErrorKind {
        match self {
            DirectoryEntryIterationError::DeviceError(device_error) => device_error.kind(),
            DirectoryEntryIterationError::StreamError(stream_error) => stream_error.kind(),
            DirectoryEntryIterationError::StreamEndReached => ErrorKind::Other,
            DirectoryEntryIterationError::EntryInvalid(_) => ErrorKind::Other,
            DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected => ErrorKind::Other,
        }
    }
}

impl<DE, SE> From<SE> for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: SE) -> Self {
        Self::StreamError(value)
    }
}

impl<DE, SE> From<AllocationTableError<SE>> for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: AllocationTableError<SE>) -> Self {
        match value {
            AllocationTableError::StreamError(device_error) => Self::StreamError(device_error),
            AllocationTableError::StreamEndReached => Self::StreamEndReached,
        }
    }
}

impl<DE, SE> From<DirectoryEntryError> for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: DirectoryEntryError) -> Self {
        Self::EntryInvalid(value)
    }
}

impl<DE, SE> From<ReadExactError<SE>> for DirectoryEntryIterationError<DE, SE>
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
