mod error;

#[cfg(test)]
mod tests;

pub use error::*;

use crate::AllocationTableKind;
use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, DirectoryEntryAttributes};
use crate::file_name::ShortFileName;
use crate::utils::{read_le_u16, read_le_u32, write_le_u16, write_le_u32};
use bon::Builder;

pub const SHORT_NAME_CHARACTER_COUNT: usize = 11;

#[derive(Builder, Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct ShortNameDirectoryEntry {
    name: ShortFileName,

    attributes: DirectoryEntryAttributes,

    first_cluster_number: u32,
    file_size: u32,
}

impl ShortNameDirectoryEntry {
    pub fn from_bytes(
        bytes: &[u8; DIRECTORY_ENTRY_SIZE],
    ) -> Result<Self, ShortNameDirectoryEntryError> {
        let mut name_bytes = [0; SHORT_NAME_CHARACTER_COUNT];
        name_bytes.copy_from_slice(&bytes[0..SHORT_NAME_CHARACTER_COUNT]);

        if name_bytes[0] == 0x05 {
            name_bytes[0] = 0xE5;
        }

        let first_cluster_number =
            (read_le_u16(bytes, 20) as u32) << 16 | read_le_u16(bytes, 26) as u32;
        let file_size = read_le_u32(bytes, 28);

        ensure!(
            file_size > 0 || first_cluster_number != 0,
            ShortNameDirectoryEntryError::FirstClusterNumberInvalid
        );
        ensure!(
            first_cluster_number != 1,
            ShortNameDirectoryEntryError::FirstClusterNumberInvalid
        );

        Ok(Self {
            name: ShortFileName::new(name_bytes)?,
            attributes: DirectoryEntryAttributes::from_bits_retain(bytes[11]),

            first_cluster_number,
            file_size,
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
