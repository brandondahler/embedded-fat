use super::*;
use crate::Device;
use crate::mock::{DataStream, ErroringStream, ErroringStreamScenarios, IoError, VoidStream};
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt::{Debug, Display};
use embedded_io::ErrorType;
use strum::IntoEnumIterator;

mod kind {
    use super::*;

    #[test]
    fn returns_construction_value() {
        for kind in AllocationTableKind::iter() {
            let allocation_table = AllocationTable::new(kind, 0, 10);

            assert_eq!(allocation_table.kind(), kind);
        }
    }
}

mod read_entry {
    use super::*;

    #[test]
    fn fat_12_entry_values_read_successfully() {
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 2, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 10);
        let mut stream = DataStream::from_bytes([0x12, 0x34]);

        let result = allocation_table
            .read_entry(&mut stream, 0)
            .expect_err("Read should fail");

        assert!(
            matches!(result, AllocationTableReadError::StreamEndReached),
            "Error should be StreamEndReached"
        );
    }

    #[test]
    fn stream_seek_error_propagated() {
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 10);
        let mut stream = ErroringStream::new(
            DataStream::from_bytes([0, 0, 0, 0]),
            IoError::default(),
            ErroringStreamScenarios::SEEK,
        );

        let result = allocation_table
            .read_entry(&mut stream, 0)
            .expect_err("Read should fail");

        assert!(
            matches!(result, AllocationTableReadError::StreamError(_)),
            "Error should be StreamError"
        );
    }

    #[test]
    fn stream_read_error_propagated() {
        for allocation_table_kind in [AllocationTableKind::Fat16, AllocationTableKind::Fat32] {
            let allocation_table = AllocationTable::new(allocation_table_kind, 0, 10);
            let mut stream = ErroringStream::new(
                DataStream::from_bytes([0, 0, 0, 0]),
                IoError::default(),
                ErroringStreamScenarios::READ,
            );

            let result = allocation_table
                .read_entry(&mut stream, 0)
                .expect_err("Read should fail");

            assert!(
                matches!(result, AllocationTableReadError::StreamError(_)),
                "Error should be StreamError"
            );
        }
    }
}

mod read_entry_async {
    use super::*;

    #[tokio::test]
    async fn fat_12_entry_values_read_successfully() {
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 2, 10);
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
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 10);
        let mut stream = DataStream::from_bytes([0x12, 0x34]);

        let result = allocation_table
            .read_entry_async(&mut stream, 0)
            .await
            .expect_err("Read should fail");

        assert!(
            matches!(result, AllocationTableReadError::StreamEndReached),
            "Error should be StreamEndReached"
        );
    }

    #[tokio::test]
    async fn stream_seek_error_propagated() {
        let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 10);
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
            matches!(result, AllocationTableReadError::StreamError(_)),
            "Error should be StreamError"
        );
    }

    #[tokio::test]
    async fn stream_read_error_propagated() {
        for allocation_table_kind in [AllocationTableKind::Fat16, AllocationTableKind::Fat32] {
            let allocation_table = AllocationTable::new(allocation_table_kind, 0, 10);
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
                matches!(result, AllocationTableReadError::StreamError(_)),
                "Error should be StreamError"
            );
        }
    }
}

mod find_free_entry {
    use super::*;

    #[test]
    fn correct_entry_identified() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0x00, 0x00,
            0xFF, 0xFF,
            0x00, 0x00,
        ]);

        let result = allocation_table.find_free_entry(&mut stream).unwrap();

        assert_eq!(result, Some(3));
    }

    #[test]
    fn repeated_search_finds_same_entry() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0x00, 0x00,
            0xFF, 0xFF,
            0x00, 0x00,
        ]);

        let result1 = allocation_table.find_free_entry(&mut stream).unwrap();
        let result2 = allocation_table.find_free_entry(&mut stream).unwrap();

        assert_eq!(result2, result1);
    }

    #[test]
    fn after_update_finds_next_entry() {
        #[rustfmt::skip]
        let bytes = Rc::new(RefCell::new([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0x00, 0x00,
            0xFF, 0xFF,
            0x00, 0x00,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);
        let mut stream = DataStream::from_bytes_ref_cell(bytes.clone());

        let result1 = allocation_table.find_free_entry(&mut stream).unwrap();

        {
            let mut updatable_bytes = bytes.borrow_mut();
            updatable_bytes[6] = 0xFF;
            updatable_bytes[7] = 0xFF;
        }

        let result2 = allocation_table.find_free_entry(&mut stream).unwrap();

        assert_eq!(result1, Some(3));
        assert_eq!(result2, Some(5));
    }

    #[test]
    fn first_two_cluster_entries_ignored() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0x00, 0x00,
            0x00, 0x00,
            0x00, 0x00
        ]);

        let result = allocation_table.find_free_entry(&mut stream).unwrap();

        assert_eq!(result, Some(2));
    }

    #[test]
    fn no_free_clusters_returns_none() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 4);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF
        ]);

        let result = allocation_table.find_free_entry(&mut stream).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn read_error_propagated() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 4);
        let mut stream = ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        );

        let result = allocation_table.find_free_entry(&mut stream);

        assert!(result.is_err(), "Err should be returned");
    }
}

mod find_free_entry_async {
    use super::*;

    #[tokio::test]
    async fn correct_entry_identified() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0x00, 0x00,
            0xFF, 0xFF,
            0x00, 0x00,
        ]);

        let result = allocation_table
            .find_free_entry_async(&mut stream)
            .await
            .unwrap();

        assert_eq!(result, Some(3));
    }

    #[tokio::test]
    async fn repeated_search_finds_same_entry() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0x00, 0x00,
            0xFF, 0xFF,
            0x00, 0x00,
        ]);

        let result1 = allocation_table
            .find_free_entry_async(&mut stream)
            .await
            .unwrap();
        let result2 = allocation_table
            .find_free_entry_async(&mut stream)
            .await
            .unwrap();

        assert_eq!(result2, result1);
    }

    #[tokio::test]
    async fn after_update_finds_next_entry() {
        #[rustfmt::skip]
        let bytes = Rc::new(RefCell::new([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0x00, 0x00,
            0xFF, 0xFF,
            0x00, 0x00,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);
        let mut stream = DataStream::from_bytes_ref_cell(bytes.clone());

        let result1 = allocation_table
            .find_free_entry_async(&mut stream)
            .await
            .unwrap();

        {
            let mut updatable_bytes = bytes.borrow_mut();
            updatable_bytes[6] = 0xFF;
            updatable_bytes[7] = 0xFF;
        }

        let result2 = allocation_table
            .find_free_entry_async(&mut stream)
            .await
            .unwrap();

        assert_eq!(result1, Some(3));
        assert_eq!(result2, Some(5));
    }

    #[tokio::test]
    async fn first_two_cluster_entries_ignored() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 10);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0x00, 0x00,
            0x00, 0x00,
            0x00, 0x00
        ]);

        let result = allocation_table
            .find_free_entry_async(&mut stream)
            .await
            .unwrap();

        assert_eq!(result, Some(2));
    }

    #[tokio::test]
    async fn no_free_clusters_returns_none() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 4);

        #[rustfmt::skip]
        let mut stream = DataStream::from_bytes([
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF,
            0xFF, 0xFF
        ]);

        let result = allocation_table
            .find_free_entry_async(&mut stream)
            .await
            .unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn read_error_propagated() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 4);
        let mut stream = ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        );

        let result = allocation_table.find_free_entry_async(&mut stream).await;

        assert!(result.is_err(), "Err should be returned");
    }
}

mod update_entry {
    use super::*;

    #[test]
    fn fat12_non_offset_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry(&mut stream, 2, AllocationTableEntry::Free)
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF,
            0x00, 0xF0, 0xFF,
        ]);
    }

    #[test]
    fn fat12_offset_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry(&mut stream, 3, AllocationTableEntry::Free)
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF,
            0xFF, 0x0F, 0x00,
        ]);
    }

    #[test]
    fn fat16_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry(&mut stream, 2, AllocationTableEntry::Free)
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0xFF, 0xFF,
        ]);
    }

    #[test]
    fn fat32_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry(&mut stream, 2, AllocationTableEntry::Free)
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF,
        ]);
    }

    #[test]
    fn invalid_entry_returns_err() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 10);
        let mut stream = VoidStream::new();

        let error = allocation_table
            .update_entry(
                &mut stream,
                2,
                AllocationTableEntry::NextClusterNumber(u32::MAX),
            )
            .unwrap_err();

        assert_eq!(
            error,
            AllocationTableUpdateError::EntryInvalid(
                PhysicalAllocationTableEntryError::ValueInvalid(u32::MAX)
            )
        );
    }

    #[test]
    fn seek_error_propagates_err() {
        let kinds = [
            AllocationTableKind::Fat12,
            AllocationTableKind::Fat16,
            AllocationTableKind::Fat32,
        ];

        for kind in kinds {
            let mut allocation_table = AllocationTable::new(kind, 0, 10);
            let mut stream = ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            );

            let error = allocation_table
                .update_entry(&mut stream, 2, AllocationTableEntry::Free)
                .unwrap_err();

            assert_eq!(
                error,
                AllocationTableUpdateError::StreamError(IoError::default())
            );
        }
    }

    #[test]
    fn write_error_propagates_err() {
        let kinds = [AllocationTableKind::Fat16, AllocationTableKind::Fat32];

        for kind in kinds {
            let mut allocation_table = AllocationTable::new(kind, 0, 10);
            let mut stream = ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::WRITE,
            );

            let error = allocation_table
                .update_entry(&mut stream, 2, AllocationTableEntry::Free)
                .unwrap_err();

            assert_eq!(
                error,
                AllocationTableUpdateError::StreamError(IoError::default())
            );
        }
    }

    #[test]
    fn read_error_propagates_err() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 10);
        let mut stream = ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        );

        let error = allocation_table
            .update_entry(&mut stream, 2, AllocationTableEntry::Free)
            .unwrap_err();

        assert_eq!(
            error,
            AllocationTableUpdateError::StreamError(IoError::default())
        );
    }
}

mod update_entry_async {
    use super::*;

    #[tokio::test]
    async fn fat12_non_offset_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry_async(&mut stream, 2, AllocationTableEntry::Free)
            .await
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF,
            0x00, 0xF0, 0xFF,
        ]);
    }

    #[tokio::test]
    async fn fat12_offset_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry_async(&mut stream, 3, AllocationTableEntry::Free)
            .await
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF,
            0xFF, 0x0F, 0x00,
        ]);
    }

    #[tokio::test]
    async fn fat16_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat16, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry_async(&mut stream, 2, AllocationTableEntry::Free)
            .await
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0xFF, 0xFF,
        ]);
    }

    #[tokio::test]
    async fn fat32_updates_entry_successfully() {
        #[rustfmt::skip]
        let data = Rc::new(RefCell::new([
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ]));

        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0, 3);
        let mut stream = DataStream::from_bytes_ref_cell(data.clone());

        allocation_table
            .update_entry_async(&mut stream, 2, AllocationTableEntry::Free)
            .await
            .unwrap();

        let updated_data = data.borrow();

        #[rustfmt::skip]
        assert_eq!(*updated_data, [
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF,
        ]);
    }

    #[tokio::test]
    async fn invalid_entry_returns_err() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 10);
        let mut stream = VoidStream::new();

        let error = allocation_table
            .update_entry_async(
                &mut stream,
                2,
                AllocationTableEntry::NextClusterNumber(u32::MAX),
            )
            .await
            .unwrap_err();

        assert_eq!(
            error,
            AllocationTableUpdateError::EntryInvalid(
                PhysicalAllocationTableEntryError::ValueInvalid(u32::MAX)
            )
        );
    }

    #[tokio::test]
    async fn seek_error_propagates_err() {
        let kinds = [
            AllocationTableKind::Fat12,
            AllocationTableKind::Fat16,
            AllocationTableKind::Fat32,
        ];

        for kind in kinds {
            let mut allocation_table = AllocationTable::new(kind, 0, 10);
            let mut stream = ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            );

            let error = allocation_table
                .update_entry_async(&mut stream, 2, AllocationTableEntry::Free)
                .await
                .unwrap_err();

            assert_eq!(
                error,
                AllocationTableUpdateError::StreamError(IoError::default())
            );
        }
    }

    #[tokio::test]
    async fn write_error_propagates_err() {
        let kinds = [AllocationTableKind::Fat16, AllocationTableKind::Fat32];

        for kind in kinds {
            let mut allocation_table = AllocationTable::new(kind, 0, 10);
            let mut stream = ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::WRITE,
            );

            let error = allocation_table
                .update_entry_async(&mut stream, 2, AllocationTableEntry::Free)
                .await
                .unwrap_err();

            assert_eq!(
                error,
                AllocationTableUpdateError::StreamError(IoError::default())
            );
        }
    }

    #[tokio::test]
    async fn read_error_propagates_err() {
        let mut allocation_table = AllocationTable::new(AllocationTableKind::Fat12, 0, 10);
        let mut stream = ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        );

        let error = allocation_table
            .update_entry_async(&mut stream, 2, AllocationTableEntry::Free)
            .await
            .unwrap_err();

        assert_eq!(
            error,
            AllocationTableUpdateError::StreamError(IoError::default())
        );
    }
}
