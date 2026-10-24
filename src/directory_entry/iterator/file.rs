#[cfg(test)]
mod tests;

use crate::allocation_table::{AllocationTable, AllocationTableEntry};
use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, DirectoryEntry, DirectoryEntryIterationError};
use core::cell::RefCell;
use core::ops::DerefMut;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

use crate::FileError;
use crate::device::Device;
#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

#[derive(Clone, Debug)]
pub struct DirectoryFileEntryIterator<'a, S, const N: usize>
where
    S: ErrorType,
{
    device: &'a Device<S, N>,
    allocation_table: &'a AllocationTable,

    data_region_base_address: u64,
    bytes_per_cluster: u32,

    current_cluster_number: u32,
    current_cluster_offset: u32,
}

impl<'a, S, const N: usize> DirectoryFileEntryIterator<'a, S, N>
where
    S: ErrorType,
{
    pub fn new(
        device: &'a Device<S, N>,
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
    ) -> Result<bool, DirectoryEntryIterationError<S::Error>> {
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
impl<'a, S, const N: usize> DirectoryFileEntryIterator<'a, S, N>
where
    S: Read + Seek,
{
    pub fn peek(&self) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        if self.current_cluster_offset >= self.bytes_per_cluster {
            return Ok(None);
        }

        let current_address = self.current_address();

        let mut directory_entry_bytes = [0; DIRECTORY_ENTRY_SIZE];

        {
            let mut stream = self
                .device
                .acquire_stream()
                .map_err(|_| DirectoryEntryIterationError::DeviceBusy)?;

            stream.seek(SeekFrom::Start(current_address))?;
            stream.read_exact(&mut directory_entry_bytes)?;
        }

        let directory_entry = DirectoryEntry::from_bytes(&directory_entry_bytes)?;

        Ok(Some(directory_entry))
    }

    pub fn advance(&mut self) -> Result<bool, DirectoryEntryIterationError<S::Error>> {
        self.advance_offset();

        if self.current_cluster_offset < self.bytes_per_cluster {
            return Ok(true);
        }

        {
            let mut stream = self
                .device
                .acquire_stream()
                .map_err(|_| DirectoryEntryIterationError::DeviceBusy)?;

            let allocation_table_entry = self
                .allocation_table
                .read_entry(&mut *stream, self.current_cluster_number)?;

            self.try_advance_cluster(allocation_table_entry)
        }
    }

    pub fn next(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let result = self.peek()?;

        if result.is_some() {
            self.advance()?;
        }

        Ok(result)
    }
}

#[cfg(feature = "async")]
impl<'a, S, const N: usize> DirectoryFileEntryIterator<'a, S, N>
where
    S: AsyncRead + AsyncSeek,
{
    pub async fn peek_async(
        &self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        if self.current_cluster_offset >= self.bytes_per_cluster {
            return Ok(None);
        }

        let current_address = self.current_address();
        let mut directory_entry_bytes = [0; DIRECTORY_ENTRY_SIZE];

        {
            let mut stream = self
                .device
                .acquire_stream_async()
                .await
                .map_err(|_| DirectoryEntryIterationError::DeviceBusy)?;

            stream.seek(SeekFrom::Start(current_address)).await?;
            stream.read_exact(&mut directory_entry_bytes).await?;
        }

        let directory_entry = DirectoryEntry::from_bytes(&directory_entry_bytes)?;

        Ok(Some(directory_entry))
    }

    pub async fn advance_async(&mut self) -> Result<bool, DirectoryEntryIterationError<S::Error>> {
        self.advance_offset();

        if self.current_cluster_offset < self.bytes_per_cluster {
            return Ok(true);
        }

        {
            let mut stream = self
                .device
                .acquire_stream_async()
                .await
                .map_err(|_| DirectoryEntryIterationError::DeviceBusy)?;

            let allocation_table_entry = self
                .allocation_table
                .read_entry_async(&mut *stream, self.current_cluster_number)
                .await?;

            self.try_advance_cluster(allocation_table_entry)
        }
    }

    pub async fn next_async(
        &mut self,
    ) -> Result<Option<DirectoryEntry>, DirectoryEntryIterationError<S::Error>> {
        let result = self.peek_async().await?;

        if result.is_some() {
            self.advance_async().await?;
        }

        Ok(result)
    }
}
