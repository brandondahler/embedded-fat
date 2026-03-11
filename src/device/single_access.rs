mod error;

#[cfg(test)]
mod tests;

pub use error::*;

use crate::device::Device;
use core::cell::RefCell;
use core::fmt::Display;
use core::ops::{Deref, DerefMut};
use embedded_io::ErrorType;

#[cfg(feature = "sync")]
use {
    crate::{SyncDevice, SyncFlushableDevice},
    embedded_io::{Read, Seek, Write},
};

#[cfg(feature = "async")]
use {
    crate::{AsyncDevice, AsyncFlushableDevice},
    embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite},
};

#[derive(Clone, Debug)]
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

#[cfg(feature = "sync")]
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

#[cfg(feature = "sync")]
impl<S> SyncFlushableDevice for SingleAccessDevice<S>
where
    S: Write,
{
    fn flush(&self) -> Result<(), Self::Error> {
        let mut stream = self.stream.try_borrow_mut()?;

        stream.flush().map_err(SingleAccessDeviceError::FlushFailed)
    }
}

#[cfg(feature = "async")]
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

#[cfg(feature = "async")]
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
