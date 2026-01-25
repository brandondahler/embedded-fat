use crate::Device;
use crate::mock::IoError;
use core::borrow::Borrow;
use core::cmp::min;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

#[derive(Clone, Debug)]
pub struct DataStream<B>
where
    B: Borrow<[u8]>,
{
    bytes: B,
    position: usize,
}

impl<B> DataStream<B>
where
    B: Borrow<[u8]>,
{
    pub fn new(bytes: B, position: usize) -> Self {
        Self { bytes, position }
    }

    pub fn from_bytes(bytes: B) -> Self {
        Self::new(bytes, 0)
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        let bytes = self.bytes.borrow();

        let start = min(self.position, bytes.len());
        let end = min(start + buf.len(), bytes.len());

        let bytes_read = end - start;

        if bytes_read > 0 {
            buf[0..bytes_read].copy_from_slice(&bytes[start..end]);
            self.position += bytes_read;
        }

        Ok(bytes_read)
    }

    fn seek_internal(&mut self, pos: SeekFrom) -> Result<u64, IoError> {
        self.position = match pos {
            SeekFrom::Start(value) => value as usize,
            SeekFrom::End(value) => (self.bytes.borrow().len() as i64 + value) as usize,
            SeekFrom::Current(value) => (self.position as i64 + value) as usize,
        };

        Ok(self.position as u64)
    }
}

impl<D> ErrorType for DataStream<D>
where
    D: Borrow<[u8]>,
{
    type Error = IoError;
}

impl<D> Read for DataStream<D>
where
    D: Borrow<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl<D> AsyncRead for DataStream<D>
where
    D: Borrow<[u8]>,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl<D> Seek for DataStream<D>
where
    D: Borrow<[u8]>,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

impl<D> AsyncSeek for DataStream<D>
where
    D: Borrow<[u8]>,
{
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}
