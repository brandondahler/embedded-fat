use crate::allocation_table::{AllocationTable, AllocationTableEntry};
use crate::device::{AsyncDevice, Device, SyncDevice};
use crate::directory_entry::{
    DIRECTORY_ENTRY_SIZE, DirectoryEntry, DirectoryEntryIterationError,
    DirectoryEntryIteratorResult,
};
use core::ops::DerefMut;
use embedded_io::{ErrorType, Read, Seek, SeekFrom};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

#[derive(Clone, Debug)]
pub struct DirectoryFileEntryIterator<'a, D>
where
    D: Device,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory_entry::FreeDirectoryEntry;
    use crate::mock::{
        DataStream, ErroringDevice, ErroringStream, ErroringStreamScenarios, IoError, VoidStream,
    };
    use crate::utils::write_le_u32;
    use crate::{AllocationTableKind, SingleAccessDevice};
    use alloc::vec;
    use alloc::vec::Vec;

    mod peek {
        use super::*;

        #[test]
        fn initial_iteration_returns_first_entry() {
            let test_instance = TestInstance::new(1, 1);
            let iterator = test_instance.iterator();

            let result = iterator
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

        #[test]
        fn second_iteration_returns_second_entry() {
            let test_instance = TestInstance::new(1, 2);
            let mut iterator = test_instance.iterator();

            iterator.advance();

            let result = iterator
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

        #[test]
        fn multiple_peeks_does_not_advance_iterator() {
            let test_instance = TestInstance::new(1, 2);
            let iterator = test_instance.iterator();

            let result = iterator
                .peek()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
                ),
                "CurrentOnly free directory entry should be returned"
            );

            let result = iterator
                .peek()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
                ),
                "CurrentOnly free directory entry should be returned"
            );
        }

        #[test]
        fn after_last_entry_returns_none() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            iterator.advance();

            let result = iterator.peek();

            assert!(result.is_none(), "None should be returned");
        }

        #[test]
        fn seek_err_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[test]
        fn read_err_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::READ,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[test]
        fn stream_end_reached_error_propagated() {
            let device = SingleAccessDevice::new(DataStream::from_data([]));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamEndReached),
                "StreamEndReached should be returned"
            );
        }

        #[test]
        fn device_err_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &ErroringDevice,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::DeviceError(IoError(_))),
                "DeviceError should be returned"
            );
        }

        #[test]
        fn invalid_directory_entry_error_propagated() {
            let mut data = [0; DIRECTORY_ENTRY_SIZE];
            data[0] = 0x20;

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::EntryInvalid(_)),
                "EntryInvalid should be returned"
            );
        }
    }

    mod advance {
        use super::*;

        #[test]
        fn same_cluster_successful() {
            let test_instance = TestInstance::new(1, 2);
            let mut iterator = test_instance.iterator();

            let result = iterator.advance().expect("Ok should be returned");

            assert_eq!(result, true, "True should be returned");
        }

        #[test]
        fn different_cluster_transition_successful() {
            let test_instance = TestInstance::new(2, 1);
            let mut iterator = test_instance.iterator();

            let result = iterator.advance().expect("Ok should be returned");

            assert_eq!(result, true, "True should be returned");
        }

        #[test]
        fn no_next_cluster_handled_correct() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            let result = iterator.advance().expect("Ok should be returned");
            assert_eq!(result, false, "False should be returned");
        }

        #[test]
        fn allocation_table_entry_free_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 0);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator.advance().expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[test]
        fn allocation_table_entry_reserved_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 1);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator.advance().expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[test]
        fn allocation_table_entry_bad_sector_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, AllocationTableKind::Fat32.bad_sector_value());

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator.advance().expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[test]
        fn stream_seek_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator.advance().expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[test]
        fn stream_read_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::READ,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator.advance().expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[test]
        fn stream_end_reached_error_propagated() {
            let device = SingleAccessDevice::new(DataStream::from_data([]));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator.advance().expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamEndReached),
                "StreamEndReached should be returned"
            );
        }

        #[test]
        fn device_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &ErroringDevice,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator.advance().expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::DeviceError(_)),
                "DeviceError should be returned"
            );
        }
    }

    mod next {
        use super::*;

        #[test]
        fn initial_returns_first_entry() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            let result = iterator
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

        #[test]
        fn second_returns_second_entry() {
            let test_instance = TestInstance::new(1, 2);
            let mut iterator = test_instance.iterator();

            let result = iterator
                .next()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
                ),
                "CurrentOnly free directory entry should be returned"
            );

            let result = iterator
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

        #[test]
        fn after_end_returns_none() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            iterator.advance();

            let result = iterator.next();

            assert!(matches!(result, None), "None should be returned");
        }

        #[test]
        fn allocation_table_entry_free_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 0);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[test]
        fn allocation_table_entry_reserved_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 1);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[test]
        fn allocation_table_entry_bad_sector_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, AllocationTableKind::Fat32.bad_sector_value());

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[test]
        fn stream_seek_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[test]
        fn stream_read_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::READ,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[test]
        fn stream_end_reached_error_propagated() {
            let device = SingleAccessDevice::new(DataStream::from_data([]));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamEndReached),
                "StreamEndReached should be returned"
            );
        }

        #[test]
        fn device_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &ErroringDevice,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next()
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::DeviceError(_)),
                "DeviceError should be returned"
            );
        }
    }

    mod peek_async {
        use super::*;

        #[tokio::test]
        async fn initial_iteration_returns_first_entry() {
            let test_instance = TestInstance::new(1, 1);
            let iterator = test_instance.iterator();

            let result = iterator
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

        #[tokio::test]
        async fn second_iteration_returns_second_entry() {
            let test_instance = TestInstance::new(1, 2);
            let mut iterator = test_instance.iterator();

            iterator.advance();

            let result = iterator
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

        #[tokio::test]
        async fn multiple_peeks_does_not_advance_iterator() {
            let test_instance = TestInstance::new(1, 2);
            let iterator = test_instance.iterator();

            let result = iterator
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
                ),
                "CurrentOnly free directory entry should be returned"
            );

            let result = iterator
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
                ),
                "CurrentOnly free directory entry should be returned"
            );
        }

        #[tokio::test]
        async fn after_last_entry_returns_none() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            iterator.advance();

            let result = iterator.peek_async().await;

            assert!(result.is_none(), "None should be returned");
        }

        #[tokio::test]
        async fn seek_err_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[tokio::test]
        async fn read_err_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::READ,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[tokio::test]
        async fn stream_end_reached_error_propagated() {
            let device = SingleAccessDevice::new(DataStream::from_data([]));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamEndReached),
                "StreamEndReached should be returned"
            );
        }

        #[tokio::test]
        async fn device_err_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &ErroringDevice,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::DeviceError(IoError(_))),
                "DeviceError should be returned"
            );
        }

        #[tokio::test]
        async fn invalid_directory_entry_error_propagated() {
            let mut data = [0; DIRECTORY_ENTRY_SIZE];
            data[0] = 0x20;

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .peek_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::EntryInvalid(_)),
                "EntryInvalid should be returned"
            );
        }
    }

    mod advance_async {
        use super::*;

        #[tokio::test]
        async fn same_cluster_successful() {
            let test_instance = TestInstance::new(1, 2);
            let mut iterator = test_instance.iterator();

            let result = iterator
                .advance_async()
                .await
                .expect("Ok should be returned");

            assert_eq!(result, true, "True should be returned");
        }

        #[tokio::test]
        async fn different_cluster_transition_successful() {
            let test_instance = TestInstance::new(2, 1);
            let mut iterator = test_instance.iterator();

            let result = iterator
                .advance_async()
                .await
                .expect("Ok should be returned");

            assert_eq!(result, true, "True should be returned");
        }

        #[tokio::test]
        async fn no_next_cluster_handled_correct() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            let result = iterator
                .advance_async()
                .await
                .expect("Ok should be returned");
            assert_eq!(result, false, "False should be returned");
        }

        #[tokio::test]
        async fn allocation_table_entry_free_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 0);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .advance_async()
                .await
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[tokio::test]
        async fn allocation_table_entry_reserved_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 1);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .advance_async()
                .await
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[tokio::test]
        async fn allocation_table_entry_bad_sector_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, AllocationTableKind::Fat32.bad_sector_value());

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .advance_async()
                .await
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[tokio::test]
        async fn stream_seek_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .advance_async()
                .await
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[tokio::test]
        async fn stream_read_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::READ,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .advance_async()
                .await
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[tokio::test]
        async fn stream_end_reached_error_propagated() {
            let device = SingleAccessDevice::new(DataStream::from_data([]));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .advance_async()
                .await
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamEndReached),
                "StreamEndReached should be returned"
            );
        }

        #[tokio::test]
        async fn device_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &ErroringDevice,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .advance_async()
                .await
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::DeviceError(_)),
                "DeviceError should be returned"
            );
        }
    }

    mod next_async {
        use super::*;

        #[tokio::test]
        async fn initial_returns_first_entry() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            let result = iterator
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

        #[tokio::test]
        async fn second_returns_second_entry() {
            let test_instance = TestInstance::new(1, 2);
            let mut iterator = test_instance.iterator();

            let result = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert!(
                matches!(
                    result,
                    DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
                ),
                "CurrentOnly free directory entry should be returned"
            );

            let result = iterator
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

        #[tokio::test]
        async fn after_end_returns_none() {
            let test_instance = TestInstance::new(1, 1);
            let mut iterator = test_instance.iterator();

            iterator.advance();

            let result = iterator.next_async().await;

            assert!(matches!(result, None), "None should be returned");
        }

        #[tokio::test]
        async fn allocation_table_entry_free_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 0);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[tokio::test]
        async fn allocation_table_entry_reserved_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, 1);

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[tokio::test]
        async fn allocation_table_entry_bad_sector_returns_error() {
            let mut data = [0; 12 + DIRECTORY_ENTRY_SIZE];
            write_le_u32(&mut data, 8, AllocationTableKind::Fat32.bad_sector_value());

            let device = SingleAccessDevice::new(DataStream::from_data(data));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                12,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(
                    error,
                    DirectoryEntryIterationError::AllocationTableEntryTypeUnexpected
                ),
                "AllocationTableEntryTypeUnexpected should be returned"
            );
        }

        #[tokio::test]
        async fn stream_seek_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::SEEK,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[tokio::test]
        async fn stream_read_error_propagated() {
            let device = SingleAccessDevice::new(ErroringStream::new(
                VoidStream::new(),
                IoError::default(),
                ErroringStreamScenarios::READ,
            ));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamError(IoError(_))),
                "StreamError should be returned"
            );
        }

        #[tokio::test]
        async fn stream_end_reached_error_propagated() {
            let device = SingleAccessDevice::new(DataStream::from_data([]));
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &device,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::StreamEndReached),
                "StreamEndReached should be returned"
            );
        }

        #[tokio::test]
        async fn device_error_propagated() {
            let allocation_table = AllocationTable::new(AllocationTableKind::Fat32, 0);

            let mut iterator = DirectoryFileEntryIterator::new(
                &ErroringDevice,
                &allocation_table,
                0,
                DIRECTORY_ENTRY_SIZE as u32,
                2,
            );

            let error = iterator
                .next_async()
                .await
                .expect("Some should be returned")
                .expect_err("Err should be returned");

            assert!(
                matches!(error, DirectoryEntryIterationError::DeviceError(_)),
                "DeviceError should be returned"
            );
        }
    }

    type TestInstanceDevice = SingleAccessDevice<DataStream<Vec<u8>>>;

    struct TestInstance {
        device: TestInstanceDevice,
        allocation_table: AllocationTable,

        data_region_base_address: u32,
        bytes_per_cluster: u32,
    }

    impl TestInstance {
        /// Creates an instance where it has the configured number of clusters and entries per
        /// cluster which form a valid directory file.  All but the last entry will be
        /// `FreeDirectoryEntry::CurrentOnly`, the last will be `FreeDirectoryEntry::AllFollowing`.
        fn new(cluster_count: usize, entries_per_cluster: usize) -> Self {
            let data_region_base_address = (cluster_count + 2) * 4;

            let mut data = vec![
                0;
                ((cluster_count + 2) * 4)
                    + (entries_per_cluster * cluster_count * DIRECTORY_ENTRY_SIZE)
            ];
            for cluster_index in 2..(cluster_count + 1) {
                write_le_u32(&mut data, cluster_index * 4, cluster_index as u32 + 1);
            }
            write_le_u32(
                &mut data,
                (cluster_count + 1) * 4,
                AllocationTableKind::Fat32.end_of_chain_value(),
            );

            for cluster_index in 0..cluster_count {
                for entry_index in 0..entries_per_cluster {
                    let is_last_entry = cluster_index == cluster_count - 1
                        && entry_index == entries_per_cluster - 1;

                    if !is_last_entry {
                        let entry_address = data_region_base_address
                            + (cluster_index * entries_per_cluster * DIRECTORY_ENTRY_SIZE)
                            + (entry_index * DIRECTORY_ENTRY_SIZE);

                        data[entry_address] = 0xE5;
                    }
                }
            }

            Self {
                device: DataStream::from_data(data).into(),
                allocation_table: AllocationTable::new(AllocationTableKind::Fat32, 0),

                data_region_base_address: data_region_base_address as u32,
                bytes_per_cluster: (entries_per_cluster * DIRECTORY_ENTRY_SIZE) as u32,
            }
        }

        fn iterator(&self) -> DirectoryFileEntryIterator<'_, TestInstanceDevice> {
            DirectoryFileEntryIterator::new(
                &self.device,
                &self.allocation_table,
                self.data_region_base_address,
                self.bytes_per_cluster,
                2,
            )
        }
    }
}
