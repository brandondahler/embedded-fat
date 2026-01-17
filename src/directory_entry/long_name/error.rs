use crate::directory_entry::LONG_NAME_MAX_ENTRY_COUNT;
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum LongNameDirectoryEntryError {
    EntryNumberInvalid,
    NameCharacterInvalid { character: u16, offset: u8 },
}

impl Display for LongNameDirectoryEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LongNameDirectoryEntryError::EntryNumberInvalid => {
                write!(
                    f,
                    "entry number must be between 1 and {LONG_NAME_MAX_ENTRY_COUNT}"
                )
            }
            LongNameDirectoryEntryError::NameCharacterInvalid { character, offset } => {
                write!(
                    f,
                    "the long name directory entry's name has the invalid character 0x{character:04X} at offset {offset}"
                )
            }
        }
    }
}

impl Error for LongNameDirectoryEntryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                LongNameDirectoryEntryError::EntryNumberInvalid,
                LongNameDirectoryEntryError::NameCharacterInvalid {
                    character: 0x0000,
                    offset: 0,
                },
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
