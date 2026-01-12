use crate::allocation_table::AllocationTableKind;

#[derive(Debug, Eq, PartialEq)]
pub enum AllocationTableEntry {
    Free,
    NextClusterNumber(u32),
    EndOfFile,
    BadSector,
    Reserved,
}

impl AllocationTableEntry {
    pub fn from_entry_value(table_kind: AllocationTableKind, entry_value: u32) -> Self {
        match entry_value {
            0 => AllocationTableEntry::Free,
            1 => AllocationTableEntry::Reserved,
            _ => {
                if entry_value == table_kind.bad_sector_value() {
                    AllocationTableEntry::BadSector
                } else if entry_value < table_kind.end_of_chain_value() {
                    AllocationTableEntry::NextClusterNumber(entry_value)
                } else {
                    AllocationTableEntry::EndOfFile
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
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
                    verify_parses_correctly(
                        table_kind,
                        entry_value,
                        AllocationTableEntry::EndOfFile,
                    );
                }
            }
        }

        fn verify_parses_correctly(
            table_kind: AllocationTableKind,
            entry_value: u32,
            expected_entry: AllocationTableEntry,
        ) {
            assert_eq!(
                AllocationTableEntry::from_entry_value(table_kind, entry_value),
                expected_entry,
                "table_kind={table_kind:?}, value=0x{entry_value:0X} parses correctly"
            );
        }
    }
}
