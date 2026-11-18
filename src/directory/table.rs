#[cfg(test)]
mod tests;

use crate::directory_entry::DirectoryTableEntryIterator;
use crate::filesystem_state::FilesystemState;
use crate::queued_resource::QueuedResource;
use core::cell::RefCell;
use embedded_io::ErrorType;

#[derive(Clone, Debug)]
pub struct DirectoryTable<'a, S, const N: usize>
where
    S: ErrorType,
{
    state: &'a QueuedResource<FilesystemState<S>, N>,

    start_address: u64,
    entry_count: u16,
}

impl<'a, S, const N: usize> DirectoryTable<'a, S, N>
where
    S: ErrorType,
{
    pub fn new(
        state: &'a QueuedResource<FilesystemState<S>, N>,
        start_address: u64,
        entry_count: u16,
    ) -> Self {
        Self {
            state,

            start_address,
            entry_count,
        }
    }

    pub fn entries(&self) -> DirectoryTableEntryIterator<'_, S, N> {
        DirectoryTableEntryIterator::new(self.state, self.start_address, self.entry_count)
    }
}
