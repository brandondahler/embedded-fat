use crate::File;
use crate::device::{AsyncDevice, AsyncFlushableDevice, Device, SyncDevice, SyncFlushableDevice};
use core::cell::{BorrowMutError, RefCell, RefMut};
use core::fmt::{Display, Formatter};
use core::ops::{Deref, DerefMut};
use embedded_io::{Error, ErrorKind, ErrorType, Read, Seek, SeekFrom, Write};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};

pub struct SingleAccessDevice<S> {
    stream: RefCell<S>,
}

impl<S> SingleAccessDevice<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream: RefCell::new(stream),
        }
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
