use crate::directory_item::DeviceDirectoryItemIterationError;
use crate::{
    AsciiOnlyEncoder, CodePageEncoder, Device, FileSystem, FileSystemError, SingleAccessDevice,
};
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use {
    crate::SyncDevice,
    embedded_io::{Read, Seek},
};

#[cfg(feature = "async")]
use {
    crate::AsyncDevice,
    embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek},
};

type FileSystemBuilderResult<D, CPE, IDE> = Result<
    FileSystem<D, CPE, IDE>,
    FileSystemError<<D as Device>::Error, <<D as Device>::Stream as ErrorType>::Error>,
>;

#[derive(Clone, Debug)]
pub struct FileSystemBuilder<D, CPE, IDE>
where
    D: Device,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    device: D,
    code_page_encoder: CPE,
    on_invalid_directory_entry: IDE,
}

impl<D> FileSystemBuilder<D, AsciiOnlyEncoder, fn(DeviceDirectoryItemIterationError<D>)>
where
    D: Device,
{
    pub fn from_device(device: D) -> Self {
        Self {
            device,
            code_page_encoder: AsciiOnlyEncoder,
            on_invalid_directory_entry: |_| {},
        }
    }
}

impl<S>
    FileSystemBuilder<
        SingleAccessDevice<S>,
        AsciiOnlyEncoder,
        fn(DeviceDirectoryItemIterationError<SingleAccessDevice<S>>),
    >
where
    S: ErrorType,
{
    pub fn from_stream(stream: S) -> Self {
        Self {
            device: SingleAccessDevice::new(stream),
            code_page_encoder: AsciiOnlyEncoder,
            on_invalid_directory_entry: |_| {},
        }
    }
}

impl<D, CPE, IDE> FileSystemBuilder<D, CPE, IDE>
where
    D: Device,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    pub fn with_code_page_encoder<CPE2>(
        self,
        code_page_encoder: CPE2,
    ) -> FileSystemBuilder<D, CPE2, IDE>
    where
        CPE2: CodePageEncoder,
    {
        FileSystemBuilder {
            device: self.device,
            code_page_encoder,
            on_invalid_directory_entry: self.on_invalid_directory_entry,
        }
    }

    pub fn on_invalid_directory_entry<IDE2>(
        self,
        on_invalid_directory_entry: IDE2,
    ) -> FileSystemBuilder<D, CPE, IDE2>
    where
        IDE2: Fn(DeviceDirectoryItemIterationError<D>),
    {
        FileSystemBuilder {
            device: self.device,
            code_page_encoder: self.code_page_encoder,
            on_invalid_directory_entry,
        }
    }
}

#[cfg(feature = "sync")]
impl<D, S, CPE, IDE> FileSystemBuilder<D, CPE, IDE>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    pub fn build(self) -> FileSystemBuilderResult<D, CPE, IDE> {
        FileSystem::new(
            self.device,
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
    }
}

#[cfg(feature = "async")]
impl<D, S, CPE, IDE> FileSystemBuilder<D, CPE, IDE>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    pub async fn build_async(self) -> FileSystemBuilderResult<D, CPE, IDE> {
        FileSystem::new_async(
            self.device,
            self.code_page_encoder,
            self.on_invalid_directory_entry,
        )
        .await
    }
}
