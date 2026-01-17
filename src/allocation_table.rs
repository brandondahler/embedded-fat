mod entry;
mod error;
mod kind;

pub use entry::*;
pub use error::*;
pub use kind::*;

use crate::utils::read_le_u32;
use embedded_io::{Read, Seek, SeekFrom};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

#[derive(Clone, Debug)]
pub struct AllocationTable {
    kind: AllocationTableKind,
    base_address: u32,
}

impl AllocationTable {
    pub fn new(kind: AllocationTableKind, base_address: u32) -> Self {
        Self { kind, base_address }
    }

    pub(crate) fn kind(&self) -> AllocationTableKind {
        self.kind
    }

    pub fn read_entry<S>(
        &self,
        stream: &mut S,
        cluster_number: u32,
    ) -> Result<AllocationTableEntry, AllocationTableError<S::Error>>
    where
        S: Read + Seek,
    {
        let mut entry_value_bytes = [0u8; 4];
        let entry_address = self.resolve_entry_address(cluster_number);

        stream.seek(SeekFrom::Start(entry_address.address as u64))?;

        match self.kind {
            AllocationTableKind::Fat12 | AllocationTableKind::Fat16 => {
                stream.read_exact(&mut entry_value_bytes[0..2])?;
            }
            AllocationTableKind::Fat32 => {
                stream.read_exact(&mut entry_value_bytes)?;
            }
        }

        Ok(AllocationTableEntry::from_entry_value(
            self.kind,
            self.resolve_entry_value(&entry_value_bytes, entry_address.is_nibble_offset),
        ))
    }

    pub async fn read_entry_async<S>(
        &self,
        stream: &mut S,
        cluster_number: u32,
    ) -> Result<AllocationTableEntry, AllocationTableError<S::Error>>
    where
        S: AsyncRead + AsyncSeek,
    {
        let mut entry_value_bytes = [0u8; 4];
        let entry_address = self.resolve_entry_address(cluster_number);

        stream
            .seek(SeekFrom::Start(entry_address.address as u64))
            .await?;

        match self.kind {
            AllocationTableKind::Fat12 | AllocationTableKind::Fat16 => {
                stream.read_exact(&mut entry_value_bytes[0..2]).await?;
            }
            AllocationTableKind::Fat32 => {
                stream.read_exact(&mut entry_value_bytes).await?;
            }
        }

        Ok(AllocationTableEntry::from_entry_value(
            self.kind,
            self.resolve_entry_value(&entry_value_bytes, entry_address.is_nibble_offset),
        ))
    }

    fn resolve_entry_value(&self, entry_value_bytes: &[u8; 4], is_nibble_offset: bool) -> u32 {
        let mut entry_value = read_le_u32(entry_value_bytes, 0);

        if is_nibble_offset {
            entry_value >>= 4;
        }

        entry_value & self.kind.entry_mask()
    }

    fn resolve_entry_address(&self, cluster_number: u32) -> AllocationTableEntryOffset {
        let address_offset = match self.kind {
            AllocationTableKind::Fat12 => cluster_number + (cluster_number / 2),
            AllocationTableKind::Fat16 => cluster_number * 2,
            AllocationTableKind::Fat32 => cluster_number * 4,
        };

        AllocationTableEntryOffset {
            address: self.base_address + address_offset,
            is_nibble_offset: matches!(self.kind, AllocationTableKind::Fat12)
                && cluster_number % 2 == 1,
        }
    }
}

struct AllocationTableEntryOffset {
    pub address: u32,
    pub is_nibble_offset: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Device;
    use crate::device::SyncDevice;
    use crate::mock::{DataStream, ErroringStream, ErroringStreamScenarios, IoError};
    use core::fmt::{Debug, Display};
    use embedded_io::ErrorType;
    use strum::IntoEnumIterator;

    mod kind {
        use super::*;

        #[test]
        fn returns_construction_value() {
            for kind in AllocationTableKind::iter() {
                let allocation_table = AllocationTable::new(kind, 0);

                assert_eq!(allocation_table.kind(), kind);
            }
        }
    }

    mod read_entry {
        use super::*;

        #[test]
        fn fat_12_entry_values_read_successfully() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0);
            let mut stream = DataStream::from_data([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 0)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x412),
                "Non-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 1)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x563),
                "Nibble-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 2)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0xA78),
                "Byte offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 3)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0xBC9),
                "Byte and nibble offset value should read correctly"
            );
        }

        #[test]
        fn fat_16_offset_entry_values_read_successfully() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0);
            let mut stream = DataStream::from_data([0x12, 0x34, 0x56, 0x78]);

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 0)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x3412),
                "Non-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 1)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x7856),
                "Offset value should read correctly"
            );
        }

        #[test]
        fn fat_32_offset_entry_values_read_successfully() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream =
                DataStream::from_data([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xFF]);

            // NOTE: Fat32 only uses the lower 28 of the 32 bits
            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 0)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x08563412),
                "Non-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 1)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x0FDEBC9A),
                "Offset value should read correctly"
            );
        }

        #[test]
        fn base_address_honored() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 2);
            let mut stream = DataStream::from_data([0x12, 0x34, 0x56, 0x78]);

            assert_eq!(
                allocation_table
                    .read_entry(&mut stream, 0)
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x7856),
                "Value should read correctly"
            );
        }

        #[test]
        fn stream_not_long_enough_returns_error() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream = DataStream::from_data([0x12, 0x34]);

            let result = allocation_table
                .read_entry(&mut stream, 0)
                .expect_err("Read should fail");

            assert!(
                matches!(result, AllocationTableError::StreamEndReached),
                "Error should be StreamEndReached"
            );
        }

        #[test]
        fn stream_seek_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream = ErroringStream::new(
                DataStream::from_data([0, 0, 0, 0]),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            );

            let result = allocation_table
                .read_entry(&mut stream, 0)
                .expect_err("Read should fail");

            assert!(
                matches!(result, AllocationTableError::StreamError(_)),
                "Error should be StreamError"
            );
        }

        #[test]
        fn stream_read_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream = ErroringStream::new(
                DataStream::from_data([0, 0, 0, 0]),
                IoError::default(),
                ErroringStreamScenarios::READ,
            );

            let result = allocation_table
                .read_entry(&mut stream, 0)
                .expect_err("Read should fail");

            assert!(
                matches!(result, AllocationTableError::StreamError(_)),
                "Error should be StreamError"
            );
        }
    }

    mod read_entry_async {
        use super::*;

        #[tokio::test]
        async fn fat_12_entry_values_read_successfully() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0);
            let mut stream = DataStream::from_data([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 0)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x412),
                "Non-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 1)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x563),
                "Nibble-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 2)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0xA78),
                "Byte offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 3)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0xBC9),
                "Byte and nibble offset value should read correctly"
            );
        }

        #[tokio::test]
        async fn fat_16_offset_entry_values_read_successfully() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0);
            let mut stream = DataStream::from_data([0x12, 0x34, 0x56, 0x78]);

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 0)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x3412),
                "Non-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 1)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x7856),
                "Offset value should read correctly"
            );
        }

        #[tokio::test]
        async fn fat_32_offset_entry_values_read_successfully() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream =
                DataStream::from_data([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xFF]);

            // NOTE: Fat32 only uses the lower 28 of the 32 bits
            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 0)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x08563412),
                "Non-offset value should read correctly"
            );

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 1)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x0FDEBC9A),
                "Offset value should read correctly"
            );
        }

        #[tokio::test]
        async fn base_address_honored() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 2);
            let mut stream = DataStream::from_data([0x12, 0x34, 0x56, 0x78]);

            assert_eq!(
                allocation_table
                    .read_entry_async(&mut stream, 0)
                    .await
                    .expect("Read should succeed"),
                AllocationTableEntry::NextClusterNumber(0x7856),
                "Value should read correctly"
            );
        }

        #[tokio::test]
        async fn stream_not_long_enough_returns_error() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream = DataStream::from_data([0x12, 0x34]);

            let result = allocation_table
                .read_entry_async(&mut stream, 0)
                .await
                .expect_err("Read should fail");

            assert!(
                matches!(result, AllocationTableError::StreamEndReached),
                "Error should be StreamEndReached"
            );
        }

        #[tokio::test]
        async fn stream_seek_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream = ErroringStream::new(
                DataStream::from_data([0, 0, 0, 0]),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            );

            let result = allocation_table
                .read_entry_async(&mut stream, 0)
                .await
                .expect_err("Read should fail");

            assert!(
                matches!(result, AllocationTableError::StreamError(_)),
                "Error should be StreamError"
            );
        }

        #[tokio::test]
        async fn stream_read_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);
            let mut stream = ErroringStream::new(
                DataStream::from_data([0, 0, 0, 0]),
                IoError::default(),
                ErroringStreamScenarios::READ,
            );

            let result = allocation_table
                .read_entry_async(&mut stream, 0)
                .await
                .expect_err("Read should fail");

            assert!(
                matches!(result, AllocationTableError::StreamError(_)),
                "Error should be StreamError"
            );
        }
    }
}
