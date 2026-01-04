mod entry_iteration_error;
mod entry_iterator;
mod file;
mod table;

pub use entry_iteration_error::*;
pub use entry_iterator::*;
pub use file::*;
pub use table::*;

use crate::device::{Device, SyncDevice};
use crate::directory_entry::DirectoryEntryIterator;
use crate::directory_item::DirectoryItemIterator;
use embedded_io::{ErrorType, Read, Seek};

pub enum Directory<'a, D>
where
    D: Device,
{
    Table(DirectoryTable<'a, D>),
    File(DirectoryFile<'a, D>),
}

impl<'a, D> Directory<'a, D>
where
    D: Device,
{
    pub fn items(&'a self) -> DirectoryItemIterator<'a, D> {
        DirectoryItemIterator::new(self.entries())
    }

    fn entries(&'a self) -> DirectoryEntryIterator<'a, D> {
        match self {
            Directory::Table(table) => table.entries().into(),
            Directory::File(file) => file.entries().into(),
        }
    }
}

impl<'a, D> From<DirectoryTable<'a, D>> for Directory<'a, D>
where
    D: Device,
{
    fn from(value: DirectoryTable<'a, D>) -> Self {
        Self::Table(value)
    }
}

impl<'a, D> From<DirectoryFile<'a, D>> for Directory<'a, D>
where
    D: Device,
{
    fn from(value: DirectoryFile<'a, D>) -> Self {
        Self::File(value)
    }
}
