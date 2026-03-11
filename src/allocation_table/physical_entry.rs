#[cfg(test)]
mod tests;

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
