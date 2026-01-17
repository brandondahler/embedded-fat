use crate::Device;
use bitflags::bitflags;
use core::fmt::Display;
use embedded_io::{ErrorType, Read, Seek, SeekFrom, Write};
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct ErroringStreamScenarios: u8 {
        const READ  = 1 << 0;
        const SEEK  = 1 << 1;
        const WRITE = 1 << 2;
        const FLUSH = 1 << 3;

        const ALL = ErroringStreamScenarios::READ.bits()
            | ErroringStreamScenarios::SEEK.bits()
            | ErroringStreamScenarios::WRITE.bits()
            | ErroringStreamScenarios::FLUSH.bits();
    }
}

#[derive(Clone, Debug)]
pub struct ErroringStream<S, E>
where
    S: ErrorType<Error = E>,
    E: embedded_io::Error + Clone,
{
    stream: S,

    error: E,
    error_scenarios: ErroringStreamScenarios,
}

impl<S, E> ErroringStream<S, E>
where
    S: ErrorType<Error = E>,
    E: embedded_io::Error + Clone,
{
    pub fn new(stream: S, error: E, error_scenarios: ErroringStreamScenarios) -> Self {
        Self {
            stream,
            error,
            error_scenarios,
        }
    }
}

impl<S, E> ErrorType for ErroringStream<S, E>
where
    S: ErrorType<Error = E>,
    E: embedded_io::Error + Clone,
{
    type Error = E;
}

impl<S, E> Read for ErroringStream<S, E>
where
    S: ErrorType<Error = E> + Read,
    E: embedded_io::Error + Clone,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.error_scenarios.contains(ErroringStreamScenarios::READ) {
            Err(self.error.clone())
        } else {
            self.stream.read(buf)
        }
    }
}

impl<S, E> AsyncRead for ErroringStream<S, E>
where
    S: ErrorType<Error = E> + AsyncRead,
    E: embedded_io::Error + Clone,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.error_scenarios.contains(ErroringStreamScenarios::READ) {
            Err(self.error.clone())
        } else {
            self.stream.read(buf).await
        }
    }
}

impl<S, E> Seek for ErroringStream<S, E>
where
    S: ErrorType<Error = E> + Seek,
    E: embedded_io::Error + Clone,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        if self.error_scenarios.contains(ErroringStreamScenarios::SEEK) {
            Err(self.error.clone())
        } else {
            self.stream.seek(pos)
        }
    }
}

impl<S, E> AsyncSeek for ErroringStream<S, E>
where
    S: ErrorType<Error = E> + AsyncSeek,
    E: embedded_io::Error + Clone,
{
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        if self.error_scenarios.contains(ErroringStreamScenarios::SEEK) {
            Err(self.error.clone())
        } else {
            self.stream.seek(pos).await
        }
    }
}

impl<S, E> Write for ErroringStream<S, E>
where
    S: ErrorType<Error = E> + Write,
    E: embedded_io::Error + Clone,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if self
            .error_scenarios
            .contains(ErroringStreamScenarios::WRITE)
        {
            Err(self.error.clone())
        } else {
            self.stream.write(buf)
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        if self
            .error_scenarios
            .contains(ErroringStreamScenarios::FLUSH)
        {
            Err(self.error.clone())
        } else {
            self.stream.flush()
        }
    }
}

impl<S, E> AsyncWrite for ErroringStream<S, E>
where
    S: ErrorType<Error = E> + AsyncWrite,
    E: embedded_io::Error + Clone,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if self
            .error_scenarios
            .contains(ErroringStreamScenarios::WRITE)
        {
            Err(self.error.clone())
        } else {
            self.stream.write(buf).await
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        if self
            .error_scenarios
            .contains(ErroringStreamScenarios::FLUSH)
        {
            Err(self.error.clone())
        } else {
            self.stream.flush().await
        }
    }
}
