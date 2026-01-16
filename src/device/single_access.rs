use crate::File;
use crate::device::{AsyncDevice, AsyncFlushableDevice, Device, SyncDevice, SyncFlushableDevice};
use core::cell::{BorrowMutError, RefCell, RefMut};
use core::fmt::{Display, Formatter};
use core::ops::{Deref, DerefMut};
use embedded_io::{Error, ErrorKind, ErrorType, Read, Seek, SeekFrom, Write};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};

pub struct SingleAccessDevice<S>
where
    S: ErrorType,
{
    stream: RefCell<S>,
}

impl<S> SingleAccessDevice<S>
where
    S: ErrorType,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream: RefCell::new(stream),
        }
    }
}

impl<S> From<S> for SingleAccessDevice<S>
where
    S: ErrorType,
{
    fn from(value: S) -> Self {
        Self::new(value)
    }
}

impl<S> Device for SingleAccessDevice<S>
where
    S: ErrorType,
{
    type Stream = S;
    type Error = SingleAccessDeviceError<S::Error>;
}

impl<S> SyncDevice for SingleAccessDevice<S>
where
    S: ErrorType,
{
    fn with_stream<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&mut Self::Stream) -> R,
    {
        let mut stream = self.stream.try_borrow_mut()?;

        Ok(f(stream.deref_mut()))
    }
}

impl<S> SyncFlushableDevice for SingleAccessDevice<S>
where
    S: Write,
{
    fn flush(&self) -> Result<(), Self::Error> {
        let mut stream = self.stream.try_borrow_mut()?;

        stream.flush().map_err(SingleAccessDeviceError::FlushFailed)
    }
}

impl<S> AsyncDevice for SingleAccessDevice<S>
where
    S: ErrorType,
{
    #[allow(clippy::await_holding_refcell_ref)]
    async fn with_stream<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: AsyncFnOnce(&mut Self::Stream) -> R,
    {
        let mut stream = self.stream.try_borrow_mut()?;

        Ok(f(stream.deref_mut()).await)
    }
}

impl<S> AsyncFlushableDevice for SingleAccessDevice<S>
where
    S: AsyncWrite,
{
    #[allow(clippy::await_holding_refcell_ref)]
    async fn flush(&self) -> Result<(), Self::Error> {
        let mut stream = self.stream.try_borrow_mut()?;

        stream
            .flush()
            .await
            .map_err(SingleAccessDeviceError::FlushFailed)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SingleAccessDeviceError<E>
where
    E: Error,
{
    /// The stream is already in use by another process
    StreamInUse,

    /// Attempting to flush the underlying stream failed
    FlushFailed(E),
}

impl<E> core::error::Error for SingleAccessDeviceError<E> where E: Error {}

impl<E> Display for SingleAccessDeviceError<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SingleAccessDeviceError::StreamInUse => {
                write!(f, "some other process is already using the device's stream")
            }
            SingleAccessDeviceError::FlushFailed(e) => write!(
                f,
                "an error occurred while flushing the underlying stream: {}",
                e
            ),
        }
    }
}

impl<E> Error for SingleAccessDeviceError<E>
where
    E: Error,
{
    fn kind(&self) -> ErrorKind {
        match self {
            SingleAccessDeviceError::StreamInUse => ErrorKind::Other,
            SingleAccessDeviceError::FlushFailed(e) => e.kind(),
        }
    }
}

impl<E> From<BorrowMutError> for SingleAccessDeviceError<E>
where
    E: Error,
{
    fn from(value: BorrowMutError) -> Self {
        Self::StreamInUse
    }
}

#[cfg(test)]
mod tests {
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
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use crate::mock::IoError;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                SingleAccessDeviceError::StreamInUse,
                SingleAccessDeviceError::FlushFailed(IoError::default()),
            ];

            for value in values {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }

    mod kind {
        use super::*;

        #[test]
        fn stream_in_use_is_other() {
            assert_eq!(
                SingleAccessDeviceError::<IoError>::StreamInUse.kind(),
                ErrorKind::Other
            );
        }

        #[test]
        fn flush_failed_inherits_value() {
            assert_eq!(
                SingleAccessDeviceError::<IoError>::FlushFailed(IoError(ErrorKind::AddrInUse))
                    .kind(),
                ErrorKind::AddrInUse
            );
        }
    }
}
