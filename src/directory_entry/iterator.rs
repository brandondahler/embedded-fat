mod error;
mod file;
mod table;

#[cfg(test)]
mod tests;

pub use error::*;
pub use file::*;
pub use table::*;

use crate::directory_entry::DirectoryEntry;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

#[cfg(test)]
use crate::mock::ScriptedDirectoryEntryIterator;

#[derive(Clone, Debug)]
pub enum DirectoryEntryIterator<'a, S, const N: usize>
where
    S: ErrorType,
{
    Table(DirectoryTableEntryIterator<'a, S, N>),
    File(DirectoryFileEntryIterator<'a, S, N>),

    #[cfg(test)]
    Scripted(ScriptedDirectoryEntryIterator<'a, S>),
}

#[cfg(feature = "sync")]
impl<S, const N: usize> DirectoryEntryIterator<'_, S, N>
where
    S: Read + Seek,
{
    pub fn peek(&self) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.peek(),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.peek(),

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => scripted_iterator.peek(),
        }
    }

    pub fn advance(&mut self) -> Result<bool, DirectoryEntryIterationError<S::Error>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => Ok(table_iterator.advance()),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.advance(),

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => scripted_iterator.advance(),
        }
    }

    pub fn next(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.next(),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.next(),

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => scripted_iterator.next(),
        }
    }
}

#[cfg(feature = "async")]
impl<S, const N: usize> DirectoryEntryIterator<'_, S, N>
where
    S: AsyncRead + AsyncSeek,
{
    pub async fn peek_async(
        &self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.peek_async().await,
            DirectoryEntryIterator::File(file_iterator) => file_iterator.peek_async().await,

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => {
                scripted_iterator.peek_async().await
            }
        }
    }

    pub async fn advance_async(&mut self) -> Result<bool, DirectoryEntryIterationError<S::Error>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => Ok(table_iterator.advance()),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.advance_async().await,

            #[cfg(test)]
            DirectoryEntryIterator::Scripted(scripted_iterator) => {
                scripted_iterator.advance_async().await
            }
        }
    }

    pub async fn next_async(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
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

impl<'a, S, const N: usize> From<DirectoryTableEntryIterator<'a, S, N>>
    for DirectoryEntryIterator<'a, S, N>
where
    S: ErrorType,
{
    fn from(value: DirectoryTableEntryIterator<'a, S, N>) -> Self {
        Self::Table(value)
    }
}

impl<'a, S, const N: usize> From<DirectoryFileEntryIterator<'a, S, N>>
    for DirectoryEntryIterator<'a, S, N>
where
    S: ErrorType,
{
    fn from(value: DirectoryFileEntryIterator<'a, S, N>) -> Self {
        Self::File(value)
    }
}

#[cfg(test)]
impl<'a, S> From<ScriptedDirectoryEntryIterator<'a, S>> for DirectoryEntryIterator<'a, S, 0>
where
    S: ErrorType,
{
    fn from(value: ScriptedDirectoryEntryIterator<'a, S>) -> Self {
        Self::Scripted(value)
    }
}
