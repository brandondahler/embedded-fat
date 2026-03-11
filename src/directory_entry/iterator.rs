mod error;
mod file;
mod table;

#[cfg(test)]
mod tests;

pub use error::*;
pub use file::*;
pub use table::*;

use crate::Device;
use crate::directory_entry::DirectoryEntry;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use {
    crate::device::SyncDevice,
    embedded_io::{Read, Seek},
};

#[cfg(feature = "async")]
use {
    crate::AsyncDevice,
    embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek},
};

#[cfg(test)]
use crate::mock::ScriptedDirectoryEntryIterator;

pub type DirectoryEntryIteratorResult<R, D> = Result<
    R,
    DirectoryEntryIterationError<<D as Device>::Error, <<D as Device>::Stream as ErrorType>::Error>,
>;

#[derive(Clone, Debug)]
pub enum DirectoryEntryIterator<'a, D>
where
    D: Device,
{
    Table(DirectoryTableEntryIterator<'a, D>),
    File(DirectoryFileEntryIterator<'a, D>),

    #[cfg(test)]
    Scripted(ScriptedDirectoryEntryIterator<'a, D>),
}

#[cfg(feature = "sync")]
impl<D, S> DirectoryEntryIterator<'_, D>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
{
    pub fn peek(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.peek(),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.peek(),

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => scripted_iterator.peek(),
        }
    }

    pub fn advance(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => Ok(table_iterator.advance()),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.advance(),

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => scripted_iterator.advance(),
        }
    }

    pub fn next(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.next(),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.next(),

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => scripted_iterator.next(),
        }
    }
}

#[cfg(feature = "async")]
impl<D, S> DirectoryEntryIterator<'_, D>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
{
    pub async fn peek_async(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.peek_async().await,
            DirectoryEntryIterator::File(file_iterator) => file_iterator.peek_async().await,

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => {
                scripted_iterator.peek_async().await
            }
        }
    }

    pub async fn advance_async(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => Ok(table_iterator.advance()),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.advance_async().await,

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => {
                scripted_iterator.advance_async().await
            }
        }
    }

    pub async fn next_async(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.next_async().await,
            DirectoryEntryIterator::File(file_iterator) => file_iterator.next_async().await,

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => {
                scripted_iterator.next_async().await
            }
        }
    }
}

impl<'a, D> From<DirectoryTableEntryIterator<'a, D>> for DirectoryEntryIterator<'a, D>
where
    D: Device,
{
    fn from(value: DirectoryTableEntryIterator<'a, D>) -> Self {
        Self::Table(value)
    }
}

impl<'a, D> From<DirectoryFileEntryIterator<'a, D>> for DirectoryEntryIterator<'a, D>
where
    D: Device,
{
    fn from(value: DirectoryFileEntryIterator<'a, D>) -> Self {
        Self::File(value)
    }
}

#[cfg(test)]
impl<'a, D> From<ScriptedDirectoryEntryIterator<'a, D>> for DirectoryEntryIterator<'a, D>
where
    D: Device,
{
    fn from(value: ScriptedDirectoryEntryIterator<'a, D>) -> Self {
        Self::Scripted(value)
    }
}
