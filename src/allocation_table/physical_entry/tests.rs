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
