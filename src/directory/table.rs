use crate::device::Device;
use crate::directory_entry::DirectoryTableEntryIterator;

#[derive(Clone, Debug)]
pub struct DirectoryTable<'a, D>
where
    D: Device,
{
    device: &'a D,

    start_address: u32,
    entry_count: u16,
}

impl<'a, D> DirectoryTable<'a, D>
where
    D: Device,
{
    pub fn new(device: &'a D, start_address: u32, entry_count: u16) -> Self {
        Self {
            device,

            start_address,
            entry_count,
        }
    }

    pub fn entries(&self) -> DirectoryTableEntryIterator<'_, D> {
        DirectoryTableEntryIterator::new(self.device, self.start_address, self.entry_count)
    }
}
