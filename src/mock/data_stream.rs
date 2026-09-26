use crate::Device;
use crate::mock::IoError;
use alloc::rc::Rc;
use core::borrow::Borrow;
use core::cell::{Ref, RefCell};
use core::cmp::min;
use core::ops::Deref;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek, Write};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};

#[derive(Clone, Debug)]
pub struct DataStream<E>
where
    E: AsRef<[u8]>,
{
    bytes: Rc<RefCell<E>>,
    position: usize,
}

impl<E> DataStream<E>
where
    E: AsRef<[u8]>,
{
    pub fn new(bytes: Rc<RefCell<E>>, position: usize) -> Self {
        Self { bytes, position }
    }

    pub fn from_bytes_ref_cell(bytes: Rc<RefCell<E>>) -> Self {
        Self::new(bytes, 0)
    }

    pub fn from_bytes(bytes: E) -> Self {
        Self::new(Rc::new(RefCell::new(bytes)), 0)
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        let bytes_ref = self.bytes.deref().borrow();
        let bytes = bytes_ref.as_ref();

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
            SeekFrom::End(value) => {
                let bytes_ref = self.bytes.deref().borrow();
                let bytes = bytes_ref.as_ref();

                (bytes.len() as i64 + value) as usize
            }
            SeekFrom::Current(value) => (self.position as i64 + value) as usize,
        };

        Ok(self.position as u64)
    }
}

impl<E> DataStream<E>
where
    E: AsRef<[u8]> + AsMut<[u8]>,
{
    fn write_internal(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        let mut bytes_ref = self.bytes.deref().borrow_mut();
        let mut bytes = bytes_ref.as_mut();

        bytes[self.position..(self.position + buf.len())].copy_from_slice(buf);

        Ok(buf.len())
    }
}

impl<E> ErrorType for DataStream<E>
where
    E: AsRef<[u8]>,
{
    type Error = IoError;
}

impl<E> Read for DataStream<E>
where
    E: AsRef<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl<E> AsyncRead for DataStream<E>
where
    E: AsRef<[u8]>,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl<E> Seek for DataStream<E>
where
    E: AsRef<[u8]>,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

impl<E> AsyncSeek for DataStream<E>
where
    E: AsRef<[u8]>,
{
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

impl<E> Write for DataStream<E>
where
    E: AsRef<[u8]> + AsMut<[u8]>,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write_internal(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<E> AsyncWrite for DataStream<E>
where
    E: AsRef<[u8]> + AsMut<[u8]>,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write_internal(buf)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
