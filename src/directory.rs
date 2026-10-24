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
pub enum Directory<'a, S, const N: usize>
where
    S: ErrorType,
{
    Table(DirectoryTable<'a, S, N>),
    File(DirectoryFile<'a, S, N>),
}

impl<'a, S, const N: usize> Directory<'a, S, N>
where
    S: ErrorType,
{
    pub fn items(&'a self) -> DirectoryItemIterator<'a, S, N> {
        DirectoryItemIterator::new(self.entries())
    }

    fn entries(&'a self) -> DirectoryEntryIterator<'a, S, N> {
        match self {
            Directory::Table(table) => table.entries().into(),
            Directory::File(file) => file.entries().into(),
        }
    }
}

impl<'a, S, const N: usize> From<DirectoryTable<'a, S, N>> for Directory<'a, S, N>
where
    S: ErrorType,
{
    fn from(value: DirectoryTable<'a, S, N>) -> Self {
        Self::Table(value)
    }
}

impl<'a, S, const N: usize> From<DirectoryFile<'a, S, N>> for Directory<'a, S, N>
where
    S: ErrorType,
{
    fn from(value: DirectoryFile<'a, S, N>) -> Self {
        Self::File(value)
    }
}
