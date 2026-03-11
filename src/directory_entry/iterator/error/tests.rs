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
            DirectoryEntryIterationError::EntryInvalid(DirectoryEntryError::ShortNameEntryInvalid(
                ShortNameDirectoryEntryError::NameInvalid(ShortFileNameError::CharacterInvalid {
                    character: 0x41,
                    offset: 0,
                }),
            )),
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
