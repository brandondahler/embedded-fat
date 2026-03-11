use super::*;
use crate::file_name::ShortFileNameError;
use alloc::string::ToString;

mod display {
    use super::*;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            DirectoryEntryError::ShortNameEntryInvalid(ShortNameDirectoryEntryError::NameInvalid(
                ShortFileNameError::CharacterInvalid {
                    character: 0x41,
                    offset: 0,
                },
            )),
            DirectoryEntryError::LongNameEntryInvalid(
                LongNameDirectoryEntryError::EntryNumberInvalid,
            ),
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
