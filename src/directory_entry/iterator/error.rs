use crate::allocation_table::AllocationTableError;
use crate::directory_entry::DirectoryEntryError;
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::ReadExactError;

#[derive(Clone, Debug)]
pub enum DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    AllocationTableEntryTypeUnexpected,
    EntryInvalid(DirectoryEntryError),
    DeviceError(DE),
    StreamEndReached,
    StreamError(SE),
}

impl<DE, SE> Error for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
}

impl<DE, SE> Display for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
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
            DirectoryEntryIterationError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            DirectoryEntryIterationError::StreamError(e) => {
                write!(f, "stream error occurred: {}", e)
            }
        }
    }
}

impl<DE, SE> From<SE> for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: SE) -> Self {
        Self::StreamError(value)
    }
}

impl<DE, SE> From<AllocationTableError<SE>> for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: AllocationTableError<SE>) -> Self {
        match value {
            AllocationTableError::StreamEndReached => Self::StreamEndReached,
            AllocationTableError::StreamError(device_error) => Self::StreamError(device_error),
        }
    }
}

impl<DE, SE> From<DirectoryEntryError> for DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: embedded_io::Error,
{
    fn from(value: DirectoryEntryError) -> Self {
        Self::EntryInvalid(value)
    }
}

impl<DE, SE> From<ReadExactError<SE>> for DirectoryEntryIterationError<DE, SE>
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
    use crate::ShortNameDirectoryEntryError;
    use crate::file_name::ShortFileNameError;
    use crate::mock::IoError;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected,
                DirectoryEntryIterationError::EntryInvalid(
                    DirectoryEntryError::ShortNameEntryInvalid(
                        ShortNameDirectoryEntryError::NameInvalid(
                            ShortFileNameError::CharacterInvalid {
                                character: 0x41,
                                offset: 0,
                            },
                        ),
                    ),
                ),
                DirectoryEntryIterationError::DeviceError(IoError::default()),
                DirectoryEntryIterationError::StreamEndReached,
                DirectoryEntryIterationError::StreamError(IoError::default()),
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
