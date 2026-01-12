use crate::device::SyncDevice;
use crate::mock::{IoError, VoidStream};
use crate::{AsyncDevice, AsyncFlushableDevice, Device, SyncFlushableDevice};

pub struct ErroringDevice;

impl Device for ErroringDevice {
    type Stream = VoidStream;
    type Error = IoError;
}

impl SyncDevice for ErroringDevice {
    fn with_stream<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&mut Self::Stream) -> R,
    {
        Err(IoError::default())
    }
}

impl SyncFlushableDevice for ErroringDevice {
    fn flush(&self) -> Result<(), Self::Error> {
        Err(IoError::default())
    }
}

impl AsyncDevice for ErroringDevice {
    async fn with_stream<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: AsyncFnOnce(&mut Self::Stream) -> R,
    {
        Err(IoError::default())
    }
}

impl AsyncFlushableDevice for ErroringDevice {
    async fn flush(&self) -> Result<(), Self::Error> {
        Err(IoError::default())
    }
}
