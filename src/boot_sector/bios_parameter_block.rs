mod error;

pub use error::*;

use crate::allocation_table::AllocationTableKind;
use crate::directory_entry::DIRECTORY_ENTRY_SIZE;
use crate::utils::{read_le_u16, read_le_u32, write_le_u16, write_le_u32};
use core::fmt::Display;

#[derive(Clone, Debug)]
pub struct BiosParameterBlock {
    allocation_table_kind: AllocationTableKind,
    active_allocation_table_index: u8,
    allocation_table_mirroring_enabled: bool,

    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    fs_info_sector_index: Option<u16>,
    allocation_table_count: u8,
    root_directory_entry_count: u16,
    root_directory_file_cluster_number: Option<u32>,
    last_cluster_number: u32,
    sectors_per_allocation_table: u32,
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

        let allocation_table_count = bytes[16];
        ensure!(
            allocation_table_count != 0,
            BiosParameterBlockError::AllocationTableCountInvalid
        );

        let root_directory_entry_count = read_le_u16(bytes, 17);
        let total_sector_count_16bit = read_le_u16(bytes, 19);
        ensure!(
            matches!(bytes[21], 0xF0 | 0xF8..=0xFF),
            BiosParameterBlockError::MediaTypeInvalid
        );

        let sectors_per_allocation_table_16bit = read_le_u16(bytes, 22);

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

        let sectors_per_allocation_table = if sectors_per_allocation_table_16bit > 0 {
            sectors_per_allocation_table_16bit as u32
        } else {
            let sectors_per_allocation_table_32bit = read_le_u32(bytes, 36);
            ensure!(
                sectors_per_allocation_table_32bit != 0,
                BiosParameterBlockError::SectorsPerAllocationTableNotSet
            );

            sectors_per_allocation_table_32bit
        };

        let root_directory_sectors = (root_directory_entry_count as u32
            * DIRECTORY_ENTRY_SIZE as u32)
            .div_ceil(bytes_per_sector as u32);
        let data_sectors_count = total_sector_count
            - (reserved_sector_count as u32
                + (allocation_table_count as u32 * sectors_per_allocation_table)
                + root_directory_sectors);
        let data_cluster_count = data_sectors_count / sectors_per_cluster as u32;

        let allocation_table_kind = AllocationTableKind::new(data_cluster_count);

        let mut active_allocation_table_index = 0;
        let mut allocation_table_mirroring_enabled = true;
        let mut root_directory_file_cluster_number: Option<u32> = None;
        let mut fs_info_sector_index: Option<u16> = None;

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
                sectors_per_allocation_table_16bit == 0,
                BiosParameterBlockError::SectorsPerAllocationTable16BitInvalid
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

            fs_info_sector_index = Some({
                let value = read_le_u16(bytes, 48);
                ensure!(
                    value >= 1,
                    BiosParameterBlockError::FsInfoSectorNumberInvalid
                );

                value
            });
        } else {
            ensure!(
                sectors_per_allocation_table_16bit != 0,
                BiosParameterBlockError::SectorsPerAllocationTable16BitInvalid
            );

            ensure!(
                root_directory_entry_count > 0,
                BiosParameterBlockError::RootDirectoryEntryCountInvalid
            );
        }

        let allocation_table_bytes = sectors_per_allocation_table as u64 * bytes_per_sector as u64;
        let allocation_table_entry_count = match allocation_table_kind {
            AllocationTableKind::Fat12 => (allocation_table_bytes * 3) / 2,
            AllocationTableKind::Fat16 => allocation_table_bytes / 2,
            AllocationTableKind::Fat32 => allocation_table_bytes / 4,
        };

        ensure!(
            allocation_table_entry_count >= data_cluster_count as u64 + 2,
            BiosParameterBlockError::AllocationTableTooSmall
        );

        Ok(Self {
            allocation_table_kind,

            bytes_per_sector,
            sectors_per_cluster,

            reserved_sector_count,
            sectors_per_allocation_table,
            allocation_table_count,
            root_directory_entry_count,
            root_directory_file_cluster_number,
            last_cluster_number: data_cluster_count + 1,

            active_allocation_table_index,
            allocation_table_mirroring_enabled,
            fs_info_sector_index,
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

    pub fn allocation_table_count(&self) -> u8 {
        self.allocation_table_count
    }

    pub fn bytes_per_cluster(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_cluster as u32
    }

    pub fn directory_table_base_address(&self) -> u64 {
        self.allocation_table_base_address()
            + (self.bytes_per_sector as u64
                * self.sectors_per_allocation_table as u64
                * self.allocation_table_count as u64)
    }

    pub fn directory_table_entry_count(&self) -> u16 {
        self.root_directory_entry_count
    }

    pub fn data_region_base_address(&self) -> u64 {
        self.directory_table_base_address()
            + (self.root_directory_entry_count as u64 * DIRECTORY_ENTRY_SIZE as u64)
    }

    pub fn fs_info_base_address(&self) -> Option<u64> {
        Some(self.fs_info_sector_index? as u64 * self.bytes_per_sector as u64)
    }

    pub fn last_cluster_number(&self) -> u32 {
        self.last_cluster_number
    }

    pub fn root_directory_file_cluster_number(&self) -> Option<u32> {
        self.root_directory_file_cluster_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_boot_sector {
        use super::*;

        #[test]
        fn allocation_table_too_small_returns_err() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.sectors_per_allocation_table_32bit = 1;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let result =
                BiosParameterBlock::from_boot_sector(&bytes).expect_err("Err should be returned");

            assert_eq!(result, BiosParameterBlockError::AllocationTableTooSmall);
        }

        mod bytes_per_sector {
            use super::*;

            #[test]
            fn valid_values_parse_successfully() {
                let valid_values = [512, 1024, 2048, 4096];

                for valid_value in valid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.bytes_per_sector = valid_value;
                    config.sectors_per_allocation_table_32bit *= 16;

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect("Ok should be returned");

                    assert_eq!(result.allocation_table_kind(), AllocationTableKind::Fat32);
                }
            }

            #[test]
            fn invalid_returns_err() {
                let invalid_values = [0, 1, 256, 768];

                for invalid_value in invalid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.bytes_per_sector = invalid_value;

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect_err("Err should be returned");

                    assert_eq!(result, BiosParameterBlockError::BytesPerSectorInvalid);
                }
            }
        }

        mod sectors_per_cluster {
            use super::*;

            #[test]
            fn valid_values_parse_successfully() {
                let valid_values = [1, 2, 4, 8, 16, 32, 64, 128];

                for valid_value in valid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.sectors_per_cluster = valid_value;
                    config.total_sector_count_32bit *= 128;
                    config.sectors_per_allocation_table_32bit *= 128;

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect("Ok should be returned");

                    assert_eq!(result.allocation_table_kind(), AllocationTableKind::Fat32);
                }
            }

            #[test]
            fn invalid_returns_err() {
                let invalid_values = [0, 3, 96, 255];

                for invalid_value in invalid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.sectors_per_cluster = invalid_value;

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect_err("Err should be returned");

                    assert_eq!(result, BiosParameterBlockError::SectorsPerClusterInvalid);
                }
            }
        }

        mod reserved_sector_count {
            use super::*;

            #[test]
            fn valid_values_parse_successfully() {
                let valid_values = [1, 2, 3, 16, 32, 128, 9999];

                for valid_value in valid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.reserved_sector_count = valid_value;

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect("Ok should be returned");

                    assert_eq!(result.allocation_table_kind(), AllocationTableKind::Fat32);
                }
            }

            #[test]
            fn invalid_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.reserved_sector_count = 0;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(result, BiosParameterBlockError::ReservedSectorCountInvalid);
            }
        }

        mod allocation_table_count {
            use super::*;

            #[test]
            fn valid_values_parse_successfully() {
                let valid_values = [1, 2, 3, 16, 32, 128];

                for valid_value in valid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.allocation_table_count = valid_value;
                    config.total_sector_count_32bit +=
                        (valid_value as u32 * config.sectors_per_allocation_table_32bit);

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect("Ok should be returned");

                    assert_eq!(result.allocation_table_kind(), AllocationTableKind::Fat32);
                }
            }

            #[test]
            fn invalid_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.allocation_table_count = 0;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(result, BiosParameterBlockError::AllocationTableCountInvalid);
            }
        }

        mod media_type {
            use super::*;

            #[test]
            fn valid_values_parse_successfully() {
                let valid_values = [0xF0, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFF];

                for valid_value in valid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.media_type = valid_value;

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect("Ok should be returned");

                    assert_eq!(result.allocation_table_kind(), AllocationTableKind::Fat32);
                }
            }

            #[test]
            fn invalid_returns_err() {
                let invalid_values = [0, 0x12, 0xE0, 0xF1, 0xF7];

                for invalid_value in invalid_values {
                    let mut config = BiosParameterBlockConfig::fat32();
                    config.media_type = invalid_value;

                    let mut bytes = [0x00; 512];
                    config.write(&mut bytes);

                    let result = BiosParameterBlock::from_boot_sector(&bytes)
                        .expect_err("Err should be returned");

                    assert_eq!(result, BiosParameterBlockError::MediaTypeInvalid);
                }
            }
        }

        mod total_sector_count {
            use super::*;

            #[test]
            fn both_fields_zero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.total_sector_count_16bit = 0;
                config.total_sector_count_32bit = 0;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(result, BiosParameterBlockError::TotalSectorCountNotSet);
            }
        }

        mod sectors_per_allocation_table {
            use super::*;

            #[test]
            fn both_fields_zero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.sectors_per_allocation_table_16bit = 0;
                config.sectors_per_allocation_table_32bit = 0;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::SectorsPerAllocationTableNotSet
                );
            }
        }

        mod allocation_table_fat32 {
            use super::*;

            #[test]
            fn root_directory_entry_count_nonzero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.root_directory_entry_count = 1;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::RootDirectoryEntryCountInvalid
                );
            }

            #[test]
            fn total_sector_count_16bit_nonzero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.total_sector_count_16bit = 0xFFFF;
                config.reserved_sector_count = 1;
                config.sectors_per_allocation_table_32bit = 1;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::TotalSectorCount16BitInvalid
                );
            }

            #[test]
            fn sectors_per_allocation_table_16bit_nonzero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.sectors_per_allocation_table_16bit =
                    config.sectors_per_allocation_table_32bit as u16;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::SectorsPerAllocationTable16BitInvalid
                );
            }

            #[test]
            fn filesystem_version_minor_nonzero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.filesystem_version_minor = 1;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::FilesystemVersionUnsupported
                );
            }

            #[test]
            fn filesystem_version_major_nonzero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.filesystem_version_major = 1;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::FilesystemVersionUnsupported
                );
            }

            #[test]
            fn root_directory_file_cluster_number_low_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.root_directory_file_cluster_number = 1;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::RootDirectoryFileClusterNumberInvalid
                );
            }

            #[test]
            fn fs_info_sector_index_zero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat32();
                config.fs_info_sector_index = 0;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(result, BiosParameterBlockError::FsInfoSectorNumberInvalid);
            }
        }

        mod allocation_table_non_fat32 {
            use super::*;

            #[test]
            fn sectors_per_allocation_table_16bit_zero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat16();
                config.sectors_per_allocation_table_32bit =
                    config.sectors_per_allocation_table_16bit as u32;
                config.sectors_per_allocation_table_16bit = 0;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::SectorsPerAllocationTable16BitInvalid
                );
            }

            #[test]
            fn root_directory_entry_count_zero_returns_err() {
                let mut config = BiosParameterBlockConfig::fat16();
                config.root_directory_entry_count = 0;

                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let result = BiosParameterBlock::from_boot_sector(&bytes)
                    .expect_err("Err should be returned");

                assert_eq!(
                    result,
                    BiosParameterBlockError::RootDirectoryEntryCountInvalid
                );
            }
        }
    }

    mod active_allocation_table {
        use super::*;

        #[test]
        fn non_fat32_returns_zero() {
            let configs = [
                BiosParameterBlockConfig::fat12(),
                BiosParameterBlockConfig::fat16(),
            ];

            for config in configs {
                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

                assert_eq!(bios_parameter_block.active_allocation_table_index(), 0);
            }
        }

        #[test]
        fn fat32_derived_from_ext_flags() {
            let expected_index = 5;

            let mut config = BiosParameterBlockConfig::fat32();
            config.ext_flags = (config.ext_flags & !0b111) | (expected_index as u16 & 0b111);

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(
                bios_parameter_block.active_allocation_table_index(),
                expected_index
            );
        }
    }

    mod allocation_table_kind {
        use super::*;

        #[test]
        fn returns_correct_value() {
            let expected_values = [
                (
                    BiosParameterBlockConfig::fat12(),
                    AllocationTableKind::Fat12,
                ),
                (
                    BiosParameterBlockConfig::fat16(),
                    AllocationTableKind::Fat16,
                ),
                (
                    BiosParameterBlockConfig::fat32(),
                    AllocationTableKind::Fat32,
                ),
            ];

            for (config, expected_allocation_table_kind) in expected_values {
                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

                assert_eq!(
                    bios_parameter_block.allocation_table_kind(),
                    expected_allocation_table_kind
                );
            }
        }
    }

    mod allocation_table_mirroring_enabled {
        use super::*;

        #[test]
        fn non_fat32_returns_true() {
            let configs = [
                BiosParameterBlockConfig::fat12(),
                BiosParameterBlockConfig::fat16(),
            ];

            for config in configs {
                let mut bytes = [0x00; 512];
                config.write(&mut bytes);

                let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

                assert_eq!(
                    bios_parameter_block.allocation_table_mirroring_enabled(),
                    true
                );
            }
        }

        #[test]
        fn fat32_derived_from_ext_flags() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.ext_flags &= !(1 << 7);

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(
                bios_parameter_block.allocation_table_mirroring_enabled(),
                false
            );
        }
    }

    mod allocation_table_base_address {
        use super::*;

        #[test]
        fn derived_from_configurations_correctly() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.bytes_per_sector = 1024;
            config.reserved_sector_count = 7;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(bios_parameter_block.allocation_table_base_address(), 7168);
        }
    }

    mod allocation_table_count {
        use super::*;

        #[test]
        fn returns_correct_value() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.allocation_table_count = 7;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(bios_parameter_block.allocation_table_count(), 7);
        }
    }

    mod bytes_per_cluster {
        use super::*;

        #[test]
        fn derived_from_configurations_correctly() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.bytes_per_sector = 1024;
            config.sectors_per_cluster = 4;

            config.total_sector_count_32bit *= 8;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(bios_parameter_block.bytes_per_cluster(), 4096);
        }
    }

    mod directory_table_base_address {
        use crate::boot_sector::BiosParameterBlock;
        use crate::boot_sector::bios_parameter_block::tests::BiosParameterBlockConfig;

        #[test]
        fn derived_from_configurations_correctly() {
            let mut config = BiosParameterBlockConfig::fat16();
            config.bytes_per_sector = 1024;
            config.reserved_sector_count = 7;
            config.sectors_per_allocation_table_16bit = 1024;
            config.allocation_table_count = 3;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(
                bios_parameter_block.directory_table_base_address(),
                7_168 + 3_145_728
            );
        }
    }

    mod directory_table_entry_count {
        use super::*;

        #[test]
        fn returns_correct_value() {
            let mut config = BiosParameterBlockConfig::fat16();
            config.root_directory_entry_count = 1337;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(bios_parameter_block.directory_table_entry_count(), 1337);
        }
    }

    mod data_region_base_address {
        use super::*;

        #[test]
        fn derived_from_configurations_correctly() {
            let mut config = BiosParameterBlockConfig::fat16();
            config.bytes_per_sector = 1024;
            config.reserved_sector_count = 7;
            config.sectors_per_allocation_table_16bit = 1024;
            config.allocation_table_count = 3;
            config.root_directory_entry_count = 1337;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(
                bios_parameter_block.data_region_base_address(),
                7_168 + 3_145_728 + 42_784
            );
        }
    }

    mod fs_info_base_address {
        use super::*;

        #[test]
        fn non_fat32_returns_none() {
            let configs = [
                BiosParameterBlockConfig::fat12(),
                BiosParameterBlockConfig::fat16(),
            ];

            for config in configs {
                let mut bytes = [0; 512];
                config.write(&mut bytes);

                let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

                assert_eq!(bios_parameter_block.fs_info_base_address(), None);
            }
        }

        #[test]
        fn fat32_derived_from_configurations_correctly() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.bytes_per_sector = 1024;
            config.fs_info_sector_index = 5;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(bios_parameter_block.fs_info_base_address(), Some(5120));
        }
    }

    mod last_cluster_number {
        use super::*;

        #[test]
        fn non_fat32_derived_from_configurations_correctly() {
            let mut config = BiosParameterBlockConfig::fat16();
            config.bytes_per_sector = 1024;
            config.sectors_per_cluster = 2;
            config.total_sector_count_16bit = 0xF000;
            config.reserved_sector_count = 7;
            config.allocation_table_count = 3;
            config.sectors_per_allocation_table_16bit = 512;
            config.root_directory_entry_count = 1024;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(bios_parameter_block.last_cluster_number(), 29_933);
        }

        #[test]
        fn fat32_derived_from_configurations_correctly() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.bytes_per_sector = 1024;
            config.sectors_per_cluster = 2;
            config.total_sector_count_32bit = 0x4_0000;
            config.reserved_sector_count = 7;
            config.allocation_table_count = 3;
            config.sectors_per_allocation_table_32bit = 1024;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(bios_parameter_block.last_cluster_number(), 129_533);
        }
    }

    mod root_directory_file_cluster_number {
        use super::*;

        #[test]
        fn non_fat32_returns_none() {
            let configs = [
                BiosParameterBlockConfig::fat12(),
                BiosParameterBlockConfig::fat16(),
            ];

            for config in configs {
                let mut bytes = [0; 512];
                config.write(&mut bytes);

                let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

                assert_eq!(
                    bios_parameter_block.root_directory_file_cluster_number(),
                    None
                );
            }
        }

        #[test]
        fn fat32_returns_configured_value() {
            let mut config = BiosParameterBlockConfig::fat32();
            config.root_directory_file_cluster_number = 1337;

            let mut bytes = [0x00; 512];
            config.write(&mut bytes);

            let bios_parameter_block = BiosParameterBlock::from_boot_sector(&bytes).unwrap();

            assert_eq!(
                bios_parameter_block.root_directory_file_cluster_number(),
                Some(1337)
            );
        }
    }

    struct BiosParameterBlockConfig {
        bytes_per_sector: u16,
        sectors_per_cluster: u8,
        reserved_sector_count: u16,
        allocation_table_count: u8,
        root_directory_entry_count: u16,
        total_sector_count_16bit: u16,
        media_type: u8,
        sectors_per_allocation_table_16bit: u16,
        total_sector_count_32bit: u32,
        sectors_per_allocation_table_32bit: u32,
        ext_flags: u16,
        filesystem_version_minor: u8,
        filesystem_version_major: u8,
        root_directory_file_cluster_number: u32,
        fs_info_sector_index: u16,
    }

    impl BiosParameterBlockConfig {
        fn fat12() -> BiosParameterBlockConfig {
            BiosParameterBlockConfig {
                bytes_per_sector: 512,
                sectors_per_cluster: 4,
                reserved_sector_count: 1,
                allocation_table_count: 1,
                root_directory_entry_count: 128,
                total_sector_count_16bit: 8192,
                media_type: 0xF0,
                sectors_per_allocation_table_16bit: 3,
                total_sector_count_32bit: 0,
                sectors_per_allocation_table_32bit: 0,
                ext_flags: 0,
                filesystem_version_minor: 0,
                filesystem_version_major: 0,
                root_directory_file_cluster_number: 0,
                fs_info_sector_index: 0,
            }
        }

        fn fat16() -> BiosParameterBlockConfig {
            BiosParameterBlockConfig {
                bytes_per_sector: 512,
                sectors_per_cluster: 1,
                reserved_sector_count: 1,
                allocation_table_count: 1,
                root_directory_entry_count: 512,
                total_sector_count_16bit: 32768,
                media_type: 0xF0,
                sectors_per_allocation_table_16bit: 128,
                total_sector_count_32bit: 0,
                sectors_per_allocation_table_32bit: 0,
                ext_flags: 0,
                filesystem_version_minor: 0,
                filesystem_version_major: 0,
                root_directory_file_cluster_number: 0,
                fs_info_sector_index: 0,
            }
        }

        fn fat32() -> BiosParameterBlockConfig {
            BiosParameterBlockConfig {
                bytes_per_sector: 512,
                sectors_per_cluster: 1,
                reserved_sector_count: 32,
                allocation_table_count: 1,
                root_directory_entry_count: 0,
                total_sector_count_16bit: 0,
                media_type: 0xF0,
                sectors_per_allocation_table_16bit: 0,
                total_sector_count_32bit: 131_072,
                sectors_per_allocation_table_32bit: 1024,
                ext_flags: 0,
                filesystem_version_minor: 0,
                filesystem_version_major: 0,
                root_directory_file_cluster_number: 2,
                fs_info_sector_index: 6,
            }
        }

        fn write(&self, bytes: &mut [u8; 512]) {
            write_le_u16(bytes, 11, self.bytes_per_sector);
            bytes[13] = self.sectors_per_cluster;
            write_le_u16(bytes, 14, self.reserved_sector_count);
            bytes[16] = self.allocation_table_count;
            write_le_u16(bytes, 17, self.root_directory_entry_count);
            write_le_u16(bytes, 19, self.total_sector_count_16bit);
            bytes[21] = self.media_type;
            write_le_u16(bytes, 22, self.sectors_per_allocation_table_16bit);
            write_le_u32(bytes, 32, self.total_sector_count_32bit);

            write_le_u32(bytes, 36, self.sectors_per_allocation_table_32bit);
            write_le_u16(bytes, 40, self.ext_flags);
            bytes[42] = self.filesystem_version_minor;
            bytes[43] = self.filesystem_version_major;
            write_le_u32(bytes, 44, self.root_directory_file_cluster_number);
            write_le_u16(bytes, 48, self.fs_info_sector_index);
        }
    }
}
