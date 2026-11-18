#[cfg(test)]
mod tests;

use crate::allocation_table::AllocationTableReadError;
use crate::directory_entry::DirectoryEntryError;
use crate::queued_resource::QueuedResourceBusyError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::ReadExactError;

#[derive(Clone, Debug)]
pub enum DirectoryEntryIterationError<SE>
where
    SE: embedded_io::Error,
{
    AllocationTableEntryTypeUnexpected,
    EntryInvalid(DirectoryEntryError),
    FilesystemBusy,
    StreamEndReached,
    StreamError(SE),
}

impl<SE> Error for DirectoryEntryIterationError<SE> where SE: embedded_io::Error {}

impl<SE> Display for DirectoryEntryIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected => {
                write!(f, "the allocation table entry was an unexpected type")
            }
            DirectoryEntryIterationError::EntryInvalid(e) => {
                write!(f, "an entry was invalid: {}", e)
            }
            DirectoryEntryIterationError::FilesystemBusy => {
                write!(f, "the filesystem is current busy")
            }
            DirectoryEntryIterationError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            DirectoryEntryIterationError::StreamError(e) => {
                write!(f, "stream error occurred: {}", e)
            }
        }
    }
}

impl<SE> From<SE> for DirectoryEntryIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: SE) -> Self {
        Self::StreamError(value)
    }
}

impl<SE> From<AllocationTableReadError<SE>> for DirectoryEntryIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: AllocationTableReadError<SE>) -> Self {
        match value {
            AllocationTableReadError::StreamEndReached => Self::StreamEndReached,
            AllocationTableReadError::StreamError(stream_error) => Self::StreamError(stream_error),
        }
    }
}

impl<SE> From<DirectoryEntryError> for DirectoryEntryIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: DirectoryEntryError) -> Self {
        Self::EntryInvalid(value)
    }
}

impl<SE> From<QueuedResourceBusyError> for DirectoryEntryIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: QueuedResourceBusyError) -> Self {
        Self::FilesystemBusy
    }
}

impl<SE> From<ReadExactError<SE>> for DirectoryEntryIterationError<SE>
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
