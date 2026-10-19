#[cfg(test)]
mod tests;

use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, DirectoryEntry, DirectoryEntryIterationError};
use core::cell::RefCell;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

#[derive(Clone, Debug)]
pub struct DirectoryTableEntryIterator<'a, S>
where
    S: ErrorType,
{
    device: &'a RefCell<S>,

    start_address: u64,
    entry_count: u16,

    current_entry_index: Option<u16>,
}

impl<'a, S> DirectoryTableEntryIterator<'a, S>
where
    S: ErrorType,
{
    pub fn new(device: &'a RefCell<S>, start_address: u64, entry_count: u16) -> Self {
        Self {
            device,

            start_address,
            entry_count,

            current_entry_index: if entry_count > 0 { Some(0) } else { None },
        }
    }

    pub fn advance(&mut self) -> bool {
        let next_entry_index = match self.current_entry_index {
            Some(current_entry_index) => current_entry_index + 1,
            None => return false,
        };

        self.current_entry_index = if next_entry_index < self.entry_count {
            Some(next_entry_index)
        } else {
            None
        };

        self.current_entry_index.is_some()
    }

    fn current_address(&self) -> Option<u64> {
        self.current_entry_index.map(|current_entry_index| {
            self.start_address + (current_entry_index as u64 * DIRECTORY_ENTRY_SIZE as u64)
        })
    }
}

#[cfg(feature = "sync")]
impl<S> DirectoryTableEntryIterator<'_, S>
where
    S: Read + Seek,
{
    pub fn peek(&self) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let current_address = match self.current_address() {
            Some(address) => address,
            None => return Ok(None),
        };

        let mut directory_entry_bytes = [0; DIRECTORY_ENTRY_SIZE];

        {
            let mut stream = self.device.borrow_mut();

            stream.seek(SeekFrom::Start(current_address))?;
            stream.read_exact(&mut directory_entry_bytes)?;
        }

        let directory_entry = DirectoryEntry::from_bytes(&directory_entry_bytes)?;

        Ok(Some(directory_entry))
    }

    pub fn next(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let result = self.peek()?;

        if result.is_some() {
            self.advance();
        }

        Ok(result)
    }
}

#[cfg(feature = "async")]
impl<S> DirectoryTableEntryIterator<'_, S>
where
    S: AsyncRead + AsyncSeek,
{
    pub async fn peek_async(
        &self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let current_address = match self.current_address() {
            Some(address) => address,
            None => return Ok(None),
        };

        let mut directory_entry_bytes = [0; DIRECTORY_ENTRY_SIZE];

        {
            let mut stream = self.device.borrow_mut();

            stream.seek(SeekFrom::Start(current_address)).await?;
            stream.read_exact(&mut directory_entry_bytes).await?;
        }

        let directory_entry = DirectoryEntry::from_bytes(&directory_entry_bytes)?;

        Ok(Some(directory_entry))
    }

    pub async fn next_async(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let result = self.peek_async().await?;

        if result.is_some() {
            self.advance();
        }

        Ok(result)
    }
}
