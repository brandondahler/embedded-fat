#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum AllocationTableKind {
    Fat12,
    Fat16,
    Fat32,
}

impl AllocationTableKind {
    pub(crate) const fn new(data_cluster_count: u32) -> AllocationTableKind {
        // NOTE: Values aren't round, spec promises they're right despite that being the case
        match data_cluster_count {
            0..4085 => AllocationTableKind::Fat12,
            4085..65525 => AllocationTableKind::Fat16,
            65525.. => AllocationTableKind::Fat32,
        }
    }

    pub(crate) const fn bad_sector_value(&self) -> u32 {
        match self {
            AllocationTableKind::Fat12 => 0x0000_0FF7,
            AllocationTableKind::Fat16 => 0x0000_FFF7,
            AllocationTableKind::Fat32 => 0x0FFF_0FF7,
        }
    }

    pub(crate) const fn end_of_chain_value(&self) -> u32 {
        match self {
            AllocationTableKind::Fat12 => 0x0000_0FF8,
            AllocationTableKind::Fat16 => 0x0000_FFF8,
            AllocationTableKind::Fat32 => 0x0FFF_FFF8,
        }
    }

    pub(crate) const fn entry_mask(self) -> u32 {
        let bit_count = match self {
            AllocationTableKind::Fat12 => 12,
            AllocationTableKind::Fat16 => 16,
            AllocationTableKind::Fat32 => 28,
        };

        !(!0 << bit_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    mod new {
        use super::*;

        #[test]
        fn values_map_correctly() {
            let values = [
                (1, AllocationTableKind::Fat12),
                (1024, AllocationTableKind::Fat12),
                (4000, AllocationTableKind::Fat12),
                (4084, AllocationTableKind::Fat12),
                (4085, AllocationTableKind::Fat16),
                (4096, AllocationTableKind::Fat16),
                (65000, AllocationTableKind::Fat16),
                (65524, AllocationTableKind::Fat16),
                (65525, AllocationTableKind::Fat32),
                (65536, AllocationTableKind::Fat32),
                (131_072, AllocationTableKind::Fat32),
                (1_000_000, AllocationTableKind::Fat32),
            ];

            for (value, expected_kind) in values {
                assert_eq!(AllocationTableKind::new(value), expected_kind);
            }
        }
    }

    mod bad_sector_value {
        use super::*;

        #[test]
        fn matches_expectations() {
            for kind in AllocationTableKind::iter() {
                assert!(
                    kind.bad_sector_value() > 2,
                    "Value should be greater than the free and reserved values"
                );
                assert!(
                    kind.bad_sector_value() < kind.end_of_chain_value(),
                    "Value should be less than the end of chain value"
                );
                assert!(
                    kind.bad_sector_value() <= kind.entry_mask(),
                    "Value should fit within the entry value mask"
                );
            }
        }
    }

    mod end_of_chain_value {
        use super::*;

        #[test]
        fn matches_expectations() {
            for kind in AllocationTableKind::iter() {
                assert!(
                    kind.end_of_chain_value() > kind.bad_sector_value(),
                    "Value should be greater than the bad sector value"
                );
                assert!(
                    kind.end_of_chain_value() <= kind.entry_mask(),
                    "Value should fit within the entry mask"
                );
            }
        }
    }

    mod entry_mask {
        use super::*;

        #[test]
        fn matches_expectations() {
            assert_eq!(AllocationTableKind::Fat12.entry_mask(), 0x0000_0FFF);
            assert_eq!(AllocationTableKind::Fat16.entry_mask(), 0x0000_FFFF);
            assert_eq!(AllocationTableKind::Fat32.entry_mask(), 0x0FFF_FFFF);
        }
    }
}
