use crate::device::SyncDevice;
use crate::mock::IoError;
use crate::{AsyncDevice, Device};
use core::cmp::{max, min};
use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind, ErrorType, Read, Seek, SeekFrom};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

pub struct DataStream<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> DataStream<'a> {
    pub fn new(data: &'a [u8], position: usize) -> Self {
        Self { data, position }
    }

    pub fn with_data(data: &'a [u8]) -> Self {
        Self::new(data, 0)
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        let start = min(self.position, self.data.len());
        let end = min(start + buf.len(), self.data.len());

        let bytes_read = end - start;

        if bytes_read > 0 {
            buf[0..bytes_read].copy_from_slice(&self.data[start..end]);
            self.position += bytes_read;
        }

        Ok(bytes_read)
    }

    fn seek_internal(&mut self, pos: SeekFrom) -> Result<u64, IoError> {
        self.position = match pos {
            SeekFrom::Start(value) => value as usize,
            SeekFrom::End(value) => (self.data.len() as i64 + value) as usize,
            SeekFrom::Current(value) => (self.position as i64 + value) as usize,
        };

        Ok(self.position as u64)
    }
}

impl ErrorType for DataStream<'_> {
    type Error = IoError;
}

impl Read for DataStream<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl AsyncRead for DataStream<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl Seek for DataStream<'_> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

impl AsyncSeek for DataStream<'_> {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}
