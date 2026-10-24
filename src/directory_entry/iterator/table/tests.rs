use super::*;
use crate::directory_entry::FreeDirectoryEntry;
use crate::mock::{DataStream, ErroringStream, ErroringStreamScenarios, IoError, VoidStream};
use alloc::vec;
use alloc::vec::Vec;
use core::assert_matches;

mod advance {
    use super::*;

    #[test]
    fn next_entry_exists_returns_true() {
        let device = Device::<_, 0>::new(VoidStream::new());
        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 2);

        let result = iterator.advance();

        assert_eq!(result, true, "True should be returned");
    }

    #[test]
    fn no_next_entry_returns_false() {
        let device = Device::<_, 0>::new(VoidStream::new());
        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let result = iterator.advance();
        assert_eq!(result, false, "False should be returned");
    }

    #[test]
    fn no_current_entry_returns_false() {
        let device = Device::<_, 0>::new(VoidStream::new());
        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 0);

        let result = iterator.advance();
        assert_eq!(result, false, "False should be returned");
    }
}

mod peek {
    use super::*;

    #[test]
    fn initial_iteration_returns_first_entry() {
        let test_instance = TestInstance::new(1);
        let iterator = test_instance.iterator();

        let result = iterator.peek().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[test]
    fn second_iteration_returns_second_entry() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.peek().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[test]
    fn multiple_peeks_does_not_advance_iterator() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        let result = iterator.peek().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
        );

        let result = iterator.peek().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
        );
    }

    #[test]
    fn after_last_entry_returns_none() {
        let test_instance = TestInstance::new(0);
        let mut iterator = test_instance.iterator();

        let result = iterator.peek().unwrap();

        assert!(result.is_none(), "None should be returned");
    }

    #[test]
    fn seek_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::SEEK,
        ));

        let iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

        let error = iterator.peek().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[test]
    fn read_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        ));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[test]
    fn stream_end_reached_error_propagated() {
        let device = Device::<_, 0>::new(DataStream::from_bytes([]));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamEndReached);
    }

    #[test]
    fn device_busy_error_propagated() {
        let device = Device::<_, 0>::new(VoidStream::new());
        let _device_stream = device.acquire_stream().unwrap();

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::DeviceBusy);
    }

    #[test]
    fn invalid_directory_entry_error_propagated() {
        let mut data = [0; DIRECTORY_ENTRY_SIZE];
        data[0] = 0x20;

        let device = Device::<_, 0>::new(DataStream::from_bytes(data));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::EntryInvalid(_));
    }
}

mod next {
    use super::*;

    #[test]
    fn initial_iteration_returns_first_entry() {
        let test_instance = TestInstance::new(1);
        let mut iterator = test_instance.iterator();

        let result = iterator.next().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[test]
    fn second_iteration_returns_second_entry() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.next().unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[test]
    fn after_end_returns_none() {
        let test_instance = TestInstance::new(1);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.next().unwrap();

        assert!(matches!(result, None), "None should be returned");
    }

    #[test]
    fn seek_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::SEEK,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

        let error = iterator.next().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[test]
    fn read_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[test]
    fn stream_end_reached_error_propagated() {
        let device = Device::<_, 0>::new(DataStream::from_bytes([]));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamEndReached);
    }

    #[test]
    fn device_busy_error_propagated() {
        let device = Device::<_, 0>::new(VoidStream::new());
        let _device_stream = device.acquire_stream().unwrap();

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::DeviceBusy);
    }

    #[test]
    fn invalid_directory_entry_error_propagated() {
        let mut data = [0; DIRECTORY_ENTRY_SIZE];
        data[0] = 0x20;

        let device = Device::<_, 0>::new(DataStream::from_bytes(data));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next().unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::EntryInvalid(_));
    }
}

mod peek_async {
    use super::*;

    #[tokio::test]
    async fn initial_iteration_returns_first_entry() {
        let test_instance = TestInstance::new(1);
        let iterator = test_instance.iterator();

        let result = iterator.peek_async().await.unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[tokio::test]
    async fn second_iteration_returns_second_entry() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.peek_async().await.unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[tokio::test]
    async fn multiple_peeks_does_not_advance_iterator() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        let result = iterator.peek_async().await.unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
        );

        let result = iterator.peek_async().await.unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)
        );
    }

    #[tokio::test]
    async fn after_last_entry_returns_none() {
        let test_instance = TestInstance::new(0);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.peek_async().await.unwrap();

        assert!(result.is_none(), "None should be returned");
    }

    #[tokio::test]
    async fn seek_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::SEEK,
        ));

        let iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

        let error = iterator.peek_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[tokio::test]
    async fn read_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        ));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[tokio::test]
    async fn stream_end_reached_error_propagated() {
        let device = Device::<_, 0>::new(DataStream::from_bytes([]));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamEndReached);
    }

    #[tokio::test]
    async fn device_busy_error_propagated() {
        let device = Device::<_, 0>::new(VoidStream::new());
        let _device_stream = device.acquire_stream().unwrap();

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::DeviceBusy);
    }

    #[tokio::test]
    async fn invalid_directory_entry_error_propagated() {
        let mut data = [0; DIRECTORY_ENTRY_SIZE];
        data[0] = 0x20;

        let device = Device::<_, 0>::new(DataStream::from_bytes(data));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.peek_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::EntryInvalid(_));
    }
}

mod next_async {
    use super::*;

    #[tokio::test]
    async fn initial_iteration_returns_first_entry() {
        let test_instance = TestInstance::new(1);
        let mut iterator = test_instance.iterator();

        let result = iterator.next_async().await.unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[tokio::test]
    async fn second_iteration_returns_second_entry() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.next_async().await.unwrap().unwrap();

        assert_matches!(
            result,
            DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
        );
    }

    #[tokio::test]
    async fn after_end_returns_none() {
        let test_instance = TestInstance::new(1);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.next_async().await.unwrap();

        assert!(matches!(result, None), "None should be returned");
    }

    #[tokio::test]
    async fn seek_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::SEEK,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

        let error = iterator.next_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[tokio::test]
    async fn read_err_propagated() {
        let device = Device::<_, 0>::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamError(IoError(_)));
    }

    #[tokio::test]
    async fn stream_end_reached_error_propagated() {
        let device = Device::<_, 0>::new(DataStream::from_bytes([]));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::StreamEndReached);
    }

    #[tokio::test]
    async fn device_busy_error_propagated() {
        let device = Device::<_, 0>::new(VoidStream::new());
        let _device_stream = device.acquire_stream().unwrap();

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::DeviceBusy);
    }

    #[tokio::test]
    async fn invalid_directory_entry_error_propagated() {
        let mut data = [0; DIRECTORY_ENTRY_SIZE];
        data[0] = 0x20;

        let device = Device::<_, 0>::new(DataStream::from_bytes(data));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator.next_async().await.unwrap_err();

        assert_matches!(error, DirectoryEntryIterationError::EntryInvalid(_));
    }
}

type TestInstanceStream = DataStream<Vec<u8>>;

struct TestInstance {
    device: Device<TestInstanceStream, 0>,
    entry_count: u16,
}

impl TestInstance {
    fn new(entry_count: usize) -> Self {
        let mut data = vec![0; entry_count * DIRECTORY_ENTRY_SIZE];

        if entry_count > 0 {
            for entry_index in 0..(entry_count - 1) {
                let data_offset = entry_index * DIRECTORY_ENTRY_SIZE;

                data[data_offset] = 0xE5;
            }
        }

        Self {
            device: Device::new(DataStream::from_bytes(data)),
            entry_count: entry_count as u16,
        }
    }

    fn iterator(&self) -> DirectoryTableEntryIterator<'_, TestInstanceStream, 0> {
        DirectoryTableEntryIterator::new(&self.device, 0, self.entry_count)
    }
}
