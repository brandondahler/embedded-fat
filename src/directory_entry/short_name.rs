mod error;

pub use error::*;

use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, DirectoryEntryAttributes};
use crate::file_name::ShortFileName;
use crate::utils::{read_le_u16, read_le_u32, write_le_u16, write_le_u32};
use core::error::Error;
use core::fmt::Display;

pub const SHORT_NAME_CHARACTER_COUNT: usize = 11;

#[derive(Clone, Debug)]
pub struct ShortNameDirectoryEntry {
    name: ShortFileName,

    attributes: DirectoryEntryAttributes,

    first_cluster_number: u32,
    file_size: u32,
}

impl ShortNameDirectoryEntry {
    pub fn from_bytes(
        data: &[u8; DIRECTORY_ENTRY_SIZE],
    ) -> Result<Self, ShortNameDirectoryEntryError> {
        let mut name_bytes = [0; SHORT_NAME_CHARACTER_COUNT];
        name_bytes.copy_from_slice(&data[0..SHORT_NAME_CHARACTER_COUNT]);

        if name_bytes[0] == 0x05 {
            name_bytes[0] = 0xE5;
        }

        Ok(Self {
            name: ShortFileName::new(name_bytes)?,
            attributes: DirectoryEntryAttributes::from_bits_retain(data[11]),

            first_cluster_number: (read_le_u16(data, 20) as u32) << 16
                | read_le_u16(data, 26) as u32,
            file_size: read_le_u32(data, 28),
        })
    }

    pub fn name(&self) -> &ShortFileName {
        &self.name
    }

    pub fn is_directory(&self) -> bool {
        self.attributes
            .contains(DirectoryEntryAttributes::Subdirectory)
    }

    pub fn first_cluster_number(&self) -> u32 {
        self.first_cluster_number
    }

    pub fn file_size(&self) -> u32 {
        self.file_size
    }

    pub fn write(&self, mut bytes: &mut [u8; DIRECTORY_ENTRY_SIZE]) {
        bytes[0..11].copy_from_slice(self.name.bytes());

        if bytes[0] == 0xE5 {
            bytes[0] = 0x05;
        }

        bytes[11] = self.attributes.bits();
        write_le_u16(bytes, 20, (self.first_cluster_number >> 16) as u16);
        write_le_u16(bytes, 26, self.first_cluster_number as u16);
        write_le_u32(bytes, 28, self.file_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AsciiOnlyEncoder;

    mod from_bytes {
        use super::*;

        #[test]
        fn parses_entry_correctly() {
            let mut test_data = TestData::valid();

            let entry = ShortNameDirectoryEntry::from_bytes(&mut test_data.data)
                .expect("Ok should be returned");

            assert_eq!(entry.name(), &test_data.name, "name should parse correctly");
            assert_eq!(
                entry.is_directory(),
                test_data.is_directory,
                "is_directory should be parsed correctly"
            );
            assert_eq!(
                entry.first_cluster_number(),
                test_data.first_cluster_number,
                "first_cluster_number should be parsed correctly"
            );
            assert_eq!(
                entry.file_size(),
                test_data.file_size,
                "file_size should be parsed correctly"
            );
        }

        #[test]
        fn initial_byte_05_parsed_correctly() {
            let mut data = TestData::valid().data;
            data[0] = 0x05;

            let entry =
                ShortNameDirectoryEntry::from_bytes(&mut data).expect("Ok should be returned");

            assert_eq!(
                entry.name().bytes()[0],
                0xE5,
                "First byte of name should be 0xE5"
            );
        }
    }

    mod write {
        use super::*;

        #[test]
        fn roundtrips_correctly() {
            let data = TestData::valid().data;
            let entry = ShortNameDirectoryEntry::from_bytes(&data).expect("Ok should be returned");

            let mut result = [0x00; DIRECTORY_ENTRY_SIZE];
            entry.write(&mut result);

            assert_eq!(result, data, "Input and output bytes should match exactly");
        }

        #[test]
        fn initial_byte_05_roundtrips_correctly() {
            let mut data = TestData::valid().data;
            data[0] = 0x05;

            let entry = ShortNameDirectoryEntry::from_bytes(&data).expect("Ok should be returned");

            let mut result = [0x00; DIRECTORY_ENTRY_SIZE];
            entry.write(&mut result);

            assert_eq!(result, data, "Input and output bytes should match exactly");
        }
    }

    struct TestData {
        data: [u8; DIRECTORY_ENTRY_SIZE],

        name: ShortFileName,
        is_directory: bool,
        first_cluster_number: u32,
        file_size: u32,
    }

    impl TestData {
        fn valid() -> Self {
            Self {
                #[rustfmt::skip]
                data: [
                    // Name
                    0x46, 0x4F, 0x4F, 0x42, 0x41, 0x52, 0x20, 0x20,
                    0x54, 0x58, 0x54,

                    // Attributes
                    DirectoryEntryAttributes::Subdirectory.bits(),

                    // Reserved
                    0x00,

                    // Unparsed timestamps
                    0x00,
                    0x00, 0x00,
                    0x00, 0x00,
                    0x00, 0x00,

                    // First cluster high
                    0x34, 0x12,

                    // Unparsed timestamps
                    0x00, 0x00,
                    0x00, 0x00,

                    // First cluster low
                    0x78, 0x56,

                    // File Size
                    0xF1, 0xDE, 0xBC, 0x9A,
                ],

                name: ShortFileName::from_str(&AsciiOnlyEncoder, "foobar.txt").unwrap(),
                is_directory: true,
                first_cluster_number: 0x12345678,
                file_size: 0x9ABCDEF1,
            }
        }
    }
}
