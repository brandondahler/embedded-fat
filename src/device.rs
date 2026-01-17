mod single_access;

use core::error::Error;
pub use single_access::*;

use core::fmt::Debug;
use core::ops::DerefMut;
use embedded_io::{ErrorType, Read, Seek, Write};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

pub trait Device {
    type Stream: ErrorType;
    type Error: Error;
}

/// A producer of stateful streams to the underlying data.
///
/// The concurrency semantics of the `Device` implementation will be inherited by the `FileSystem`.
///
/// If the `Device` only supports a single active stream, the `FileSystem` will be limited to either
/// a single management action or open `File` at any given time.  Conversely, if the `Device`
/// supports multiple streams, multiple actions may be performed and/or multiple `File`s may be
/// opened.
pub trait SyncDevice: Device {
    /// Runs the provided operation with a `Stream` reference that gives access to the underlying
    /// bytes of the `FileSystem`.
    fn with_stream<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&mut Self::Stream) -> R;
}

pub trait SyncFlushableDevice: SyncDevice {
    fn flush(&self) -> Result<(), Self::Error>;
}

/// An asynchronous producer of stateful streams to the underlying data.
///
/// The concurrency semantics of the `AsyncDevice` implementation will be inherited by the
/// `FileSystem`.
///
/// If the `AsyncDevice` only supports a single active stream, the `FileSystem` will be limited to
/// either a single management action or open `File` at any given time.  Conversely, if the
/// `AsyncDevice` supports multiple streams, multiple actions may be performed and/or multiple
/// `File`s may be opened.
pub trait AsyncDevice: Device {
    /// Runs the provided operation with a `Stream` reference that gives access to the underlying
    /// bytes of the `FileSystem`.
    fn with_stream<F, R>(&self, f: F) -> impl Future<Output = Result<R, Self::Error>>
    where
        F: AsyncFnOnce(&mut Self::Stream) -> R;
}

pub trait AsyncFlushableDevice: SyncDevice {
    fn flush(&self) -> impl Future<Output = Result<(), Self::Error>>;
}
