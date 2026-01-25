mod error;

pub use error::*;

use crate::allocation_table::AllocationTableKind;
use crate::directory_entry::DIRECTORY_ENTRY_SIZE;
use crate::utils::{read_le_u16, read_le_u32, write_le_u16, write_le_u32};
use core::fmt::Display;

#[derive(Clone, Debug)]
pub struct BiosParameterBlock {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    fat_count: u8,
    root_directory_entry_count: u16,
    total_sectors_16bit: u16,
    media_type: u8,
    fat_sector_count: u32,
    sectors_per_track: u16,
    head_count: u16,
    hidden_sector_count: u32,
    total_sectors_32bit: u32,

    ext_flags: u16,
    filesystem_version_minor: u8,
    filesystem_version_major: u8,
    root_directory_file_cluster_number: u32,
    fs_info_sector_number: u16,

    backup_boot_sector_number: u16,

    allocation_table_kind: AllocationTableKind,
}

impl BiosParameterBlock {
    pub fn from_boot_sector(bytes: &[u8; 512]) -> Result<Self, BiosParameterBlockError> {
        let bytes_per_sector = read_le_u16(bytes, 11);
        ensure!(
            matches!(bytes_per_sector, 512 | 1024 | 2048 | 4096),
            BiosParameterBlockError::InvalidBytesPerSector
        );

        let sectors_per_cluster = bytes[13];
        ensure!(
            matches!(sectors_per_cluster, 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128),
            BiosParameterBlockError::InvalidSectorsPerCluster
        );

        let reserved_sector_count = read_le_u16(bytes, 14);
        ensure!(
            reserved_sector_count != 0,
            BiosParameterBlockError::InvalidReservedSectorCount
        );

        let fat_count = bytes[16];
        ensure!(fat_count != 0, BiosParameterBlockError::InvalidFatCount);

        let root_directory_entry_count = read_le_u16(bytes, 17);
        let total_sectors_count_16bit = read_le_u16(bytes, 19);
        let media_type = bytes[21];
        ensure!(
            matches!(media_type, 0xF0 | 0xF8..=0xFF),
            BiosParameterBlockError::InvalidMediaType
        );

        let fat_sectors_count_16bit = read_le_u16(bytes, 22);
        let sectors_per_track = read_le_u16(bytes, 24);
        let head_count = read_le_u16(bytes, 26);
        let hidden_sector_count = read_le_u32(bytes, 28);
        let total_sectors_count_32bit = read_le_u32(bytes, 32);
        ensure!(
            total_sectors_count_16bit != 0 || total_sectors_count_32bit != 0,
            BiosParameterBlockError::InvalidTotalSectorCount32Bit
        );

        let fat_sectors_count_32bit = read_le_u32(bytes, 36);
        let ext_flags = read_le_u16(bytes, 40);
        let filesystem_version_minor = bytes[42];
        let filesystem_version_major = bytes[43];
        let root_directory_file_cluster_number = read_le_u32(bytes, 44);
        let fs_info_sector_number = read_le_u16(bytes, 48);
        let backup_boot_sector_number = read_le_u16(bytes, 50);

        let fat_sector_count = if fat_sectors_count_16bit > 0 {
            fat_sectors_count_16bit as u32
        } else {
            fat_sectors_count_32bit
        };

        let total_sector_count = if total_sectors_count_16bit > 0 {
            total_sectors_count_16bit as u32
        } else {
            total_sectors_count_32bit
        };

        let root_directory_sectors = (root_directory_entry_count * 32).div_ceil(bytes_per_sector);
        let data_sectors = total_sector_count
            - (reserved_sector_count as u32 + (fat_count as u32 * fat_sector_count))
            + root_directory_sectors as u32;
        let allocation_table_kind =
            AllocationTableKind::new(data_sectors / sectors_per_cluster as u32);

        if matches!(allocation_table_kind, AllocationTableKind::Fat32) {
            ensure!(
                root_directory_entry_count == 0,
                BiosParameterBlockError::InvalidRootDirectoryEntryCount
            );
            ensure!(
                total_sectors_count_16bit == 0,
                BiosParameterBlockError::InvalidTotalSectorCount16Bit
            );
            ensure!(
                fat_sectors_count_16bit == 0,
                BiosParameterBlockError::InvalidFatSectorCount16Bit
            );
        } else {
            ensure!(
                root_directory_entry_count > 0,
                BiosParameterBlockError::InvalidRootDirectoryEntryCount
            );
        }

        Ok(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sector_count,
            fat_count,
            root_directory_entry_count,
            total_sectors_16bit: total_sectors_count_16bit,
            media_type,
            sectors_per_track,
            head_count,
            hidden_sector_count,
            total_sectors_32bit: total_sectors_count_32bit,

            fat_sector_count,
            ext_flags,
            filesystem_version_minor,
            filesystem_version_major,
            root_directory_file_cluster_number,
            fs_info_sector_number,
            backup_boot_sector_number,

            allocation_table_kind,
        })
    }

    pub fn write(&self, bytes: &mut [u8]) {
        write_le_u16(bytes, 11, self.bytes_per_sector);
        bytes[13] = self.sectors_per_cluster;
        write_le_u16(bytes, 14, self.reserved_sector_count);
        bytes[16] = self.fat_count;
        write_le_u16(bytes, 17, self.root_directory_entry_count);
        write_le_u16(bytes, 19, self.total_sectors_16bit);
        bytes[21] = self.media_type;
        write_le_u16(
            bytes,
            22,
            if !matches!(self.allocation_table_kind, AllocationTableKind::Fat32) {
                self.fat_sector_count as u16
            } else {
                0
            },
        );
        write_le_u16(bytes, 24, self.sectors_per_track);
        write_le_u16(bytes, 26, self.sectors_per_track);
        write_le_u32(bytes, 28, self.hidden_sector_count);
        write_le_u32(bytes, 32, self.total_sectors_32bit);

        if matches!(self.allocation_table_kind, AllocationTableKind::Fat32) {
            write_le_u32(bytes, 36, self.fat_sector_count);

            let ext_flags = read_le_u16(bytes, 40);
            let filesystem_version_minor = bytes[42];
            let filesystem_version_major = bytes[43];
            let root_directory_file_cluster_number = read_le_u32(bytes, 44);
            let fs_info_sector_number = read_le_u16(bytes, 48);
            let backup_boot_sector_number = read_le_u16(bytes, 50);
        }
    }

    pub fn allocation_table_kind(&self) -> AllocationTableKind {
        self.allocation_table_kind
    }

    pub fn bytes_per_cluster(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_cluster as u32
    }

    pub fn allocation_table_base_address(&self) -> u32 {
        self.bytes_per_sector as u32 * self.reserved_sector_count as u32
    }

    pub fn allocation_table_size(&self) -> u32 {
        self.fat_sector_count * self.bytes_per_sector as u32
    }

    pub fn directory_table_base_address(&self) -> u32 {
        self.allocation_table_base_address()
            + (self.allocation_table_size() * self.fat_count as u32)
    }

    pub fn directory_table_entry_count(&self) -> u16 {
        self.root_directory_entry_count
    }

    pub fn directory_table_size(&self) -> u32 {
        self.root_directory_entry_count as u32 * DIRECTORY_ENTRY_SIZE as u32
    }

    pub fn data_region_base_address(&self) -> u32 {
        self.directory_table_base_address() + self.directory_table_size()
    }

    pub fn root_directory_file_cluster_number(&self) -> u32 {
        self.root_directory_file_cluster_number
    }
}
