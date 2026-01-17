use crate::allocation_table::AllocationTableError;
use crate::directory_entry::DirectoryEntryError;
use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind, ReadExactError};

#[derive(Clone, Debug)]
pub enum DirectoryEntryIterationError<DE, SE>
where
    DE: Error,
    SE: Error,
{
    AllocationTableEntryTypeUnexpected,
    EntryInvalid(DirectoryEntryError),
    DeviceError(DE),
    StreamEndReached,
    StreamError(SE),
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
            DirectoryEntryIterationError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            DirectoryEntryIterationError::StreamError(e) => {
                write!(f, "stream error occurred: {}", e)
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
            DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected => ErrorKind::Other,
            DirectoryEntryIterationError::DeviceError(device_error) => device_error.kind(),
            DirectoryEntryIterationError::EntryInvalid(_) => ErrorKind::Other,
            DirectoryEntryIterationError::StreamEndReached => ErrorKind::Other,
            DirectoryEntryIterationError::StreamError(stream_error) => stream_error.kind(),
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
            AllocationTableError::StreamEndReached => Self::StreamEndReached,
            AllocationTableError::StreamError(device_error) => Self::StreamError(device_error),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_name::ShortFileNameError;
    use crate::mock::IoError;
    use crate::{LongNameDirectoryEntryError, ShortNameDirectoryEntryError};
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

    mod kind {
        use super::*;

        #[test]
        fn allocation_table_entry_type_unexpected_is_other() {
            assert_eq!(
                DirectoryEntryIterationError::<IoError, IoError>::AllocationTableEntryTypeUnexpected.kind(),
                ErrorKind::Other
            );
        }

        #[test]
        fn device_error_inherits_value() {
            assert_eq!(
                DirectoryEntryIterationError::<IoError, IoError>::DeviceError(IoError(
                    ErrorKind::AddrInUse
                ))
                .kind(),
                ErrorKind::AddrInUse
            );
        }

        #[test]
        fn entry_invalid_is_other() {
            assert_eq!(
                DirectoryEntryIterationError::<IoError, IoError>::EntryInvalid(
                    DirectoryEntryError::LongNameEntryInvalid(
                        LongNameDirectoryEntryError::EntryNumberInvalid
                    )
                )
                .kind(),
                ErrorKind::Other
            );
        }

        #[test]
        fn stream_error_inherits_value() {
            assert_eq!(
                DirectoryEntryIterationError::<IoError, IoError>::StreamError(IoError(
                    ErrorKind::AddrInUse
                ))
                .kind(),
                ErrorKind::AddrInUse
            );
        }

        #[test]
        fn stream_end_reached_is_other() {
            assert_eq!(
                DirectoryEntryIterationError::<IoError, IoError>::StreamEndReached.kind(),
                ErrorKind::Other
            );
        }
    }
}
