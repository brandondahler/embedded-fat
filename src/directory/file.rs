#[cfg(test)]
mod tests;

use crate::allocation_table::AllocationTable;
use crate::directory_entry::DirectoryFileEntryIterator;
use crate::filesystem_state::FilesystemState;
use crate::queued_resource::QueuedResource;
use core::cell::RefCell;
use embedded_io::ErrorType;

#[derive(Clone, Debug)]
pub struct DirectoryFile<'a, S, const N: usize>
where
    S: ErrorType,
{
    allocation_table: &'a AllocationTable,
    state: &'a QueuedResource<FilesystemState<S>, N>,

    data_region_base_address: u64,
    bytes_per_cluster: u32,

    start_cluster_number: u32,
}

impl<'a, S, const N: usize> DirectoryFile<'a, S, N>
where
    S: ErrorType,
{
    pub fn new(
        allocation_table: &'a AllocationTable,
        state: &'a QueuedResource<FilesystemState<S>, N>,
        data_region_base_address: u64,
        bytes_per_cluster: u32,
        start_cluster_number: u32,
    ) -> Self {
        Self {
            allocation_table,
            state,

            data_region_base_address,
            bytes_per_cluster,

            start_cluster_number,
        }
    }

    pub fn entries(&self) -> DirectoryFileEntryIterator<'_, S, N> {
        DirectoryFileEntryIterator::new(
            self.allocation_table,
            self.state,
            self.data_region_base_address,
            self.bytes_per_cluster,
            self.start_cluster_number,
        )
    }
}
