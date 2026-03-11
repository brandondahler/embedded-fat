use super::*;
use strum::IntoEnumIterator;

mod from_entry_value {
    use super::*;

    #[test]
    fn free_value_parsed_successfully() {
        for table_kind in AllocationTableKind::iter() {
            verify_parses_correctly(table_kind, 0, AllocationTableEntry::Free);
        }
    }

    #[test]
    fn reserved_value_parsed_successfully() {
        for table_kind in AllocationTableKind::iter() {
            verify_parses_correctly(table_kind, 1, AllocationTableEntry::Reserved);
        }
    }

    #[test]
    fn allocated_cluster_value_parsed_successfully() {
        for table_kind in AllocationTableKind::iter() {
            verify_parses_correctly(table_kind, 2, AllocationTableEntry::NextClusterNumber(2));

            verify_parses_correctly(
                table_kind,
                table_kind.bad_sector_value() - 1,
                AllocationTableEntry::NextClusterNumber(table_kind.bad_sector_value() - 1),
            );
        }
    }

    #[test]
    fn bad_sector_value_parsed_correctly() {
        for table_kind in AllocationTableKind::iter() {
            verify_parses_correctly(
                table_kind,
                table_kind.bad_sector_value(),
                AllocationTableEntry::BadSector,
            );
        }
    }

    #[test]
    fn end_of_file_value_parsed_correctly() {
        for table_kind in AllocationTableKind::iter() {
            for entry_value in table_kind.end_of_chain_value()..=table_kind.entry_mask() {
                verify_parses_correctly(table_kind, entry_value, AllocationTableEntry::EndOfFile);
            }
        }
    }

    fn verify_parses_correctly(
        table_kind: AllocationTableKind,
        entry_value: u32,
        expected_entry: AllocationTableEntry,
    ) {
        assert!(
            matches!(
                AllocationTableEntry::new(table_kind, entry_value),
                expected_entry
            ),
            "table_kind={table_kind:?}, value=0x{entry_value:0X} parses correctly"
        );
    }
}

mod as_physical_entry {
    use super::*;

    #[test]
    fn values_roundtrips_physical_representation() {
        let values = [
            AllocationTableEntry::Free,
            AllocationTableEntry::Reserved,
            AllocationTableEntry::NextClusterNumber(2),
            AllocationTableEntry::BadSector,
            AllocationTableEntry::EndOfFile,
        ];

        for table_kind in AllocationTableKind::iter() {
            for value in &values {
                let physical_entry = value
                    .as_physical_entry(table_kind)
                    .expect("Ok should be returned");

                assert_eq!(physical_entry.as_logical_entry(), *value);
            }
        }
    }

    #[test]
    fn unsupported_cluster_number_returns_none() {
        let entry = AllocationTableEntry::NextClusterNumber(0x00FF_FFFF);
        let result = entry.as_physical_entry(AllocationTableKind::Fat16);

        assert!(result.is_err(), "Err should be returned");
    }
}
