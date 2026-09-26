mod entry;
mod entry_offset;
mod kind;
mod physical_entry;
mod read_error;
mod update_error;

#[cfg(test)]
mod tests;

use core::cmp::min;
pub use entry::*;
pub use entry_offset::*;
pub use kind::*;
pub use physical_entry::*;
pub use read_error::*;
pub use update_error::*;

use crate::utils::read_le_u32;

#[cfg(any(feature = "sync", feature = "async"))]
use embedded_io::SeekFrom;

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek, Write};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};

#[derive(Clone, Debug)]
pub struct AllocationTable {
    kind: AllocationTableKind,
    base_address: u64,
    last_cluster_number: u32,

    free_cluster_scan_start_number: u32,
}

impl AllocationTable {
    pub fn new(kind: AllocationTableKind, base_address: u64, last_cluster_number: u32) -> Self {
        assert!(
            last_cluster_number >= 1,
            "last_cluster_number must be greater than 0"
        );
        assert!(last_cluster_number <= kind.entry_mask());

        Self {
            kind,
            base_address,
            last_cluster_number,
            free_cluster_scan_start_number: 2,
        }
    }

    pub(crate) fn kind(&self) -> AllocationTableKind {
        self.kind
    }

    #[cfg(feature = "sync")]
    pub fn read_entry<S>(
        &self,
        stream: &mut S,
        cluster_number: u32,
    ) -> Result<AllocationTableEntry, AllocationTableReadError<S::Error>>
    where
        S: Read + Seek,
    {
        let mut entry_value_bytes = [0u8; 4];
        let entry_offest = self.resolve_entry_offset(cluster_number);

        stream.seek(SeekFrom::Start(
            self.base_address + entry_offest.byte_offset,
        ))?;

        match self.kind {
            AllocationTableKind::Fat12 | AllocationTableKind::Fat16 => {
                stream.read_exact(&mut entry_value_bytes[0..2])?;
            }
            AllocationTableKind::Fat32 => {
                stream.read_exact(&mut entry_value_bytes)?;
            }
        }

        Ok(PhysicalAllocationTableEntry::from_bytes(
            self.kind,
            &entry_value_bytes,
            entry_offest.is_nibble_offset,
        )
        .as_logical_entry())
    }

    #[cfg(feature = "async")]
    pub async fn read_entry_async<S>(
        &self,
        stream: &mut S,
        cluster_number: u32,
    ) -> Result<AllocationTableEntry, AllocationTableReadError<S::Error>>
    where
        S: AsyncRead + AsyncSeek,
    {
        let mut entry_value_bytes = [0u8; 4];
        let entry_offset = self.resolve_entry_offset(cluster_number);

        stream
            .seek(SeekFrom::Start(
                self.base_address + entry_offset.byte_offset,
            ))
            .await?;

        match self.kind {
            AllocationTableKind::Fat12 | AllocationTableKind::Fat16 => {
                stream.read_exact(&mut entry_value_bytes[0..2]).await?;
            }
            AllocationTableKind::Fat32 => {
                stream.read_exact(&mut entry_value_bytes).await?;
            }
        }

        Ok(PhysicalAllocationTableEntry::from_bytes(
            self.kind,
            &entry_value_bytes,
            entry_offset.is_nibble_offset,
        )
        .as_logical_entry())
    }

    #[cfg(feature = "sync")]
    pub fn find_free_entry<S>(
        &mut self,
        stream: &mut S,
    ) -> Result<Option<u32>, AllocationTableReadError<S::Error>>
    where
        S: Read + Seek,
    {
        for cluster_number in self.free_cluster_scan_start_number..=self.last_cluster_number {
            let entry = self.read_entry(stream, cluster_number)?;

            if matches!(entry, AllocationTableEntry::Free) {
                self.free_cluster_scan_start_number = cluster_number;
                return Ok(Some(cluster_number));
            }
        }

        self.free_cluster_scan_start_number = self.last_cluster_number + 1;
        Ok(None)
    }

    #[cfg(feature = "async")]
    pub async fn find_free_entry_async<S>(
        &mut self,
        stream: &mut S,
    ) -> Result<Option<u32>, AllocationTableReadError<S::Error>>
    where
        S: AsyncRead + AsyncSeek,
    {
        for cluster_number in self.free_cluster_scan_start_number..=self.last_cluster_number {
            let entry = self.read_entry_async(stream, cluster_number).await?;

            if matches!(entry, AllocationTableEntry::Free) {
                self.free_cluster_scan_start_number = cluster_number;
                return Ok(Some(cluster_number));
            }
        }

        Ok(None)
    }

    #[cfg(feature = "sync")]
    pub fn update_entry<S>(
        &mut self,
        stream: &mut S,
        cluster_number: u32,
        allocation_table_entry: AllocationTableEntry,
    ) -> Result<(), AllocationTableUpdateError<S::Error>>
    where
        S: Read + Seek + Write,
    {
        let physical_entry = allocation_table_entry.as_physical_entry(self.kind)?;
        let entry_offset = self.resolve_entry_offset(cluster_number);

        let mut bytes = [0; 4];

        if matches!(self.kind, AllocationTableKind::Fat12) {
            stream.seek(SeekFrom::Start(entry_offset.byte_offset))?;
            stream.read_exact(&mut bytes[0..2])?;
        };

        physical_entry.write(&mut bytes, entry_offset.is_nibble_offset);

        match self.kind {
            AllocationTableKind::Fat12 | AllocationTableKind::Fat16 => {
                stream.seek(SeekFrom::Start(entry_offset.byte_offset))?;
                stream.write_all(&bytes[0..2])?;
            }
            AllocationTableKind::Fat32 => {
                stream.seek(SeekFrom::Start(entry_offset.byte_offset))?;
                stream.write_all(&bytes)?;
            }
        };

        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn update_entry_async<S>(
        &mut self,
        stream: &mut S,
        cluster_number: u32,
        allocation_table_entry: AllocationTableEntry,
    ) -> Result<(), AllocationTableUpdateError<S::Error>>
    where
        S: AsyncRead + AsyncSeek + AsyncWrite,
    {
        let physical_entry = allocation_table_entry.as_physical_entry(self.kind)?;
        let entry_offset = self.resolve_entry_offset(cluster_number);

        let mut bytes = [0; 4];

        if matches!(self.kind, AllocationTableKind::Fat12) {
            stream
                .seek(SeekFrom::Start(entry_offset.byte_offset))
                .await?;
            stream.read_exact(&mut bytes[0..2]).await?;
        };

        physical_entry.write(&mut bytes, entry_offset.is_nibble_offset);

        match self.kind {
            AllocationTableKind::Fat12 | AllocationTableKind::Fat16 => {
                stream
                    .seek(SeekFrom::Start(entry_offset.byte_offset))
                    .await?;
                stream.write_all(&bytes[0..2]).await?;
            }
            AllocationTableKind::Fat32 => {
                stream
                    .seek(SeekFrom::Start(entry_offset.byte_offset))
                    .await?;
                stream.write_all(&bytes).await?;
            }
        };

        if matches!(allocation_table_entry, AllocationTableEntry::Free) {
            self.free_cluster_scan_start_number =
                min(self.free_cluster_scan_start_number, cluster_number);
        }

        Ok(())
    }

    fn resolve_entry_offset(&self, cluster_number: u32) -> AllocationTableEntryOffset {
        let entry_index = cluster_number as u64;
        let byte_offset = match self.kind {
            AllocationTableKind::Fat12 => entry_index + (entry_index / 2),
            AllocationTableKind::Fat16 => entry_index * 2,
            AllocationTableKind::Fat32 => entry_index * 4,
        };

        AllocationTableEntryOffset {
            byte_offset,
            is_nibble_offset: matches!(self.kind, AllocationTableKind::Fat12)
                && cluster_number % 2 == 1,
        }
    }
}
