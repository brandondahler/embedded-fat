use embedded_io::{ErrorType, Read as SyncRead, Seek as SyncSeek, SeekFrom, Write as SyncWrite};
use std::fs::File;
use std::io::{Error, Read, Seek, Write};

pub struct StdFile {
    file: File,
}

impl StdFile {
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl ErrorType for StdFile {
    type Error = Error;
}

impl SyncRead for StdFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.file.read(buf)
    }
}

impl SyncSeek for StdFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.file.seek(pos.into())
    }
}

impl SyncWrite for StdFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.file.flush()
    }
}
