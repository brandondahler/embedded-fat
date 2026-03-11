#[cfg(test)]
mod tests;

use crate::allocation_table::{AllocationTableKind, PhysicalAllocationTableEntry};

/// Represents a single logical entry in the allocation table.
///
/// `PhysicalAllocationTableEntry` values may map to a single logical `AllocationTableEntry` value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationTableEntry {
    Free,
    Reserved,
    NextClusterNumber(u32),
    EndOfFile,
    BadSector,
}

impl AllocationTableEntry {
    /// Parses the raw entry value in the context of the provided `AllocationTableKind`.
    pub fn new(table_kind: AllocationTableKind, entry_value: u32) -> Self {
        match entry_value {
            0 => AllocationTableEntry::Free,
            1 => AllocationTableEntry::Reserved,
            _ => {
                if entry_value < table_kind.bad_sector_value() {
                    AllocationTableEntry::NextClusterNumber(entry_value)
                } else if entry_value == table_kind.bad_sector_value() {
                    AllocationTableEntry::BadSector
                } else {
                    AllocationTableEntry::EndOfFile
                }
            }
        }
    }

    pub fn as_physical_entry(
        &self,
        table_kind: AllocationTableKind,
    ) -> Result<PhysicalAllocationTableEntry, ()> {
        let value = match self {
            AllocationTableEntry::Free => 0,
            AllocationTableEntry::Reserved => 1,
            AllocationTableEntry::NextClusterNumber(cluster_number) => *cluster_number,
            AllocationTableEntry::BadSector => table_kind.bad_sector_value(),
            AllocationTableEntry::EndOfFile => table_kind.end_of_chain_value(),
        };

        PhysicalAllocationTableEntry::new(table_kind, value)
    }
}
