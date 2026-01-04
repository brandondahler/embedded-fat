#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
