use crate::file_name::ShortFileNameError;
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum ShortNameDirectoryEntryError {
    FirstClusterNumberInvalid,
    FileSizeInvalid,
    NameInvalid(ShortFileNameError),
}

impl Display for ShortNameDirectoryEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ShortNameDirectoryEntryError::FileSizeInvalid => {
                write!(
                    f,
                    "the file size must be zero when the first cluster number is zero"
                )
            }
            ShortNameDirectoryEntryError::FirstClusterNumberInvalid => {
                write!(f, "the first cluster number value must not be 1")
            }
            ShortNameDirectoryEntryError::NameInvalid(error) => {
                write!(
                    f,
                    "the short name directory entry's name is invalid: {error}"
                )
            }
        }
    }
}

impl Error for ShortNameDirectoryEntryError {}

impl From<ShortFileNameError> for ShortNameDirectoryEntryError {
    fn from(value: ShortFileNameError) -> Self {
        ShortNameDirectoryEntryError::NameInvalid(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [ShortNameDirectoryEntryError::NameInvalid(
                ShortFileNameError::CharacterInvalid {
                    character: 0,
                    offset: 0,
                },
            )];

            for value in values {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }
}
