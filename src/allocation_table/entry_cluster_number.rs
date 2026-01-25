use crate::AllocationTableKind;
use crate::allocation_table::AllocationTable;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationTableEntryClusterNumber(u32);

impl AllocationTableEntryClusterNumber {
    pub fn new(value: u32) -> Result<Self, ()> {
        if value >= 2 {
            Ok(AllocationTableEntryClusterNumber(value))
        } else {
            Err(())
        }
    }

    pub fn value(&self, table_kind: AllocationTableKind) -> Result<u32, ()> {
        if self.0 < table_kind.bad_sector_value() {
            Ok(self.0)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod new {
        use super::*;

        #[test]
        fn valid_values_returns_ok() {
            let values = [
                2,
                4,
                8,
                16,
                256,
                1024,
                4096,
                0x1000,
                0x10_000,
                0x1000_0000,
                0xFFFF_FFFF,
            ];

            for value in values {
                let result =
                    AllocationTableEntryClusterNumber::new(value).expect("Ok should be returned");

                assert_eq!(result.0, value);
            }
        }

        #[test]
        fn less_than_2_returns_err() {
            for value in 0..2 {
                let result = AllocationTableEntryClusterNumber::new(value);

                assert!(result.is_err(), "Err should be returned");
            }
        }
    }

    mod value {
        use crate::AllocationTableKind;
        use crate::allocation_table::AllocationTableEntryClusterNumber;
        use strum::IntoEnumIterator;

        #[test]
        fn within_table_kind_range_returns_ok() {
            for table_kind in AllocationTableKind::iter() {
                let values = [2, 16, table_kind.bad_sector_value() - 1];

                for value in values {
                    let cluster_number = AllocationTableEntryClusterNumber::new(value).unwrap();
                    let result = cluster_number
                        .value(table_kind)
                        .expect("Ok should be returned");

                    assert_eq!(result, value);
                }
            }
        }

        #[test]
        fn invalid_for_table_kind_returns_err() {
            for table_kind in AllocationTableKind::iter() {
                let values = [
                    table_kind.bad_sector_value(),
                    table_kind.end_of_chain_value(),
                    table_kind.entry_mask(),
                    table_kind.entry_mask() + 1,
                ];

                for value in values {
                    let cluster_number = AllocationTableEntryClusterNumber::new(value).unwrap();
                    let result = cluster_number.value(table_kind);

                    assert!(result.is_err(), "Err should be returned");
                }
            }
        }
    }
}
