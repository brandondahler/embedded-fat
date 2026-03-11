use super::*;
use crate::mock::{ErroringStream, ErroringStreamScenarios, IoError, VoidStream};
use core::fmt::Debug;
use embedded_io::ErrorType;

mod sync_with_stream {
    use super::*;

    #[test]
    fn basic_usage_works() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let expected_result = 5;

        let result = SyncDevice::with_stream(&device, |_| expected_result)
            .expect("with_stream should be successful");

        assert_eq!(
            result, expected_result,
            "Result should match expected value"
        );
    }

    #[test]
    fn nested_usage_returns_err() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let expected_result = 5;

        let result = SyncDevice::with_stream(&device, |_| {
            SyncDevice::with_stream(&device, |_| unreachable!())
                .expect_err("Inner usage should fail")
        })
        .expect("Outer usage should succeed");

        assert!(
            matches!(result, SingleAccessDeviceError::StreamInUse),
            "Result should be StreamInUsage"
        );
    }
}

mod sync_flush {
    use super::*;

    #[test]
    fn basic_usage_works() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let expected_result = 5;

        let result = SyncFlushableDevice::flush(&device);

        assert!(result.is_ok(), "Flush should succeed");
    }

    #[test]
    fn nested_usage_returns_err() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let expected_result = 5;

        let result = SyncDevice::with_stream(&device, |_| {
            SyncFlushableDevice::flush(&device).expect_err("Inner usage should fail")
        })
        .expect("Outer usage should succeed");

        assert!(
            matches!(result, SingleAccessDeviceError::StreamInUse),
            "Result should be StreamInUsage"
        );
    }

    #[test]
    fn stream_flush_failure_propagated() {
        let device = SingleAccessDevice::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::FLUSH,
        ));

        let result = SyncFlushableDevice::flush(&device).expect_err("Flush should fail");

        assert!(
            matches!(result, SingleAccessDeviceError::FlushFailed(IoError(_))),
            "Err should be FlushFailed"
        );
    }
}

mod async_with_stream {
    use super::*;

    #[tokio::test]
    async fn basic_usage_works() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let expected_result = 5;

        let result = AsyncDevice::with_stream(&device, async |_| expected_result)
            .await
            .expect("with_stream should be successful");

        assert_eq!(
            result, expected_result,
            "Result should match expected value"
        );
    }

    #[tokio::test]
    async fn nested_usage_returns_err() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let expected_result = 5;

        let result = AsyncDevice::with_stream(&device, async |_| {
            AsyncDevice::with_stream(&device, async |_| unreachable!())
                .await
                .expect_err("Inner usage should fail")
        })
        .await
        .expect("Outer usage should succeed");

        assert!(
            matches!(result, SingleAccessDeviceError::StreamInUse),
            "Result should be StreamInUsage"
        );
    }
}

mod async_flush {
    use super::*;

    #[tokio::test]
    async fn basic_usage_works() {
        let device = SingleAccessDevice::new(VoidStream::new());

        let expected_result = 5;
        {
            let result = AsyncFlushableDevice::flush(&device).await;

            assert!(result.is_ok(), "Flush should succeed");
        }
    }

    #[tokio::test]
    async fn nested_usage_returns_err() {
        let device = SingleAccessDevice::new(VoidStream::new());
        let expected_result = 5;

        let result = AsyncDevice::with_stream(&device, async |_| {
            AsyncFlushableDevice::flush(&device)
                .await
                .expect_err("Inner usage should fail")
        })
        .await
        .expect("Outer usage should succeed");

        assert!(
            matches!(result, SingleAccessDeviceError::StreamInUse),
            "Result should be StreamInUsage"
        );
    }

    #[tokio::test]
    async fn stream_flush_failure_propagated() {
        let device = SingleAccessDevice::new(ErroringStream::new(
            VoidStream::new(),
            IoError::default(),
            ErroringStreamScenarios::FLUSH,
        ));

        let result = AsyncFlushableDevice::flush(&device)
            .await
            .expect_err("Flush should fail");

        assert!(
            matches!(result, SingleAccessDeviceError::FlushFailed(IoError(_))),
            "Err should be FlushFailed"
        );
    }
}
