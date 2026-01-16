use crate::directory_entry::{LongNameDirectoryEntryError, ShortNameDirectoryEntryError};
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum DirectoryEntryError {
    ShortNameEntryInvalid(ShortNameDirectoryEntryError),
    LongNameEntryInvalid(LongNameDirectoryEntryError),
}

impl Display for DirectoryEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DirectoryEntryError::ShortNameEntryInvalid(e) => {
                write!(f, "the short name directory entry is invalid: {}", e)
            }
            DirectoryEntryError::LongNameEntryInvalid(e) => {
                write!(f, "the long name directory entry is invalid: {}", e)
            }
        }
    }
}

impl Error for DirectoryEntryError {}

impl From<ShortNameDirectoryEntryError> for DirectoryEntryError {
    fn from(value: ShortNameDirectoryEntryError) -> Self {
        Self::ShortNameEntryInvalid(value)
    }
}

impl From<LongNameDirectoryEntryError> for DirectoryEntryError {
    fn from(value: LongNameDirectoryEntryError) -> Self {
        Self::LongNameEntryInvalid(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_name::ShortFileNameError;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                DirectoryEntryError::ShortNameEntryInvalid(
                    ShortNameDirectoryEntryError::NameInvalid(
                        ShortFileNameError::CharacterInvalid {
                            character: 0x41,
                            offset: 0,
                        },
                    ),
                ),
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
}
