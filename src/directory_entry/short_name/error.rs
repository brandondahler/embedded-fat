use crate::file_name::ShortFileNameError;
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum ShortNameDirectoryEntryError {
    NameInvalid(ShortFileNameError),
}

impl Display for ShortNameDirectoryEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
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
