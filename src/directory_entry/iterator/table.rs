use crate::device::{AsyncDevice, Device, SyncDevice};
use crate::directory_entry::{
    DIRECTORY_ENTRY_SIZE, DirectoryEntry, DirectoryEntryIterationError,
    DirectoryEntryIteratorResult,
};
use core::cell::RefCell;
use embedded_io::{ErrorType, Read, Seek, SeekFrom};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

pub struct DirectoryTableEntryIterator<'a, D>
where
    D: Device,
{
    device: &'a D,

    start_address: u32,
    entry_count: u16,

    next_entry_index: Option<u16>,
}

impl<'a, D> DirectoryTableEntryIterator<'a, D>
where
    D: Device,
{
    pub fn new(device: &'a D, start_address: u32, entry_count: u16) -> Self {
        assert!(
            entry_count > 0,
            "DirectoryTable must have at least one entry"
        );

        Self {
            device,

            start_address,
            entry_count,

            next_entry_index: Some(0),
        }
    }

    pub fn advance(&mut self) -> bool {
        let next_entry_index = match self.next_entry_index {
            Some(next_entry_index) => next_entry_index + 1,
            None => {
                return false;
            }
        };

        self.next_entry_index = if next_entry_index < self.entry_count {
            Some(next_entry_index)
        } else {
            None
        };

        true
    }

    fn current_address(&self) -> Option<u32> {
        let current_entry_index = self.next_entry_index?;

        Some(self.start_address + (current_entry_index as u32 * DIRECTORY_ENTRY_SIZE as u32))
    }
}

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
                    stream.seek(SeekFrom::Start(current_address as u64))?;
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
                    stream.seek(SeekFrom::Start(current_address as u64)).await?;
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
