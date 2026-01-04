use crate::allocation_table::{AllocationTable, AllocationTableEntry};
use crate::device::{AsyncDevice, Device, SyncDevice};
use crate::directory_entry::{
    DIRECTORY_ENTRY_SIZE, DirectoryEntry, DirectoryEntryIterationError,
    DirectoryEntryIteratorResult,
};
use core::cell::RefCell;
use core::ops::DerefMut;
use embedded_io::{ErrorType, Read, Seek, SeekFrom};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

pub struct DirectoryFileEntryIterator<'a, D> {
    device: &'a D,
    allocation_table: &'a AllocationTable,

    data_region_base_address: u32,
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
        data_region_base_address: u32,
        bytes_per_cluster: u32,
        first_cluster_number: u32,
    ) -> Self {
        Self {
            device,
            allocation_table,

            data_region_base_address,
            bytes_per_cluster,

            current_cluster_number: first_cluster_number,
            current_cluster_offset: 0,
        }
    }

    fn current_address(&self) -> u32 {
        self.data_region_base_address
            + ((self.current_cluster_number - 2) * self.bytes_per_cluster)
            + self.current_cluster_offset
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
