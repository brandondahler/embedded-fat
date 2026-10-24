#[cfg(test)]
mod tests;

use crate::{
    AsciiOnlyEncoder, CodePageEncoder, DirectoryItemIterationError, FileSystem, FileSystemError,
};
use core::cell::RefCell;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

use crate::device::Device;
#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

/// Helper for creating [FileSystem] instances
#[derive(Clone, Debug)]
pub struct FileSystemBuilder<S, const N: usize, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    stream: S,
    code_page_encoder: CPE,
    on_invalid_directory_entry: IDE,
}

impl<S> FileSystemBuilder<S, 1, AsciiOnlyEncoder, fn(DirectoryItemIterationError<S::Error>)>
where
    S: ErrorType,
{
    /// Creates an instance with the provided stream, no queue, and the default [AsciiOnlyEncoder].
    pub fn from_stream(stream: S) -> Self {
        Self {
            stream,
            code_page_encoder: AsciiOnlyEncoder,
            on_invalid_directory_entry: |_| {},
        }
    }
}

impl<S, const N: usize, CPE, IDE> FileSystemBuilder<S, N, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    pub fn with_device_queue_length<const N2: usize>(self) -> FileSystemBuilder<S, N2, CPE, IDE> {
        FileSystemBuilder {
            stream: self.stream,
            code_page_encoder: self.code_page_encoder,
            on_invalid_directory_entry: self.on_invalid_directory_entry,
        }
    }

    /// Updates the builder to use the provided [CodePageEncoder].
    pub fn with_code_page_encoder<CPE2>(
        self,
        code_page_encoder: CPE2,
    ) -> FileSystemBuilder<S, N, CPE2, IDE>
    where
        CPE2: CodePageEncoder,
    {
        FileSystemBuilder {
            stream: self.stream,
            code_page_encoder,
            on_invalid_directory_entry: self.on_invalid_directory_entry,
        }
    }

    /// Updates the builder to use the provided callback when an invalid directory entry is
    /// encountered.
    pub fn on_invalid_directory_entry<IDE2>(
        self,
        on_invalid_directory_entry: IDE2,
    ) -> FileSystemBuilder<S, N, CPE, IDE2>
    where
        IDE2: Fn(DirectoryItemIterationError<S::Error>),
    {
        FileSystemBuilder {
            stream: self.stream,
            code_page_encoder: self.code_page_encoder,
            on_invalid_directory_entry,
        }
    }
}

#[cfg(feature = "sync")]
impl<S, const N: usize, CPE, IDE> FileSystemBuilder<S, N, CPE, IDE>
where
    S: Read + Seek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Creates the [FileSystem] instance based on the builder's current configuration.
    pub fn build(self) -> Result<FileSystem<S, N, CPE, IDE>, FileSystemError<S::Error>> {
        FileSystem::new(
            Device::new(self.stream),
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
    }
}

#[cfg(feature = "async")]
impl<S, const N: usize, CPE, IDE> FileSystemBuilder<S, N, CPE, IDE>
where
    S: AsyncRead + AsyncSeek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Creates the [FileSystem] instance based on the builder's current configuration.
    pub async fn build_async(
        self,
    ) -> Result<FileSystem<S, N, CPE, IDE>, FileSystemError<S::Error>> {
        FileSystem::new_async(
            Device::new(self.stream),
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
        .await
    }
}
