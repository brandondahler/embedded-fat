mod error;

#[cfg(test)]
mod tests;

use bon::Builder;
pub use error::*;

use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, DirectoryEntryAttributes};
use crate::encoding::Utf16CodeUnit;
use crate::file_name::LONG_NAME_MAX_LENGTH;
use crate::utils::{read_le_u16, write_le_u16};

pub const LONG_NAME_CHARACTERS_PER_ENTRY: usize = 13;
pub const LONG_NAME_MAX_ENTRY_COUNT: u8 =
    LONG_NAME_MAX_LENGTH.div_ceil(LONG_NAME_CHARACTERS_PER_ENTRY) as u8;

#[derive(Builder, Clone, Debug)]
pub struct LongNameDirectoryEntry {
    order_byte: u8,

    utf16_code_units: [Utf16CodeUnit; LONG_NAME_CHARACTERS_PER_ENTRY],
    short_name_checksum: u8,
}

impl LongNameDirectoryEntry {
    pub fn from_bytes(
        bytes: &[u8; DIRECTORY_ENTRY_SIZE],
    ) -> Result<LongNameDirectoryEntry, LongNameDirectoryEntryError> {
        ensure!(
            matches!(bytes[0] & 0x3F, 1..=LONG_NAME_MAX_ENTRY_COUNT),
            LongNameDirectoryEntryError::EntryNumberInvalid
        );

        let mut utf16_code_units = [0; LONG_NAME_CHARACTERS_PER_ENTRY];
        for (character_index, utf16_code_unit) in utf16_code_units.iter_mut().enumerate() {
            let byte_index = match character_index {
                0..5 => (character_index * 2) + 1,
                5..11 => ((character_index - 5) * 2) + 14,
                _ => ((character_index - 11) * 2) + 28,
            };

            *utf16_code_unit = read_le_u16(bytes, byte_index);
        }

        Ok(Self {
            order_byte: bytes[0],

            utf16_code_units,
            short_name_checksum: bytes[13],
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

    pub fn utf16_code_units(&self) -> &[Utf16CodeUnit; LONG_NAME_CHARACTERS_PER_ENTRY] {
        &self.utf16_code_units
    }

    pub fn write(&self, mut bytes: &mut [u8; DIRECTORY_ENTRY_SIZE]) {
        bytes[0] = self.order_byte;

        for utf16_code_unit_index in 0..5 {
            write_le_u16(
                bytes,
                1 + (2 * utf16_code_unit_index),
                self.utf16_code_units[utf16_code_unit_index],
            );
        }

        bytes[11] |= DirectoryEntryAttributes::LongName.bits();
        bytes[13] = self.short_name_checksum;

        for utf16_code_unit_index in 5..11 {
            write_le_u16(
                bytes,
                14 + (2 * (utf16_code_unit_index - 5)),
                self.utf16_code_units[utf16_code_unit_index],
            );
        }

        for utf16_code_unit_index in 11..13 {
            write_le_u16(
                bytes,
                28 + (2 * (utf16_code_unit_index - 11)),
                self.utf16_code_units[utf16_code_unit_index],
            );
        }
    }
}
