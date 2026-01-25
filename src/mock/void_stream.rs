use crate::mock::IoError;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek, Write};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};

#[derive(Clone, Debug)]
pub struct VoidStream {
    position: u64,
}

impl VoidStream {
    pub fn new() -> Self {
        Self { position: 0 }
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        for byte in buf.iter_mut() {
            *byte = 0;
        }

        self.position += buf.len() as u64;

        Ok(buf.len())
    }

    fn seek_internal(&mut self, pos: SeekFrom) -> Result<u64, IoError> {
        self.position = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(offset) => (self.position as i64 + offset) as u64,
            SeekFrom::End(offset) => (self.position as i64 + offset) as u64,
        };

        Ok(self.position)
    }
}

impl ErrorType for VoidStream {
    type Error = IoError;
}

impl Read for VoidStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl AsyncRead for VoidStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_internal(buf)
    }
}

impl Seek for VoidStream {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

impl AsyncSeek for VoidStream {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.seek_internal(pos)
    }
}

impl Write for VoidStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl AsyncWrite for VoidStream {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
