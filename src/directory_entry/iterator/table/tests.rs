use super::*;
use crate::SingleAccessDevice;
use crate::directory_entry::FreeDirectoryEntry;
use crate::mock::{
    DataStream, ErroringDevice, ErroringStream, ErroringStreamScenarios, IoError, VoidStream,
};
use alloc::vec;
use alloc::vec::Vec;

mod advance {
    use super::*;

    #[test]
    fn next_entry_exists_returns_true() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 2);

        let result = iterator.advance();

        assert_eq!(result, true, "True should be returned");
    }

    #[test]
    fn no_next_entry_returns_false() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let result = iterator.advance();
        assert_eq!(result, false, "False should be returned");
    }

    #[test]
    fn no_current_entry_returns_false() {
        let device = SingleAccessDevice::new(VoidStream::new());
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
        let test_instance = TestInstance::new(2);
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
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

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
        let test_instance = TestInstance::new(0);
        let mut iterator = test_instance.iterator();

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

        let iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

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

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
        let device = SingleAccessDevice::new(DataStream::from_bytes([]));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
        let iterator = DirectoryTableEntryIterator::new(&ErroringDevice, 0, 1);

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

        let device = SingleAccessDevice::new(DataStream::from_bytes(data));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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

mod next {
    use super::*;

    #[test]
    fn initial_iteration_returns_first_entry() {
        let test_instance = TestInstance::new(1);
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
    fn second_iteration_returns_second_entry() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        iterator.advance();

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
        let test_instance = TestInstance::new(1);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.next();

        assert!(matches!(result, None), "None should be returned");
    }

    #[test]
    fn seek_err_propagated() {
        let device = SingleAccessDevice::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::SEEK,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

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
    fn read_err_propagated() {
        let device = SingleAccessDevice::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
        let device = SingleAccessDevice::new(DataStream::from_bytes([]));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
    fn device_err_propagated() {
        let mut iterator = DirectoryTableEntryIterator::new(&ErroringDevice, 0, 1);

        let error = iterator
            .next()
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

        let device = SingleAccessDevice::new(DataStream::from_bytes(data));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator
            .next()
            .expect("Some should be returned")
            .expect_err("Err should be returned");

        assert!(
            matches!(error, DirectoryEntryIterationError::EntryInvalid(_)),
            "EntryInvalid should be returned"
        );
    }
}

mod peek_async {
    use super::*;

    #[tokio::test]
    async fn initial_iteration_returns_first_entry() {
        let test_instance = TestInstance::new(1);
        let iterator = test_instance.iterator();

        let result = iterator
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

    #[tokio::test]
    async fn second_iteration_returns_second_entry() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator
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

    #[tokio::test]
    async fn multiple_peeks_does_not_advance_iterator() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

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
        let test_instance = TestInstance::new(0);
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

        let iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

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

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
        let device = SingleAccessDevice::new(DataStream::from_bytes([]));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
        let iterator = DirectoryTableEntryIterator::new(&ErroringDevice, 0, 1);

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

        let device = SingleAccessDevice::new(DataStream::from_bytes(data));

        let iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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

mod next_async {
    use super::*;

    #[tokio::test]
    async fn initial_iteration_returns_first_entry() {
        let test_instance = TestInstance::new(1);
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
    async fn second_iteration_returns_second_entry() {
        let test_instance = TestInstance::new(2);
        let mut iterator = test_instance.iterator();

        iterator.advance();

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
        let test_instance = TestInstance::new(1);
        let mut iterator = test_instance.iterator();

        iterator.advance();

        let result = iterator.next_async().await;

        assert!(matches!(result, None), "None should be returned");
    }

    #[tokio::test]
    async fn seek_err_propagated() {
        let device = SingleAccessDevice::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::SEEK,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 1, 1);

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
    async fn read_err_propagated() {
        let device = SingleAccessDevice::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::READ,
        ));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
        let device = SingleAccessDevice::new(DataStream::from_bytes([]));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

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
    async fn device_err_propagated() {
        let mut iterator = DirectoryTableEntryIterator::new(&ErroringDevice, 0, 1);

        let error = iterator
            .next_async()
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

        let device = SingleAccessDevice::new(DataStream::from_bytes(data));

        let mut iterator = DirectoryTableEntryIterator::new(&device, 0, 1);

        let error = iterator
            .next_async()
            .await
            .expect("Some should be returned")
            .expect_err("Err should be returned");

        assert!(
            matches!(error, DirectoryEntryIterationError::EntryInvalid(_)),
            "EntryInvalid should be returned"
        );
    }
}

type TestInstanceDevice = SingleAccessDevice<DataStream<Vec<u8>>>;

struct TestInstance {
    device: TestInstanceDevice,
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
            device: DataStream::from_bytes(data).into(),
            entry_count: entry_count as u16,
        }
    }

    fn iterator(&self) -> DirectoryTableEntryIterator<'_, TestInstanceDevice> {
        DirectoryTableEntryIterator::new(&self.device, 0, self.entry_count)
    }
}
