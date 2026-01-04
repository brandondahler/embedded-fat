use crate::allocation_table::AllocationTable;
use crate::device::Device;
use crate::directory_entry::DirectoryFileEntryIterator;
use core::cell::RefCell;
use embedded_io::ErrorType;

pub struct DirectoryFile<'a, D>
where
    D: Device,
{
    device: &'a D,
    allocation_table: &'a AllocationTable,

    data_region_base_address: u32,
    bytes_per_cluster: u32,

    start_cluster_number: u32,
}

impl<'a, D> DirectoryFile<'a, D>
where
    D: Device,
{
    pub fn new(
        device: &'a D,
        allocation_table: &'a AllocationTable,
        data_region_base_address: u32,
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

    pub fn entries(&self) -> DirectoryFileEntryIterator<'_, D> {
        DirectoryFileEntryIterator::new(
            self.device,
            self.allocation_table,
            self.data_region_base_address,
            self.bytes_per_cluster,
            self.start_cluster_number,
        )
    }
}
