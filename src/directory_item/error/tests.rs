use super::*;
use alloc::string::ToString;
use strum::IntoEnumIterator;

mod display {
    use super::*;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            DirectoryItemError::LongNameCorrupted,
            DirectoryItemError::LongNameEntryNumberWrong,
            DirectoryItemError::LongNameEmpty,
            DirectoryItemError::LongNameFirstEntryInvalid,
            DirectoryItemError::LongNameInvalid(LongFileNameError::UnpairedSurrogateEncountered {
                offset: 0,
            }),
            DirectoryItemError::LongNameOrphaned,
            DirectoryItemError::LongNameShortNameChecksumInconsistent,
            DirectoryItemError::LongNameTooLong,
            DirectoryItemError::ShortNameChecksumMismatch,
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
