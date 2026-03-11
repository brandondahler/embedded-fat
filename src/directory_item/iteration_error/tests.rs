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
            DirectoryItemIterationError::EntryInvalid(DirectoryEntryError::ShortNameEntryInvalid(
                ShortNameDirectoryEntryError::NameInvalid(ShortFileNameError::CharacterInvalid {
                    character: 0,
                    offset: 0,
                }),
            )),
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
