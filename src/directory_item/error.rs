use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum DirectoryItemError {
    LongNameCorrupted,
    LongNameEntryNumberWrong,
    LongNameEmpty,
    LongNameFirstEntryInvalid,
    LongNameOrphaned,
    LongNameShortNameChecksumInconsistent,
    LongNameTooLong,
    ShortNameChecksumMismatch,
}

impl Display for DirectoryItemError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DirectoryItemError::LongNameCorrupted => {
                write!(
                    f,
                    "the long name directory entry contained a NULL character but had invalid padding characters"
                )
            }
            DirectoryItemError::LongNameEmpty => {
                write!(
                    f,
                    "the long name directory entry does not contain any characters"
                )
            }
            DirectoryItemError::LongNameFirstEntryInvalid => write!(
                f,
                "the first long name directory entry was missing the is_last_entry flag"
            ),
            DirectoryItemError::LongNameOrphaned => {
                write!(
                    f,
                    "the long name directory entry chain was not followed by a short directory entry"
                )
            }
            DirectoryItemError::LongNameEntryNumberWrong => {
                write!(
                    f,
                    "the latest long name directory entry in the chain has an unexpected order value"
                )
            }
            DirectoryItemError::LongNameTooLong => {
                write!(
                    f,
                    "the long name directory entry chain is longer than allowed"
                )
            }
            DirectoryItemError::LongNameShortNameChecksumInconsistent => {
                write!(
                    f,
                    "the latest long name directory entry in the chain has a different checksum from the previous entry"
                )
            }
            DirectoryItemError::ShortNameChecksumMismatch => {
                write!(
                    f,
                    "the long name directory entry chain has a checksum that does not match the following short directory entry"
                )
            }
        }
    }
}

impl Error for DirectoryItemError {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use strum::IntoEnumIterator;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            for value in DirectoryItemError::iter() {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }
}
