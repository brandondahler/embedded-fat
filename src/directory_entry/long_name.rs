mod error;

pub use error::*;

use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, DirectoryEntryAttributes};
use crate::encoding::Ucs2Character;
use crate::file_name::LONG_NAME_MAX_LENGTH;
use crate::utils::{read_le_u16, write_le_u16};

pub const LONG_NAME_CHARACTERS_PER_ENTRY: usize = 13;
pub const LONG_NAME_MAX_ENTRY_COUNT: u8 =
    LONG_NAME_MAX_LENGTH.div_ceil(LONG_NAME_CHARACTERS_PER_ENTRY) as u8;

#[derive(Clone, Debug)]
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
                _ => ((character_index - 11) * 2) + 28,
            };

            let ucs2_character_codepoint = read_le_u16(data, byte_index);

            *ucs2_character = Ucs2Character::from_u16(ucs2_character_codepoint).ok_or(
                LongNameDirectoryEntryError::NameCharacterInvalid {
                    character: ucs2_character_codepoint,
                    offset: character_index as u8,
                },
            )?;
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
                self.ucs2_characters[ucs2_character_index].to_u16(),
            );
        }

        bytes[11] |= DirectoryEntryAttributes::LongName.bits();
        bytes[13] = self.short_name_checksum;

        for ucs2_character_index in 5..11 {
            write_le_u16(
                bytes,
                14 + (2 * (ucs2_character_index - 5)),
                self.ucs2_characters[ucs2_character_index].to_u16(),
            );
        }

        for ucs2_character_index in 11..13 {
            write_le_u16(
                bytes,
                28 + (2 * (ucs2_character_index - 11)),
                self.ucs2_characters[ucs2_character_index].to_u16(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory_entry::DirectoryEntryAttributes;

    mod from_bytes {
        use super::*;

        #[test]
        fn parses_entry_correctly() {
            let mut test_data = TestData::valid();

            let entry = LongNameDirectoryEntry::from_bytes(&mut test_data.data)
                .expect("Ok should be returned");

            assert_eq!(
                entry.is_last_entry(),
                test_data.is_last_entry,
                "is_last_entry should parse correctly"
            );
            assert_eq!(
                entry.entry_number(),
                test_data.entry_number,
                "entry_number should be parsed correctly"
            );
            assert_eq!(
                entry.short_name_checksum(),
                test_data.short_name_checksum,
                "short_name_checksum should be parsed correctly"
            );
            assert_eq!(
                entry.ucs2_characters(),
                &test_data.name_characters,
                "ucs2_characters should be parsed correctly"
            );
        }

        #[test]
        fn entry_number_zero_returns_err() {
            let mut data = TestData::valid().data;
            data[0] = 0x00;

            let entry =
                LongNameDirectoryEntry::from_bytes(&mut data).expect_err("Err should be returned");

            assert!(
                matches!(entry, LongNameDirectoryEntryError::EntryNumberInvalid),
                "EntryNumberInvalid should be returned"
            );
        }

        #[test]
        fn entry_number_too_large_returns_err() {
            let mut data = TestData::valid().data;
            data[0] = 0x3F;

            let error =
                LongNameDirectoryEntry::from_bytes(&mut data).expect_err("Err should be returned");

            assert!(
                matches!(error, LongNameDirectoryEntryError::EntryNumberInvalid),
                "EntryNumberInvalid should be returned"
            );
        }

        #[test]
        fn character_invalid_returns_err() {
            let mut data = TestData::valid().data;
            data[3] = 0x00;
            data[4] = 0xD8;

            let error =
                LongNameDirectoryEntry::from_bytes(&mut data).expect_err("Err should be returned");

            assert!(matches!(
                error,
                LongNameDirectoryEntryError::NameCharacterInvalid {
                    character: 0xD800,
                    offset: 1
                }
            ));
        }
    }

    mod write {
        use super::*;

        #[test]
        fn roundtrips_correctly() {
            let data = TestData::valid().data;
            let entry = LongNameDirectoryEntry::from_bytes(&data).expect("Ok should be returned");

            let mut result = [0x00; DIRECTORY_ENTRY_SIZE];
            entry.write(&mut result);

            assert_eq!(result, data, "Input and output bytes should match exactly");
        }
    }

    struct TestData {
        data: [u8; DIRECTORY_ENTRY_SIZE],

        is_last_entry: bool,
        entry_number: u8,
        short_name_checksum: u8,
        name_characters: [Ucs2Character; LONG_NAME_CHARACTERS_PER_ENTRY],
    }

    impl TestData {
        fn valid() -> Self {
            Self {
                #[rustfmt::skip]
                data: [
                    // Order byte
                    0x41,

                    // Name stride 1
                    0x66, 0x00,
                    0x6F, 0x00,
                    0x6F, 0x00,
                    0x6B, 0x71,
                    0x36, 0x21,

                    // Attributes
                    DirectoryEntryAttributes::LongName.bits(),

                    // Reserved
                    0x00,

                    // Short name checksum
                    0x12,

                    // Name stride 2
                    0xCC, 0x18,
                    0x92, 0x5F,
                    0x99, 0xB2,
                    0xB3, 0xD4,
                    0x33, 0x60,
                    0x0C, 0xC3,

                    // Reserved
                    0x00,
                    0x00,

                    // Name stride 3
                    0x00, 0x00,
                    0xFF, 0xFF,
                ],

                is_last_entry: true,
                entry_number: 1,
                short_name_checksum: 0x12,
                name_characters: [
                    0x0066, 0x006F, 0x006F, 0x716B, 0x2136, 0x18CC, 0x5F92, 0xB299, 0xD4B3, 0x6033,
                    0xC30C, 0x0000, 0xFFFF,
                ]
                .map(|codepoint| Ucs2Character::from_u16(codepoint).unwrap()),
            }
        }
    }
}
