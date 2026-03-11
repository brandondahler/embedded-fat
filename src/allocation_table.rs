mod entry;
mod entry_offset;
mod error;
mod kind;
mod physical_entry;

#[cfg(test)]
mod tests;

pub use entry::*;
pub use entry_offset::*;
pub use error::*;
pub use kind::*;
pub use physical_entry::*;

use crate::utils::read_le_u32;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

#[derive(Clone, Debug)]
pub struct AllocationTable {
    kind: AllocationTableKind,
    base_address: u64,
}

impl AllocationTable {
    pub fn new(kind: AllocationTableKind, base_address: u64) -> Self {
        Self { kind, base_address }
    }

    pub(crate) fn kind(&self) -> AllocationTableKind {
        self.kind
    }

    #[cfg(feature = "sync")]
    pub fn read_entry<S>(
        &self,
        stream: &mut S,
        cluster_number: u32,
    ) -> Result<AllocationTableEntry, AllocationTableError<S::Error>>
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
    ) -> Result<AllocationTableEntry, AllocationTableError<S::Error>>
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
