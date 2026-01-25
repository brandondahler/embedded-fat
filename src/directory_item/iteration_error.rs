use crate::Device;
use crate::directory_entry::{DirectoryEntryError, DirectoryEntryIterationError};
use crate::directory_item::DirectoryItemError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::ErrorType;

pub type DeviceDirectoryItemIterationError<D> =
    DirectoryItemIterationError<<D as Device>::Error, <<D as Device>::Stream as ErrorType>::Error>;

#[derive(Clone, Debug)]
pub enum DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    AllocationTableEntryTypeUnexpected,
    DeviceError(DE),
    EntryInvalid(DirectoryEntryError),
    ItemError(DirectoryItemError),
    StreamEndReached,
    StreamError(SE),
}

impl<DE, SE> Display for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
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
            DirectoryItemIterationError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            DirectoryItemIterationError::StreamError(e) => {
                write!(f, "stream error occurred: {}", e)
            }
        }
    }
}

impl<DE, SE> Error for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
}

impl<DE, SE> From<DirectoryEntryIterationError<DE, SE>> for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: DirectoryEntryIterationError<DE, SE>) -> Self {
        match value {
            DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected => {
                Self::AllocationTableEntryTypeUnexpected
            }
            DirectoryEntryIterationError::DeviceError(e) => Self::DeviceError(e),
            DirectoryEntryIterationError::EntryInvalid(e) => Self::EntryInvalid(e),
            DirectoryEntryIterationError::StreamEndReached => Self::StreamEndReached,
            DirectoryEntryIterationError::StreamError(e) => Self::StreamError(e),
        }
    }
}

impl<DE, SE> From<DirectoryItemError> for DirectoryItemIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: DirectoryItemError) -> Self {
        Self::ItemError(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShortNameDirectoryEntryError;
    use crate::file_name::ShortFileNameError;
    use crate::mock::IoError;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                DirectoryItemIterationError::AllocationTableEntryTypeUnexpected,
                DirectoryItemIterationError::DeviceError(IoError::default()),
                DirectoryItemIterationError::EntryInvalid(
                    DirectoryEntryError::ShortNameEntryInvalid(
                        ShortNameDirectoryEntryError::NameInvalid(
                            ShortFileNameError::CharacterInvalid {
                                character: 0,
                                offset: 0,
                            },
                        ),
                    ),
                ),
                DirectoryItemIterationError::ItemError(DirectoryItemError::LongNameCorrupted),
                DirectoryItemIterationError::StreamEndReached,
                DirectoryItemIterationError::StreamError(IoError::default()),
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
