use crate::AllocationTableKind;
use crate::allocation_table::AllocationTableEntry;
use crate::utils::{read_le_u32, write_le_u32};

#[derive(Debug, Clone)]
pub struct PhysicalAllocationTableEntry {
    table_kind: AllocationTableKind,
    value: u32,
}

impl PhysicalAllocationTableEntry {
    pub fn new(
        table_kind: AllocationTableKind,
        value: u32,
    ) -> Result<PhysicalAllocationTableEntry, ()> {
        if value <= table_kind.entry_mask() {
            Ok(Self { table_kind, value })
        } else {
            Err(())
        }
    }

    pub fn from_bytes(
        table_kind: AllocationTableKind,
        bytes: &[u8; 4],
        is_nibble_offset: bool,
    ) -> Self {
        let mut value = read_le_u32(bytes, 0);

        if is_nibble_offset {
            assert_eq!(
                table_kind,
                AllocationTableKind::Fat12,
                "Only FAT12 tables can have bytes that are nibble offset"
            );
            value >>= 4;
        }

        Self {
            table_kind,
            value: value & table_kind.entry_mask(),
        }
    }

    pub fn as_logical_entry(&self) -> AllocationTableEntry {
        AllocationTableEntry::new(self.table_kind, self.value)
    }

    pub fn write(&self, bytes: &mut [u8; 4], is_nibble_offset: bool) {
        let mut mask = self.table_kind.entry_mask();
        let mut entry_value = self.value;

        if is_nibble_offset {
            assert_eq!(
                self.table_kind,
                AllocationTableKind::Fat12,
                "Only FAT12 tables can have bytes that are nibble offset"
            );

            mask <<= 4;
            entry_value <<= 4;
        }

        let mut value = read_le_u32(bytes, 0);
        value = (value & !mask) | entry_value;

        write_le_u32(bytes, 0, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    mod new {
        use super::*;

        #[test]
        fn valid_values_allowed() {
            let values = [
                (AllocationTableKind::Fat12, 0),
                (AllocationTableKind::Fat12, 0x0FFF),
                (AllocationTableKind::Fat16, 0),
                (AllocationTableKind::Fat16, 0xFFFF),
                (AllocationTableKind::Fat32, 0),
                (AllocationTableKind::Fat32, 0x0FFF_FFFF),
            ];

            for (table_kind, value) in values {
                let result = PhysicalAllocationTableEntry::new(table_kind, value)
                    .expect("Ok should be returned");

                assert_eq!(result.table_kind, table_kind);
                assert_eq!(result.value, value);
            }
        }

        #[test]
        fn larger_than_mask_returns_none() {
            let values = [
                (AllocationTableKind::Fat12, 0x1000),
                (AllocationTableKind::Fat16, 0x1_0000),
                (AllocationTableKind::Fat32, 0x8000_0000),
            ];

            for (table_kind, value) in values {
                let result = PhysicalAllocationTableEntry::new(table_kind, value);

                assert!(result.is_err(), "Err should be returned");
            }
        }
    }

    mod from_bytes {
        use super::*;

        #[test]
        fn value_masked_correctly() {
            let bytes = [0xFF; 4];

            for table_kind in AllocationTableKind::iter() {
                let result = PhysicalAllocationTableEntry::from_bytes(table_kind, &bytes, false);

                assert_eq!(result.table_kind, table_kind);
                assert_eq!(result.value, table_kind.entry_mask());
            }
        }

        #[test]
        fn correct_endianness_used() {
            let value = 0x12345678;
            let bytes = [0x78, 0x56, 0x34, 0x12];

            for table_kind in AllocationTableKind::iter() {
                let result = PhysicalAllocationTableEntry::from_bytes(table_kind, &bytes, false);

                assert_eq!(result.table_kind, table_kind);
                assert_eq!(result.value, value & table_kind.entry_mask());
            }
        }

        #[test]
        fn fat12_nibble_offset_parses_correctly() {
            let value = 0x123;
            let bytes = [0x3F, 0x12, 0xFF, 0xFF];

            let result =
                PhysicalAllocationTableEntry::from_bytes(AllocationTableKind::Fat12, &bytes, true);

            assert_eq!(result.table_kind, AllocationTableKind::Fat12);
            assert_eq!(result.value, value);
        }

        #[test]
        #[should_panic]
        fn fat_16_nibble_offset_panics() {
            PhysicalAllocationTableEntry::from_bytes(AllocationTableKind::Fat16, &[0x00; 4], true);
        }

        #[test]
        #[should_panic]
        fn fat_32_nibble_offset_panics() {
            PhysicalAllocationTableEntry::from_bytes(AllocationTableKind::Fat32, &[0x00; 4], true);
        }
    }

    mod write {
        use super::*;

        #[test]
        fn fat12_writes_contained_value_without_disturbing_extra_bits() {
            let source_bytes = [0x12, 0xF3, 0xFF, 0xFF];
            let physical_entry = PhysicalAllocationTableEntry::from_bytes(
                AllocationTableKind::Fat12,
                &source_bytes,
                false,
            );

            let mut output_bytes = [0xAA; 4];
            physical_entry.write(&mut output_bytes, false);

            assert_eq!(output_bytes, [0x12, 0xA3, 0xAA, 0xAA]);
        }

        #[test]
        fn fat12_nibble_offset_writes_contained_value_without_disturbing_extra_bits() {
            let source_bytes = [0x12, 0xF3, 0xFF, 0xFF];
            let physical_entry = PhysicalAllocationTableEntry::from_bytes(
                AllocationTableKind::Fat12,
                &source_bytes,
                false,
            );

            let mut output_bytes = [0xAA; 4];
            physical_entry.write(&mut output_bytes, true);

            assert_eq!(output_bytes, [0x2A, 0x31, 0xAA, 0xAA]);
        }

        #[test]
        fn fat16_writes_contained_value_without_disturbing_extra_bits() {
            let source_bytes = [0x12, 0x34, 0xFF, 0xFF];
            let physical_entry = PhysicalAllocationTableEntry::from_bytes(
                AllocationTableKind::Fat16,
                &source_bytes,
                false,
            );

            let mut output_bytes = [0xAA; 4];
            physical_entry.write(&mut output_bytes, false);

            assert_eq!(output_bytes, [0x12, 0x34, 0xAA, 0xAA]);
        }

        #[test]
        fn fat32_writes_contained_value_without_disturbing_extra_bits() {
            let source_bytes = [0x12, 0x34, 0x56, 0xF8];
            let physical_entry = PhysicalAllocationTableEntry::from_bytes(
                AllocationTableKind::Fat32,
                &source_bytes,
                false,
            );

            let mut output_bytes = [0xAA; 4];
            physical_entry.write(&mut output_bytes, false);

            assert_eq!(output_bytes, [0x12, 0x34, 0x56, 0xA8]);
        }
    }
}
