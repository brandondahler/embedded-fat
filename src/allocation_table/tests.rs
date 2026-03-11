use super::*;
use crate::Device;
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
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xFF]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34]);

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
            DataStream::from_bytes([0, 0, 0, 0]),
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
        for allocation_table_kind in [AllocationTableKind::Fat16, AllocationTableKind::Fat32] {
            let allocation_table = AllocationTable::new(allocation_table_kind, 0);
            let mut stream = ErroringStream::new(
                DataStream::from_bytes([0, 0, 0, 0]),
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
}

mod read_entry_async {
    use super::*;

    #[tokio::test]
    async fn fat_12_entry_values_read_successfully() {
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0);
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xFF]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34, 0x56, 0x78]);

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
        let mut stream = DataStream::from_bytes([0x12, 0x34]);

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
            DataStream::from_bytes([0, 0, 0, 0]),
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
        for allocation_table_kind in [AllocationTableKind::Fat16, AllocationTableKind::Fat32] {
            let allocation_table = AllocationTable::new(allocation_table_kind, 0);
            let mut stream = ErroringStream::new(
                DataStream::from_bytes([0, 0, 0, 0]),
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
