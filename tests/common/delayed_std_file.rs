use embedded_io::{ErrorType, Read as SyncRead, Seek as SyncSeek, SeekFrom, Write as SyncWrite};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};
use std::fs::File;
use std::io::{Error, Read, Seek, Write};
use std::time::Duration;

#[derive(Debug)]
pub struct DelayedStdFile {
    file: File,
    delay: Duration,
    use_tokio: bool,
}

impl DelayedStdFile {
    pub fn new(file: File, delay: Duration, use_tokio: bool) -> Self {
        Self {
            file,
            delay,
            use_tokio,
        }
    }

    fn sleep(&self) {
        std::thread::sleep(self.delay);
    }

    async fn sleep_async(&self) {
        if self.use_tokio {
            tokio::time::sleep(self.delay).await;
        } else {
            std::thread::sleep(self.delay);
        }
    }
}

impl ErrorType for DelayedStdFile {
    type Error = Error;
}

impl SyncRead for DelayedStdFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.sleep();
        self.file.read(buf)
    }
}

impl SyncSeek for DelayedStdFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.sleep();
        self.file.seek(pos.into())
    }
}

impl SyncWrite for DelayedStdFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.sleep();
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.sleep();
        self.file.flush()
    }
}

impl AsyncRead for DelayedStdFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.sleep_async().await;
        self.file.read(buf)
    }
}

impl AsyncSeek for DelayedStdFile {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        self.sleep_async().await;
        self.file.seek(pos.into())
    }
}

impl AsyncWrite for DelayedStdFile {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.sleep_async().await;
        self.file.write(buf)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.sleep_async().await;
        self.file.flush()
    }
}
