use crate::directory_entry::{
    DIRECTORY_ENTRY_SIZE, DirectoryEntryError, ShortNameDirectoryEntryError,
};
use crate::encoding::Ucs2Character;
use crate::file_name::LONG_NAME_MAX_LENGTH;
use crate::utils::{read_le_u16, write_le_u16};
use core::error::Error;
use core::fmt::{Display, Formatter};

pub const LONG_NAME_CHARACTERS_PER_ENTRY: usize = 13;
pub const LONG_NAME_MAX_ENTRY_COUNT: u8 =
    LONG_NAME_MAX_LENGTH.div_ceil(LONG_NAME_CHARACTERS_PER_ENTRY) as u8;

pub struct LongNameDirectoryEntry {
    order_byte: u8,

    ucs2_characters: [Ucs2Character; LONG_NAME_CHARACTERS_PER_ENTRY],
    short_name_checksum: u8,
}

impl LongNameDirectoryEntry {
    pub fn from_bytes(
        data: &[u8; DIRECTORY_ENTRY_SIZE],
    ) -> Result<LongNameDirectoryEntry, LongNameDirectoryEntryError> {
        ensure!(
            matches!(data[0] & 0x3F, 1..=LONG_NAME_MAX_ENTRY_COUNT),
            LongNameDirectoryEntryError::EntryNumberInvalid
        );

        let mut ucs2_characters = [Ucs2Character::null(); LONG_NAME_CHARACTERS_PER_ENTRY];
        for (character_index, ucs2_character) in ucs2_characters.iter_mut().enumerate() {
            let byte_index = match character_index {
                0..5 => (character_index * 2) + 1,
                5..11 => ((character_index - 5) * 2) + 14,
                11..13 => ((character_index - 11) * 2) + 28,
                _ => unreachable!(),
            };

            let ucs2_character_codepoint = read_le_u16(data, byte_index);

            *ucs2_character =
                Ucs2Character::from_u16(ucs2_character_codepoint).ok_or_else(|| {
                    LongNameDirectoryEntryNameError::new(
                        ucs2_character_codepoint,
                        character_index as u8,
                    )
                })?;
        }

        Ok(Self {
            order_byte: data[0],

            ucs2_characters,
            short_name_checksum: data[13],
        })
    }

    pub fn is_last_entry(&self) -> bool {
        self.order_byte & 0x40 > 0
    }

    pub fn entry_number(&self) -> u8 {
        self.order_byte & 0x3F
    }

    pub fn short_name_checksum(&self) -> u8 {
        self.short_name_checksum
    }

    pub fn ucs2_characters(&self) -> &[Ucs2Character; LONG_NAME_CHARACTERS_PER_ENTRY] {
        &self.ucs2_characters
    }

    pub fn write(&self, mut bytes: &mut [u8; DIRECTORY_ENTRY_SIZE]) {
        bytes[0] = self.order_byte;

        for ucs2_character_index in 0..5 {
            write_le_u16(
                bytes,
                1 + (2 * ucs2_character_index),
                self.ucs2_characters[ucs2_character_index].into(),
            );
        }

        bytes[13] = self.short_name_checksum;

        for ucs2_character_index in 5..11 {
            write_le_u16(
                bytes,
                14 + (2 * (ucs2_character_index - 5)),
                self.ucs2_characters[ucs2_character_index].into(),
            );
        }

        for ucs2_character_index in 11..13 {
            write_le_u16(
                bytes,
                28 + (2 * (ucs2_character_index - 11)),
                self.ucs2_characters[ucs2_character_index].into(),
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LongNameDirectoryEntryError {
    EntryNumberInvalid,
    NameInvalid(LongNameDirectoryEntryNameError),
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
            LongNameDirectoryEntryError::NameInvalid(e) => {
                write!(f, "the long name directory entry's name is not valid: {e}")
            }
        }
    }
}

impl Error for LongNameDirectoryEntryError {}

impl From<LongNameDirectoryEntryNameError> for LongNameDirectoryEntryError {
    fn from(value: LongNameDirectoryEntryNameError) -> Self {
        Self::NameInvalid(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LongNameDirectoryEntryNameError {
    ucs2_character: u16,
    offset: u8,
}

impl LongNameDirectoryEntryNameError {
    pub fn new(ucs2_character: u16, offset: u8) -> Self {
        Self {
            ucs2_character,
            offset,
        }
    }

    pub fn ucs2_character(&self) -> u16 {
        self.ucs2_character
    }

    pub fn offset(&self) -> u8 {
        self.offset
    }
}

impl Display for LongNameDirectoryEntryNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "the invalid ucs2 character \\u{:04X} was encountered at offset {}",
            self.ucs2_character, self.offset
        )
    }
}

impl Error for LongNameDirectoryEntryNameError {}
