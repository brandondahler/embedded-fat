mod error;

pub use error::*;

use crate::allocation_table::AllocationTableKind;
use crate::directory_entry::DIRECTORY_ENTRY_SIZE;
use crate::utils::{read_le_u16, read_le_u32, write_le_u16, write_le_u32};
use core::fmt::Display;

#[derive(Clone, Debug)]
pub struct BiosParameterBlock {
    allocation_table_kind: AllocationTableKind,

    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    fat_count: u8,
    root_directory_entry_count: u16,
    total_sector_count: u32,
    sectors_per_fat: u32,

    active_allocation_table_index: u8,
    allocation_table_mirroring_enabled: bool,
    root_directory_file_cluster_number: Option<u32>,
    fs_info_sector_number: Option<u16>,
    backup_boot_sector_number: Option<u16>,
}

impl BiosParameterBlock {
    pub fn from_boot_sector(bytes: &[u8; 512]) -> Result<Self, BiosParameterBlockError> {
        let bytes_per_sector = read_le_u16(bytes, 11);
        ensure!(
            matches!(bytes_per_sector, 512 | 1024 | 2048 | 4096),
            BiosParameterBlockError::BytesPerSectorInvalid
        );

        let sectors_per_cluster = bytes[13];
        ensure!(
            matches!(sectors_per_cluster, 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128),
            BiosParameterBlockError::SectorsPerClusterInvalid
        );

        let reserved_sector_count = read_le_u16(bytes, 14);
        ensure!(
            reserved_sector_count != 0,
            BiosParameterBlockError::ReservedSectorCountInvalid
        );

        let fat_count = bytes[16];
        ensure!(fat_count != 0, BiosParameterBlockError::FatCountInvalid);

        let root_directory_entry_count = read_le_u16(bytes, 17);
        let total_sector_count_16bit = read_le_u16(bytes, 19);
        ensure!(
            matches!(bytes[21], 0xF0 | 0xF8..=0xFF),
            BiosParameterBlockError::MediaTypeInvalid
        );

        let sectors_per_fat_16bit = read_le_u16(bytes, 22);

        let sectors_per_fat = if sectors_per_fat_16bit > 0 {
            sectors_per_fat_16bit as u32
        } else {
            let sectors_per_fat_32bit = read_le_u32(bytes, 36);
            ensure!(
                sectors_per_fat_32bit != 0,
                BiosParameterBlockError::SectorsPerFatNotSet
            );

            sectors_per_fat_32bit
        };

        let total_sector_count = if total_sector_count_16bit > 0 {
            total_sector_count_16bit as u32
        } else {
            let total_sector_count_32bit = read_le_u32(bytes, 32);
            ensure!(
                total_sector_count_32bit != 0,
                BiosParameterBlockError::TotalSectorCountNotSet
            );

            total_sector_count_32bit
        };

        let root_directory_sectors = (root_directory_entry_count * 32).div_ceil(bytes_per_sector);
        let data_sectors = total_sector_count
            - (reserved_sector_count as u32
                + (fat_count as u32 * sectors_per_fat)
                + root_directory_sectors as u32);
        let allocation_table_kind =
            AllocationTableKind::new(data_sectors / sectors_per_cluster as u32);

        let mut active_allocation_table_index = 0;
        let mut allocation_table_mirroring_enabled = true;
        let mut root_directory_file_cluster_number: Option<u32> = None;
        let mut fs_info_sector_number: Option<u16> = None;
        let mut backup_boot_sector_number: Option<u16> = None;

        if matches!(allocation_table_kind, AllocationTableKind::Fat32) {
            ensure!(
                root_directory_entry_count == 0,
                BiosParameterBlockError::RootDirectoryEntryCountInvalid
            );
            ensure!(
                total_sector_count_16bit == 0,
                BiosParameterBlockError::TotalSectorCount16BitInvalid
            );
            ensure!(
                sectors_per_fat_16bit == 0,
                BiosParameterBlockError::SectorsPerFat16BitInvalid
            );

            let ext_flags = read_le_u16(bytes, 40);
            active_allocation_table_index = (ext_flags & 0b111) as u8;
            allocation_table_mirroring_enabled = ext_flags & (1 << 7) > 0;

            ensure!(
                bytes[42] == 0 && bytes[43] == 0,
                BiosParameterBlockError::FilesystemVersionUnsupported
            );

            root_directory_file_cluster_number = Some({
                let value = read_le_u32(bytes, 44);
                ensure!(
                    value >= 2,
                    BiosParameterBlockError::RootDirectoryFileClusterNumberInvalid
                );

                value
            });

            fs_info_sector_number = Some({
                let value = read_le_u16(bytes, 48);
                ensure!(
                    value >= 1,
                    BiosParameterBlockError::FsInfoSectorNumberInvalid
                );

                value
            });

            backup_boot_sector_number = {
                let value = read_le_u16(bytes, 50);

                if value > 0 { Some(value) } else { None }
            };
        } else {
            ensure!(
                root_directory_entry_count > 0,
                BiosParameterBlockError::RootDirectoryEntryCountInvalid
            );
        }

        Ok(Self {
            allocation_table_kind,

            bytes_per_sector,
            sectors_per_cluster,
            reserved_sector_count,
            fat_count,
            root_directory_entry_count,
            total_sector_count,
            sectors_per_fat,

            active_allocation_table_index,
            allocation_table_mirroring_enabled,
            root_directory_file_cluster_number,
            fs_info_sector_number,
            backup_boot_sector_number,
        })
    }

    pub fn active_allocation_table_index(&self) -> u8 {
        self.active_allocation_table_index
    }

    pub fn allocation_table_kind(&self) -> AllocationTableKind {
        self.allocation_table_kind
    }

    pub fn allocation_table_mirroring_enabled(&self) -> bool {
        self.allocation_table_mirroring_enabled
    }

    pub fn allocation_table_base_address(&self) -> u64 {
        self.bytes_per_sector as u64 * self.reserved_sector_count as u64
    }

    pub fn allocation_table_size(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_fat
    }

    pub fn bytes_per_cluster(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_cluster as u32
    }

    pub fn directory_table_base_address(&self) -> u64 {
        self.allocation_table_base_address()
            + (self.allocation_table_size() as u64 * self.fat_count as u64)
    }

    pub fn directory_table_entry_count(&self) -> u16 {
        self.root_directory_entry_count
    }

    pub fn directory_table_size(&self) -> u32 {
        self.root_directory_entry_count as u32 * DIRECTORY_ENTRY_SIZE as u32
    }

    pub fn data_region_base_address(&self) -> u64 {
        self.directory_table_base_address() + self.directory_table_size() as u64
    }

    pub fn root_directory_file_cluster_number(&self) -> u32 {
        // TODO: Make this option and have consumer unwrap
        self.root_directory_file_cluster_number.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_boot_sector {
        use super::*;

        #[test]
        fn parses_valid_fat_12_boot_sector() {}
    }
}
