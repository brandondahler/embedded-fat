use crate::device::SyncDevice;
use crate::{AsyncDevice, Device};
use core::cmp::{max, min};
use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind, ErrorType, Read, Seek, SeekFrom};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

pub struct MockDevice<'a> {
    data: &'a [u8],
}

impl<'a> MockDevice<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn with_data(data: &'a [u8]) -> Self {
        Self::new(data)
    }
}

impl<'a> Device for MockDevice<'a> {
    type Stream = MockStream<'a>;
    type Error = MockError;
}

impl SyncDevice for MockDevice<'_> {
    fn with_stream<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&mut Self::Stream) -> R,
    {
        let mut stream = MockStream {
            data: self.data,
            position: 0,
        };

        Ok(f(&mut stream))
    }
}

impl AsyncDevice for MockDevice<'_> {
    async fn with_stream<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: AsyncFnOnce(&mut Self::Stream) -> R,
    {
        let mut stream = MockStream::new(self.data, 0);

        Ok(f(&mut stream).await)
    }
}

pub struct MockStream<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> MockStream<'a> {
    fn new(data: &'a [u8], position: usize) -> Self {
        Self { data, position }
    }

    pub fn with_data(data: &'a [u8]) -> Self {
        Self::new(data, 0)
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize, MockError> {
        let start = min(self.position, self.data.len());
        let end = min(start + buf.len(), self.data.len());

        let bytes_read = end - start;

        if bytes_read > 0 {
            buf[0..bytes_read].copy_from_slice(&self.data[start..end]);
            self.position += bytes_read;
        }

        Ok(bytes_read)
    }

    fn seek_internal(&mut self, pos: SeekFrom) -> Result<u64, MockError> {
        self.position = match pos {
            SeekFrom::Start(value) => value as usize,
            SeekFrom::End(value) => (self.data.len() as i64 + value) as usize,
            SeekFrom::Current(value) => (self.position as i64 + value) as usize,
        };

        Ok(self.position as u64)
    }
}

impl ErrorType for MockStream<'_> {
    type Error = MockError;
}

impl Read for MockStream<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl AsyncRead for MockStream<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl Seek for MockStream<'_> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

impl AsyncSeek for MockStream<'_> {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

#[derive(Debug)]
pub struct MockError;

impl core::error::Error for MockError {}

impl Display for MockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "MockError")
    }
}

impl Error for MockError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}
