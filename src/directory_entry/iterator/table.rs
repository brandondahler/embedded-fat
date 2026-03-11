#[cfg(test)]
mod tests;

use crate::Device;
use crate::directory_entry::{
    DIRECTORY_ENTRY_SIZE, DirectoryEntry, DirectoryEntryIterationError,
    DirectoryEntryIteratorResult,
};
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use {
    crate::SyncDevice,
    embedded_io::{Read, Seek},
};

#[cfg(feature = "async")]
use {
    crate::AsyncDevice,
    embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek},
};

#[derive(Clone, Debug)]
pub struct DirectoryTableEntryIterator<'a, D>
where
    D: Device,
{
    device: &'a D,

    start_address: u64,
    entry_count: u16,

    current_entry_index: Option<u16>,
}

impl<'a, D> DirectoryTableEntryIterator<'a, D>
where
    D: Device,
{
    pub fn new(device: &'a D, start_address: u64, entry_count: u16) -> Self {
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
impl<D, S> DirectoryTableEntryIterator<'_, D>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
{
    pub fn peek(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let current_address = self.current_address()?;
        let mut directory_entry_bytes = [0; DIRECTORY_ENTRY_SIZE];

        propagate_device_iteration_errors!(
            self.device
                .with_stream(|stream| -> DirectoryEntryIteratorResult<(), D> {
                    stream.seek(SeekFrom::Start(current_address))?;
                    stream.read_exact(&mut directory_entry_bytes)?;

                    Ok(())
                })
                .map_err(DirectoryEntryIterationError::DeviceError)
        );

        Some(Ok(propagate_iteration_error!(DirectoryEntry::from_bytes(
            &directory_entry_bytes
        ))))
    }

    pub fn next(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let result = self.peek();

        if result.is_some() {
            self.advance();
        }

        result
    }
}

#[cfg(feature = "async")]
impl<D, S> DirectoryTableEntryIterator<'_, D>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
{
    pub async fn peek_async(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let current_address = self.current_address()?;
        let mut directory_entry_bytes = [0; DIRECTORY_ENTRY_SIZE];

        // Weird, but we need to unwrap two results
        propagate_device_iteration_errors!(
            self.device
                .with_stream(async |stream| -> DirectoryEntryIteratorResult<(), D> {
                    stream.seek(SeekFrom::Start(current_address)).await?;
                    stream.read_exact(&mut directory_entry_bytes).await?;

                    Ok(())
                })
                .await
                .map_err(DirectoryEntryIterationError::DeviceError)
        );

        Some(Ok(propagate_iteration_error!(DirectoryEntry::from_bytes(
            &directory_entry_bytes
        ))))
    }

    pub async fn next_async(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let result = self.peek_async().await;

        if result.is_some() {
            self.advance();
        }

        result
    }
}
