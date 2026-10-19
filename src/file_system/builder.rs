#[cfg(test)]
mod tests;

use crate::{
    AsciiOnlyEncoder, CodePageEncoder, DirectoryItemIterationError, FileSystem, FileSystemError,
};
use core::cell::RefCell;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

/// Helper for creating [FileSystem] instances
#[derive(Clone, Debug)]
pub struct FileSystemBuilder<S, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    device: RefCell<S>,
    code_page_encoder: CPE,
    on_invalid_directory_entry: IDE,
}

impl<S> FileSystemBuilder<S, AsciiOnlyEncoder, fn(DirectoryItemIterationError<S::Error>)>
where
    S: ErrorType,
{
    /// Creates an instance with the provided stream and the default [AsciiOnlyEncoder].
    pub fn from_stream(stream: S) -> Self {
        Self::from_device(RefCell::new(stream))
    }

    /// Creates an instance with the provided device and the default [AsciiOnlyEncoder].
    pub fn from_device(device: RefCell<S>) -> Self {
        Self {
            device,
            code_page_encoder: AsciiOnlyEncoder,
            on_invalid_directory_entry: |_| {},
        }
    }
}

impl<S, CPE, IDE> FileSystemBuilder<S, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Updates the builder to use the provided [CodePageEncoder].
    pub fn with_code_page_encoder<CPE2>(
        self,
        code_page_encoder: CPE2,
    ) -> FileSystemBuilder<S, CPE2, IDE>
    where
        CPE2: CodePageEncoder,
    {
        FileSystemBuilder {
            device: self.device,
            code_page_encoder,
            on_invalid_directory_entry: self.on_invalid_directory_entry,
        }
    }

    /// Updates the builder to use the provided callback when an invalid directory entry is
    /// encountered.
    pub fn on_invalid_directory_entry<IDE2>(
        self,
        on_invalid_directory_entry: IDE2,
    ) -> FileSystemBuilder<S, CPE, IDE2>
    where
        IDE2: Fn(DirectoryItemIterationError<S::Error>),
    {
        FileSystemBuilder {
            device: self.device,
            code_page_encoder: self.code_page_encoder,
            on_invalid_directory_entry,
        }
    }
}

#[cfg(feature = "sync")]
impl<S, CPE, IDE> FileSystemBuilder<S, CPE, IDE>
where
    S: Read + Seek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Creates the [FileSystem] instance based on the builder's current configuration.
    pub fn build(self) -> Result<FileSystem<S, CPE, IDE>, FileSystemError<S::Error>> {
        FileSystem::new(
            self.device,
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
    }
}

#[cfg(feature = "async")]
impl<S, CPE, IDE> FileSystemBuilder<S, CPE, IDE>
where
    S: AsyncRead + AsyncSeek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Creates the [FileSystem] instance based on the builder's current configuration.
    pub async fn build_async(self) -> Result<FileSystem<S, CPE, IDE>, FileSystemError<S::Error>> {
        FileSystem::new_async(
            self.device,
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
        .await
    }
}
