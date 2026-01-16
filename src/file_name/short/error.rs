use crate::ShortNameDirectoryEntryError;
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum ShortFileNameError {
    CharacterInvalid { character: u8, offset: u8 },
}

impl Display for ShortFileNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ShortFileNameError::CharacterInvalid { character, offset } => {
                write!(
                    f,
                    "the invalid character 0x{character:02X} was found at offset {offset}"
                )
            }
        }
    }
}

impl Error for ShortFileNameError {}
