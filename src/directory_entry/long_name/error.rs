use crate::directory_entry::LONG_NAME_MAX_ENTRY_COUNT;
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LongNameDirectoryEntryError {
    EntryNumberInvalid,
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
            let values = [LongNameDirectoryEntryError::EntryNumberInvalid];

            for value in values {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }
}
