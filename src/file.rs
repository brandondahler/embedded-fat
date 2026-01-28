mod error;

pub use error::*;

use crate::Device;
use crate::allocation_table::{AllocationTable, AllocationTableEntry};
use core::cmp::min;
use core::ops::DerefMut;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use {
    crate::{SyncDevice, SyncFlushableDevice},
    embedded_io::{Read, Seek, Write},
};

#[cfg(feature = "async")]
use {
    crate::{AsyncDevice, AsyncFlushableDevice},
    embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite},
};

#[derive(Clone, Debug)]
pub struct File<'a, D>
where
    D: Device,
{
    device: &'a D,
    allocation_table: &'a AllocationTable,

    data_region_base_address: u64,
    bytes_per_cluster: u32,

    first_cluster_number: u32,
    file_size: u32,

    current_position: u32,

    current_cluster_number: u32,
    current_cluster_offset: u32,
}

impl<'a, D> File<'a, D>
where
    D: Device,
{
    pub fn new(
        device: &'a D,
        allocation_table: &'a AllocationTable,
        data_region_base_address: u64,
        bytes_per_cluster: u32,
        first_cluster_number: u32,
        file_size: u32,
    ) -> Self {
        Self {
            device,
            allocation_table,

            data_region_base_address,
            bytes_per_cluster,

            first_cluster_number,
            file_size,

            current_position: 0,

            current_cluster_number: first_cluster_number,
            current_cluster_offset: 0,
        }
    }

    fn current_address(&self) -> u64 {
        self.data_region_base_address
            + ((self.current_cluster_number - 2) as u64 * self.bytes_per_cluster as u64)
            + self.current_cluster_offset as u64
    }

    fn resolve_max_read_size(&self, target_buffer_length: usize) -> usize {
        min(
            min(
                target_buffer_length.try_into().unwrap_or(u32::MAX),
                self.file_size - self.current_position,
            ),
            self.bytes_per_cluster - self.current_cluster_offset,
        ) as usize
    }

    fn resolve_desired_position(&self, pos: SeekFrom) -> Result<u32, <Self as ErrorType>::Error> {
        let desired_address: u64 = match pos {
            SeekFrom::Start(desired_address) => desired_address,
            SeekFrom::Current(offset) => {
                let desired_address: i64 = self.current_position as i64 + offset;

                desired_address
                    .try_into()
                    .map_err(|_| FileError::SeekPositionImpossible(desired_address))?
            }
            SeekFrom::End(end_offset) => {
                let desired_address: i64 = self.file_size as i64 + end_offset;

                desired_address
                    .try_into()
                    .map_err(|_| FileError::SeekPositionImpossible(desired_address))?
            }
        };

        desired_address
            .try_into()
            .map_err(|_| FileError::SeekPositionBeyondLimits(desired_address))
    }
}

impl<D> ErrorType for File<'_, D>
where
    D: Device,
{
    type Error = FileError<D::Error, <D::Stream as ErrorType>::Error>;
}

#[cfg(feature = "sync")]
impl<D, S> Read for File<'_, D>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // Limit to either the end of the file or the end of the current cluster
        let target_read_size = self.resolve_max_read_size(buf.len());

        if target_read_size == 0 {
            return Ok(0);
        }

        let actual_read_size = self
            .device
            .with_stream(|stream| -> Result<usize, Self::Error> {
                stream.seek(SeekFrom::Start(self.current_address()))?;

                Ok(stream.read(&mut buf[0..target_read_size])?)
            })
            .map_err(FileError::DeviceError)??;

        self.seek(SeekFrom::Current(actual_read_size as i64))?;

        Ok(actual_read_size)
    }
}

#[cfg(feature = "async")]
impl<D, S> AsyncRead for File<'_, D>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let target_read_size = self.resolve_max_read_size(buf.len());

        if target_read_size == 0 {
            return Ok(0);
        }

        let actual_read_size = self
            .device
            .with_stream(async |stream| -> Result<usize, Self::Error> {
                stream.seek(SeekFrom::Start(self.current_address())).await?;

                Ok(stream.read(&mut buf[0..target_read_size]).await?)
            })
            .await
            .map_err(FileError::DeviceError)??;

        self.seek(SeekFrom::Current(actual_read_size as i64))
            .await?;

        Ok(actual_read_size)
    }
}

#[cfg(feature = "sync")]
impl<D, S> Seek for File<'_, D>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let desired_position = self.resolve_desired_position(pos)?;
        let relative_position_change = desired_position as i64 - self.current_position as i64;

        if relative_position_change == 0 {
            return Ok(self.current_position.into());
        }

        let mut new_cluster_number = self.current_cluster_number;
        let mut new_cluster_offset: i64 =
            self.current_cluster_offset as i64 + relative_position_change;
        let is_inside_current_cluster =
            new_cluster_offset >= 0 && new_cluster_offset < self.bytes_per_cluster as i64;

        if !is_inside_current_cluster {
            if relative_position_change < 0 {
                // Rewind back to the start
                new_cluster_number = self.first_cluster_number;
                new_cluster_offset = desired_position as i64;
            }

            self.device
                .with_stream(|stream| -> Result<(), Self::Error> {
                    // Navigate forward until we get to the correct cluster or reach EOF
                    while new_cluster_offset > self.bytes_per_cluster as i64 {
                        match self
                            .allocation_table
                            .read_entry(stream, new_cluster_number)?
                        {
                            AllocationTableEntry::NextClusterNumber(next_cluster_number) => {
                                new_cluster_number = next_cluster_number
                                    .value(self.allocation_table.kind())
                                    .unwrap();
                                new_cluster_offset -= self.bytes_per_cluster as i64;
                            }
                            AllocationTableEntry::EndOfFile => break,
                            AllocationTableEntry::Free
                            | AllocationTableEntry::BadSector
                            | AllocationTableEntry::Reserved => {
                                return Err(FileError::UnexpectedAllocationTableEntryEncountered);
                            }
                        }
                    }

                    Ok(())
                })
                .map_err(FileError::DeviceError)??;

            // Clamp to the end of the cluster if the offset is beyond the cluster's end still
            new_cluster_offset = min(new_cluster_offset, self.bytes_per_cluster as i64);
        }

        self.current_cluster_number = new_cluster_number;
        self.current_cluster_offset = new_cluster_offset as u32;
        self.current_position = desired_position;

        Ok(desired_position.into())
    }
}

#[cfg(feature = "async")]
impl<D, S> AsyncSeek for File<'_, D>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
{
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let desired_position: u32 = self.resolve_desired_position(pos)?;
        let relative_position_change = desired_position as i64 - self.current_position as i64;

        if relative_position_change == 0 {
            return Ok(self.current_position.into());
        }

        let mut new_cluster_number = self.current_cluster_number;
        let mut new_cluster_offset: i64 =
            self.current_cluster_offset as i64 + relative_position_change;
        let is_inside_current_cluster =
            new_cluster_offset >= 0 && new_cluster_offset < self.bytes_per_cluster as i64;

        if !is_inside_current_cluster {
            if relative_position_change < 0 {
                // Rewind back to the start
                new_cluster_number = self.first_cluster_number;
                new_cluster_offset = desired_position as i64;
            }

            self.device
                .with_stream(async |stream| -> Result<(), Self::Error> {
                    // Navigate forward until we get to the correct cluster or reach EOF
                    while new_cluster_offset >= self.bytes_per_cluster as i64 {
                        match self
                            .allocation_table
                            .read_entry_async(stream, new_cluster_number)
                            .await?
                        {
                            AllocationTableEntry::NextClusterNumber(next_cluster_number) => {
                                new_cluster_number = next_cluster_number
                                    .value(self.allocation_table.kind())
                                    .unwrap();
                                new_cluster_offset -= self.bytes_per_cluster as i64;
                            }
                            AllocationTableEntry::EndOfFile => break,
                            AllocationTableEntry::Free
                            | AllocationTableEntry::BadSector
                            | AllocationTableEntry::Reserved => {
                                return Err(FileError::UnexpectedAllocationTableEntryEncountered);
                            }
                        }
                    }

                    Ok(())
                })
                .await
                .map_err(FileError::DeviceError)??;

            // Clamp to the end of the cluster if the offset is beyond the cluster's end still
            new_cluster_offset = min(new_cluster_offset, self.bytes_per_cluster as i64);
        }

        self.current_cluster_number = new_cluster_number;
        self.current_cluster_offset = new_cluster_offset as u32;
        self.current_position = desired_position;

        Ok(desired_position.into())
    }
}

#[cfg(feature = "sync")]
impl<D, S> Write for File<'_, D>
where
    D: SyncFlushableDevice<Stream = S>,
    S: Read + Seek + Write,
{
    fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.device.flush().map_err(FileError::DeviceError)
    }
}

#[cfg(feature = "async")]
impl<D, S> AsyncWrite for File<'_, D>
where
    D: AsyncFlushableDevice<Stream = S>,
    S: AsyncRead + AsyncSeek + AsyncWrite,
{
    async fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.device.flush().await.map_err(FileError::DeviceError)
    }
}
