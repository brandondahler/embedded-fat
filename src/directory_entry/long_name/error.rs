#[cfg(test)]
mod tests;

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
