#[cfg(test)]
mod tests;

use crate::queued_resource::QueuedResource;
use crate::{
    AsciiOnlyEncoder, CodePageEncoder, DirectoryItemIterationError, Filesystem, FilesystemError,
};
use core::cell::RefCell;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

/// Helper for creating [Filesystem] instances
#[derive(Clone, Debug)]
pub struct FilesystemBuilder<S, const N: usize, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    stream: S,
    code_page_encoder: CPE,
    on_invalid_directory_entry: IDE,
}

impl<S> FilesystemBuilder<S, 1, AsciiOnlyEncoder, fn(DirectoryItemIterationError<S::Error>)>
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

impl<S, const N: usize, CPE, IDE> FilesystemBuilder<S, N, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    pub fn with_access_queue_length<const N2: usize>(self) -> FilesystemBuilder<S, N2, CPE, IDE> {
        FilesystemBuilder {
            stream: self.stream,
            code_page_encoder: self.code_page_encoder,
            on_invalid_directory_entry: self.on_invalid_directory_entry,
        }
    }

    /// Updates the builder to use the provided [CodePageEncoder].
    pub fn with_code_page_encoder<CPE2>(
        self,
        code_page_encoder: CPE2,
    ) -> FilesystemBuilder<S, N, CPE2, IDE>
    where
        CPE2: CodePageEncoder,
    {
        FilesystemBuilder {
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
    ) -> FilesystemBuilder<S, N, CPE, IDE2>
    where
        IDE2: Fn(DirectoryItemIterationError<S::Error>),
    {
        FilesystemBuilder {
            stream: self.stream,
            code_page_encoder: self.code_page_encoder,
            on_invalid_directory_entry,
        }
    }
}

#[cfg(feature = "sync")]
impl<S, const N: usize, CPE, IDE> FilesystemBuilder<S, N, CPE, IDE>
where
    S: Read + Seek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Creates the [Filesystem] instance based on the builder's current configuration.
    pub fn build(self) -> Result<Filesystem<S, N, CPE, IDE>, FilesystemError<S::Error>> {
        Filesystem::new(
            self.stream,
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
    }
}

#[cfg(feature = "async")]
impl<S, const N: usize, CPE, IDE> FilesystemBuilder<S, N, CPE, IDE>
where
    S: AsyncRead + AsyncSeek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Creates the [Filesystem] instance based on the builder's current configuration.
    pub async fn build_async(
        self,
    ) -> Result<Filesystem<S, N, CPE, IDE>, FilesystemError<S::Error>> {
        Filesystem::new_async(
            self.stream,
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
        .await
    }
}
