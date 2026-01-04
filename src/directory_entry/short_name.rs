use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, DirectoryEntryAttributes, DirectoryEntryError};
use crate::file_name::ShortFileName;
use crate::utils::{read_le_u16, read_le_u32, write_le_u16, write_le_u32};
use core::error::Error;
use core::fmt::{Display, Formatter};

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

        // Only validate the bytes of the name when it isn't a free directory entry
        if !matches!(name_bytes[0], 0x00 | 0xE5) {
            for (index, character) in name_bytes.iter().enumerate() {
                let is_valid_character = match character {
                    0x00..=0x04
                    | 0x06..=0x1F
                    | 0x22
                    | 0x2A..=0x2C
                    | 0x2F
                    | 0x3A..=0x3F
                    | 0x5B..=0x5D
                    | 0x7C => false,
                    0x05 => index == 0,
                    0x20 => index != 0,
                    0xE5 => index != 0,
                    _ => true,
                };

                ensure!(
                    is_valid_character,
                    ShortNameDirectoryEntryNameError::new(*character, index as u8)
                );
            }
        }

        let attributes = DirectoryEntryAttributes::from_bits_retain(data[11]);

        Ok(Self {
            name: ShortFileName::new(name_bytes),
            attributes,

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
        self.name.write(bytes);
        bytes[11] = self.attributes.bits();
        write_le_u16(bytes, 20, (self.first_cluster_number >> 16) as u16);
        write_le_u16(bytes, 26, self.first_cluster_number as u16);
        write_le_u32(bytes, 28, self.file_size);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ShortNameDirectoryEntryError {
    NameInvalid(ShortNameDirectoryEntryNameError),
}

impl Display for ShortNameDirectoryEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ShortNameDirectoryEntryError::NameInvalid(e) => {
                write!(f, "the short name directory entry's name is invalid: {}", e)
            }
        }
    }
}

impl Error for ShortNameDirectoryEntryError {}

impl From<ShortNameDirectoryEntryNameError> for ShortNameDirectoryEntryError {
    fn from(value: ShortNameDirectoryEntryNameError) -> Self {
        Self::NameInvalid(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShortNameDirectoryEntryNameError {
    character: u8,
    offset: u8,
}

impl ShortNameDirectoryEntryNameError {
    pub fn new(character: u8, offset: u8) -> Self {
        Self { character, offset }
    }

    pub fn character(&self) -> u8 {
        self.character
    }

    pub fn offset(&self) -> u8 {
        self.offset
    }
}

impl Display for ShortNameDirectoryEntryNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "the invalid character 0x{:02X} was encountered at offset {}",
            self.character, self.offset
        )
    }
}

impl Error for ShortNameDirectoryEntryNameError {}
