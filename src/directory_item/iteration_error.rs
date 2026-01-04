use crate::directory_entry::{
    DirectoryEntryError, DirectoryEntryIterationError, LongNameDirectoryEntryError,
    ShortNameDirectoryEntryError,
};
use crate::directory_item::DirectoryItemError;
use core::fmt::{Display, Formatter};
use embedded_io::{Error, ReadExactError};

#[derive(Clone, Copy, Debug)]
pub enum DirectoryItemIterationError<DE, SE> {
    AllocationTableEntryTypeUnexpected,
    DeviceError(DE),
    EntryInvalid(DirectoryEntryError),
    ItemError(DirectoryItemError),
    StreamError(SE),
    StreamEndReached,
}

impl<DE, SE> Display for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DirectoryItemIterationError::AllocationTableEntryTypeUnexpected => {
                write!(f, "the allocation table entry was an unexpected type")
            }
            DirectoryItemIterationError::DeviceError(e) => {
                write!(f, "device error occurred: {}", e)
            }
            DirectoryItemIterationError::EntryInvalid(e) => {
                write!(f, "an invalid entry was encountered: {}", e)
            }
            DirectoryItemIterationError::ItemError(e) => {
                write!(f, "an invalid item was encountered: {}", e)
            }
            DirectoryItemIterationError::StreamError(e) => {
                write!(f, "stream error occurred: {}", e)
            }
            DirectoryItemIterationError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
        }
    }
}

impl<DE, SE> core::error::Error for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
}

impl<DE, SE> From<DirectoryEntryIterationError<DE, SE>> for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: DirectoryEntryIterationError<DE, SE>) -> Self {
        match value {
            DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected => {
                Self::AllocationTableEntryTypeUnexpected
            }
            DirectoryEntryIterationError::DeviceError(e) => Self::DeviceError(e),
            DirectoryEntryIterationError::EntryInvalid(e) => Self::EntryInvalid(e),
            DirectoryEntryIterationError::StreamError(e) => Self::StreamError(e),
            DirectoryEntryIterationError::StreamEndReached => Self::StreamEndReached,
        }
    }
}

impl<DE, SE> From<DirectoryItemError> for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    fn from(value: DirectoryItemError) -> Self {
        Self::ItemError(value)
    }
}
