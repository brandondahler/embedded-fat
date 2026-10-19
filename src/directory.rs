mod file;
mod table;

#[cfg(test)]
mod tests;

pub use file::*;
pub use table::*;

use crate::directory_entry::DirectoryEntryIterator;
use crate::directory_item::DirectoryItemIterator;
use embedded_io::ErrorType;

#[derive(Clone, Debug)]
pub enum Directory<'a, S>
where
    S: ErrorType,
{
    Table(DirectoryTable<'a, S>),
    File(DirectoryFile<'a, S>),
}

impl<'a, S> Directory<'a, S>
where
    S: ErrorType,
{
    pub fn items(&'a self) -> DirectoryItemIterator<'a, S> {
        DirectoryItemIterator::new(self.entries())
    }

    fn entries(&'a self) -> DirectoryEntryIterator<'a, S> {
        match self {
            Directory::Table(table) => table.entries().into(),
            Directory::File(file) => file.entries().into(),
        }
    }
}

impl<'a, S> From<DirectoryTable<'a, S>> for Directory<'a, S>
where
    S: ErrorType,
{
    fn from(value: DirectoryTable<'a, S>) -> Self {
        Self::Table(value)
    }
}

impl<'a, S> From<DirectoryFile<'a, S>> for Directory<'a, S>
where
    S: ErrorType,
{
    fn from(value: DirectoryFile<'a, S>) -> Self {
        Self::File(value)
    }
}
