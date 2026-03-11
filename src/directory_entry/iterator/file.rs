#[cfg(test)]
mod tests;

use crate::Device;
use crate::allocation_table::{AllocationTable, AllocationTableEntry};
use crate::directory_entry::{
    DIRECTORY_ENTRY_SIZE, DirectoryEntry, DirectoryEntryIterationError,
    DirectoryEntryIteratorResult,
};
use core::ops::DerefMut;
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
pub struct DirectoryFileEntryIterator<'a, D>
where
    D: Device,
{
    device: &'a D,
    allocation_table: &'a AllocationTable,

    data_region_base_address: u64,
    bytes_per_cluster: u32,

    current_cluster_number: u32,
    current_cluster_offset: u32,
}

impl<'a, D> DirectoryFileEntryIterator<'a, D>
where
    D: Device,
{
    pub fn new(
        device: &'a D,
        allocation_table: &'a AllocationTable,
        data_region_base_address: u64,
        bytes_per_cluster: u32,
        start_cluster_number: u32,
    ) -> Self {
        Self {
            device,
            allocation_table,

            data_region_base_address,
            bytes_per_cluster,

            current_cluster_number: start_cluster_number,
            current_cluster_offset: 0,
        }
    }

    fn current_address(&self) -> u64 {
        self.data_region_base_address
            + ((self.current_cluster_number - 2) as u64 * self.bytes_per_cluster as u64)
            + self.current_cluster_offset as u64
    }

    fn advance_offset(&mut self) {
        self.current_cluster_offset += DIRECTORY_ENTRY_SIZE as u32;
    }

    fn try_advance_cluster(
        &mut self,
        allocation_table_entry: AllocationTableEntry,
    ) -> DirectoryEntryIteratorResult<bool, D> {
        match allocation_table_entry {
            AllocationTableEntry::NextClusterNumber(next_cluster_number) => {
                self.current_cluster_number = next_cluster_number;
                self.current_cluster_offset = 0;

                Ok(true)
            }
            AllocationTableEntry::EndOfFile => Ok(false),
            AllocationTableEntry::Free
            | AllocationTableEntry::BadSector
            | AllocationTableEntry::Reserved => {
                Err(DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected)
            }
        }
    }
}

#[cfg(feature = "sync")]
impl<'a, D, S> DirectoryFileEntryIterator<'a, D>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
{
    pub fn peek(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        if self.current_cluster_offset >= self.bytes_per_cluster {
            return None;
        }

        let current_address = self.current_address();

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

    pub fn advance(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        self.advance_offset();

        if self.current_cluster_offset < self.bytes_per_cluster {
            return Ok(true);
        }

        self.device
            .with_stream(|stream| -> DirectoryEntryIteratorResult<bool, D> {
                let allocation_table_entry = self
                    .allocation_table
                    .read_entry(stream, self.current_cluster_number)?;

                self.try_advance_cluster(allocation_table_entry)
            })
            .map_err(DirectoryEntryIterationError::DeviceError)?
    }

    pub fn next(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let result = self.peek();

        if result.is_some() {
            propagate_iteration_error!(self.advance());
        }

        result
    }
}

#[cfg(feature = "async")]
impl<'a, D, S> DirectoryFileEntryIterator<'a, D>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
{
    pub async fn peek_async(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        if self.current_cluster_offset >= self.bytes_per_cluster {
            return None;
        }

        let current_address = self.current_address();
        let mut directory_entry_bytes = [0; DIRECTORY_ENTRY_SIZE];

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

    pub async fn advance_async(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        self.advance_offset();

        if self.current_cluster_offset < self.bytes_per_cluster {
            return Ok(true);
        }

        self.device
            .with_stream(async |stream| -> DirectoryEntryIteratorResult<bool, D> {
                let allocation_table_entry = self
                    .allocation_table
                    .read_entry_async(stream, self.current_cluster_number)
                    .await?;

                self.try_advance_cluster(allocation_table_entry)
            })
            .await
            .map_err(DirectoryEntryIterationError::DeviceError)?
    }

    pub async fn next_async(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        let result = self.peek_async().await;

        if result.is_some() {
            propagate_iteration_error!(self.advance_async().await);
        }

        result
    }
}
