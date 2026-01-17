mod error;
mod file;
mod table;

pub use error::*;
pub use file::*;
pub use table::*;

use crate::AsyncDevice;
use crate::device::{Device, SyncDevice};
use crate::directory_entry::DirectoryEntry;
use embedded_io::{ErrorType, Read, Seek};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

pub type DirectoryEntryIteratorResult<R, D> = Result<
    R,
    DirectoryEntryIterationError<<D as Device>::Error, <<D as Device>::Stream as ErrorType>::Error>,
>;

#[derive(Clone, Debug)]
pub enum DirectoryEntryIterator<'a, D>
where
    D: Device,
{
    Table(DirectoryTableEntryIterator<'a, D>),
    File(DirectoryFileEntryIterator<'a, D>),
}

impl<D, S> DirectoryEntryIterator<'_, D>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
{
    pub fn peek(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.peek(),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.peek(),
        }
    }

    pub fn advance(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => Ok(table_iterator.advance()),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.advance(),
        }
    }

    pub fn next(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.next(),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.next(),
        }
    }
}

impl<D, S> DirectoryEntryIterator<'_, D>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
{
    pub async fn peek_async(&self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.peek_async().await,
            DirectoryEntryIterator::File(file_iterator) => file_iterator.peek_async().await,
        }
    }

    pub async fn advance_async(&mut self) -> DirectoryEntryIteratorResult<bool, D> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => Ok(table_iterator.advance()),
            DirectoryEntryIterator::File(file_iterator) => file_iterator.advance_async().await,
        }
    }

    pub async fn next_async(&mut self) -> Option<DirectoryEntryIteratorResult<DirectoryEntry, D>> {
        match self {
            DirectoryEntryIterator::Table(table_iterator) => table_iterator.next_async().await,
            DirectoryEntryIterator::File(file_iterator) => file_iterator.next_async().await,
        }
    }
}

impl<'a, D> From<DirectoryTableEntryIterator<'a, D>> for DirectoryEntryIterator<'a, D>
where
    D: Device,
{
    fn from(value: DirectoryTableEntryIterator<'a, D>) -> Self {
        Self::Table(value)
    }
}

impl<'a, D> From<DirectoryFileEntryIterator<'a, D>> for DirectoryEntryIterator<'a, D>
where
    D: Device,
{
    fn from(value: DirectoryFileEntryIterator<'a, D>) -> Self {
        Self::File(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocation_table::AllocationTable;
    use crate::directory_entry::{DIRECTORY_ENTRY_SIZE, FreeDirectoryEntry};
    use crate::mock::DataStream;
    use crate::utils::write_le_u32;
    use crate::{AllocationTableKind, SingleAccessDevice};
    use alloc::vec;
    use alloc::vec::Vec;

    mod peek {
        use super::*;

        #[test]
        fn delegates_to_implementation() {
            let test_instance = TestInstance::new(1);

            let result = test_instance
                .file_iterator()
                .peek()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );

            let result = test_instance
                .table_iterator()
                .peek()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );
        }
    }

    mod next {
        use super::*;

        #[test]
        fn delegates_to_implementation() {
            let test_instance = TestInstance::new(1);

            let result = test_instance
                .file_iterator()
                .next()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );

            let result = test_instance
                .table_iterator()
                .next()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );
        }
    }

    mod advance {
        use super::*;

        #[test]
        fn delegates_to_implementation() {
            let test_instance = TestInstance::new(1);

            let result = test_instance
                .file_iterator()
                .advance()
                .expect("Ok should be returned");

            assert_eq!(result, false, "False should be returned");

            let result = test_instance
                .table_iterator()
                .advance()
                .expect("Ok should be returned");

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
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );

            let result = test_instance
                .table_iterator()
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );
        }
    }

    mod advance_async {
        use super::*;

        #[tokio::test]
        async fn delegates_to_implementation() {
            let test_instance = TestInstance::new(1);

            let result = test_instance
                .file_iterator()
                .advance_async()
                .await
                .expect("Ok should be returned");

            assert_eq!(result, false, "False should be returned");

            let result = test_instance
                .table_iterator()
                .advance_async()
                .await
                .expect("Ok should be returned");

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
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );

            let result = test_instance
                .table_iterator()
                .next_async()
                .await
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
                ),
                "AllFollowing free directory entry should be returned"
            );
        }
    }

    type TestInstanceDevice = SingleAccessDevice<DataStream<Vec<u8>>>;

    struct TestInstance {
        device: TestInstanceDevice,
        allocation_table: AllocationTable,

        data_region_base_address: u32,
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
                device: SingleAccessDevice::new(DataStream::from_data(data)),
                allocation_table: AllocationTable::new(allocation_table_kind, 0),

                entry_count,
                data_region_base_address: 12,
            }
        }

        fn file_iterator(&self) -> DirectoryEntryIterator<'_, TestInstanceDevice> {
            let iterator = DirectoryFileEntryIterator::new(
                &self.device,
                &self.allocation_table,
                self.data_region_base_address,
                self.entry_count as u32 * DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            iterator.into()
        }

        fn table_iterator(&self) -> DirectoryEntryIterator<'_, TestInstanceDevice> {
            let iterator = DirectoryTableEntryIterator::new(
                &self.device,
                self.data_region_base_address,
                self.entry_count,
            );

            iterator.into()
        }
    }
}
