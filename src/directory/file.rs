#[cfg(test)]
mod tests;

use crate::allocation_table::AllocationTable;
use crate::directory_entry::DirectoryFileEntryIterator;
use core::cell::RefCell;
use embedded_io::ErrorType;

#[derive(Clone, Debug)]
pub struct DirectoryFile<'a, S>
where
    S: ErrorType,
{
    device: &'a RefCell<S>,
    allocation_table: &'a AllocationTable,

    data_region_base_address: u64,
    bytes_per_cluster: u32,

    start_cluster_number: u32,
}

impl<'a, S> DirectoryFile<'a, S>
where
    S: ErrorType,
{
    pub fn new(
        device: &'a RefCell<S>,
        allocation_table: &'a AllocationTable,
        data_region_base_address: u64,
        bytes_per_cluster: u32,
        start_cluster_number: u32,
    ) -> Self {
        Self {
            device,
            allocation_table,

            data_region_base_address,
            bytes_per_cluster,

            start_cluster_number,
        }
    }

    pub fn entries(&self) -> DirectoryFileEntryIterator<'_, S> {
        DirectoryFileEntryIterator::new(
            self.device,
            self.allocation_table,
            self.data_region_base_address,
            self.bytes_per_cluster,
            self.start_cluster_number,
        )
    }
}
