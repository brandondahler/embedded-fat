#[cfg(test)]
mod tests;

use crate::directory_entry::{DirectoryEntryError, DirectoryEntryIterationError};
use crate::directory_item::DirectoryItemError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::ErrorType;

#[derive(Clone, Debug)]
pub enum DirectoryItemIterationError<SE>
where
    SE: embedded_io::Error,
{
    AllocationTableEntryTypeUnexpected,
    DeviceBusy,
    EntryInvalid(DirectoryEntryError),
    ItemError(DirectoryItemError),
    StreamEndReached,
    StreamError(SE),
}

impl<SE> Display for DirectoryItemIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DirectoryItemIterationError::AllocationTableEntryTypeUnexpected => {
                write!(f, "the allocation table entry was an unexpected type")
            }
            DirectoryItemIterationError::DeviceBusy => {
                write!(f, "the device is currently busy")
            }
            DirectoryItemIterationError::EntryInvalid(e) => {
                write!(f, "an invalid entry was encountered: {}", e)
            }
            DirectoryItemIterationError::ItemError(e) => {
                write!(f, "an invalid item was encountered: {}", e)
            }
            DirectoryItemIterationError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            DirectoryItemIterationError::StreamError(e) => {
                write!(f, "stream error occurred: {}", e)
            }
        }
    }
}

impl<SE> Error for DirectoryItemIterationError<SE> where SE: embedded_io::Error {}

impl<SE> From<DirectoryEntryIterationError<SE>> for DirectoryItemIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: DirectoryEntryIterationError<SE>) -> Self {
        match value {
            DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected => {
                Self::AllocationTableEntryTypeUnexpected
            }
            DirectoryEntryIterationError::DeviceBusy => Self::DeviceBusy,
            DirectoryEntryIterationError::EntryInvalid(e) => Self::EntryInvalid(e),
            DirectoryEntryIterationError::StreamEndReached => Self::StreamEndReached,
            DirectoryEntryIterationError::StreamError(e) => Self::StreamError(e),
        }
    }
}

impl<SE> From<DirectoryItemError> for DirectoryItemIterationError<SE>
where
    SE: embedded_io::Error,
{
    fn from(value: DirectoryItemError) -> Self {
        Self::ItemError(value)
    }
}
