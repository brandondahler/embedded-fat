use super::*;
use crate::AllocationTableKind;
use crate::allocation_table::AllocationTable;
use crate::device::Device;
use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, FreeDirectoryEntry};
use crate::mock::DataStream;
use crate::utils::write_le_u32;
use alloc::vec;
use alloc::vec::Vec;
use core::assert_matches;
use core::cell::RefCell;

mod peek {
    use super::*;

    #[test]
    fn delegates_to_implementation() {
        let test_instance = TestInstance::new(1);

        let result = test_instance.file_iterator().peek().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );

        let result = test_instance.table_iterator().peek().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }
}

mod next {
    use super::*;

    #[test]
    fn delegates_to_implementation() {
        let test_instance = TestInstance::new(1);

        let result = test_instance.file_iterator().next().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );

        let result = test_instance.table_iterator().next().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }
}

mod advance {
    use super::*;

    #[test]
    fn delegates_to_implementation() {
        let test_instance = TestInstance::new(1);

        let result = test_instance.file_iterator().advance().unwrap();

        assert_eq!(result, false, "False should be returned");

        let result = test_instance.table_iterator().advance().unwrap();

        assert_eq!(result, false, "False should be returned");
    }
}

mod peek_async {
    use super::*;

    #[tokio::test]
    async fn delegates_to_implementation() {
        let test_instance = TestInstance::new(1);

        let result = test_instance
            .file_iterator()
            .peek_async()
            .await
            .unwrap()
            .unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );

        let result = test_instance
            .table_iterator()
            .peek_async()
            .await
            .unwrap()
            .unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }
}

mod advance_async {
    use super::*;

    #[tokio::test]
    async fn delegates_to_implementation() {
        let test_instance = TestInstance::new(1);

        let result = test_instance.file_iterator().advance_async().await.unwrap();

        assert_eq!(result, false, "False should be returned");

        let result = test_instance
            .table_iterator()
            .advance_async()
            .await
            .unwrap();

        assert_eq!(result, false, "False should be returned");
    }
}

mod next_async {
    use super::*;

    #[tokio::test]
    async fn delegates_to_implementation() {
        let test_instance = TestInstance::new(1);

        let result = test_instance
            .file_iterator()
            .next_async()
            .await
            .unwrap()
            .unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );

        let result = test_instance
            .table_iterator()
            .next_async()
            .await
            .unwrap()
            .unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }
}

type TestInstanceStream = DataStream<Vec<u8>>;

struct TestInstance {
    device: Device<TestInstanceStream, 0>,
    allocation_table: AllocationTable,

    data_region_base_address: u64,
    entry_count: u16,
}

impl TestInstance {
    fn new(entry_count: u16) -> Self {
        let allocation_table_kind = AllocationTableKind::Fat32;
        let mut data = vec![0; 12 + (entry_count as usize * DIRECTORY_ENTRY_SIZE)];
        write_le_u32(&mut data, 8, allocation_table_kind.end_of_chain_value());

        for entry_index in 0..(entry_count - 1) {
            data[12 + (entry_index as usize * DIRECTORY_ENTRY_SIZE)] = 0xE5;
        }

        Self {
            device: Device::new(DataStream::from_bytes(data)),
            allocation_table: AllocationTable::new(allocation_table_kind, 0, 10),

            entry_count,
            data_region_base_address: 12,
        }
    }

    fn file_iterator(&self) -> DirectoryEntryIterator<'_, TestInstanceStream, 0> {
        let iterator = DirectoryFileEntryIterator::new(
            &self.device,
            &self.allocation_table,
            self.data_region_base_address,
            self.entry_count as u32 * DIRECTORY_ENTRY_SIZE as u32,
            2,
        );

        iterator.into()
    }

    fn table_iterator(&self) -> DirectoryEntryIterator<'_, TestInstanceStream, 0> {
        let iterator = DirectoryTableEntryIterator::new(
            &self.device,
            self.data_region_base_address,
            self.entry_count,
        );

        iterator.into()
    }
}
