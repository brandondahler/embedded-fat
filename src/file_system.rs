mod builder;
mod error;

#[cfg(test)]
mod tests;

pub use builder::*;
use core::cell::RefCell;
pub use error::*;

use crate::allocation_table::AllocationTable;
use crate::boot_sector::BiosParameterBlock;
use crate::device::Device;
use crate::directory::{Directory, DirectoryFile, DirectoryTable};
use crate::directory_entry::DirectoryEntryIterationError;
use crate::directory_item::DirectoryItem;
use crate::{AllocationTableKind, CodePageEncoder, DirectoryItemIterationError, File};
use core::error::Error;
use core::marker::PhantomData;
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek, Write};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite};

/// An instance of an either FAT12, FAT16, or FAT32 filesystem stored on the configured [Device].
/// Short names are encoded and decoded with the provided [CodePageEncoder].
///
/// Read and write operation capabilities are available based on the underlying [Device]'s
/// capabilities.  Synchronous operations require [SyncDevice]s while asynchronous operations
/// require [AsyncDevice]s.
///
/// Using multiple [FileSystem] instances on the same physical storage medium may result in read
/// and/or filesystem corruption.
#[derive(Debug)]
pub struct FileSystem<S, const N: usize, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    device: Device<S, N>,
    code_page_encoder: CPE,

    allocation_table: AllocationTable,
    bios_parameter_block: BiosParameterBlock,

    on_invalid_directory_entry: IDE,
}

impl<S, const N: usize, CPE, IDE> FileSystem<S, N, CPE, IDE>
where
    S: ErrorType,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// The underlying type of FAT filesystem that was loaded
    pub fn allocation_table_kind(&self) -> AllocationTableKind {
        self.allocation_table.kind()
    }

    fn root_directory(&self) -> Directory<'_, S, N> {
        match self
            .bios_parameter_block
            .root_directory_file_cluster_number()
        {
            Some(root_directory_file_cluster_number) => DirectoryFile::new(
                &self.device,
                &self.allocation_table,
                self.bios_parameter_block.data_region_base_address(),
                self.bios_parameter_block.bytes_per_cluster(),
                root_directory_file_cluster_number,
            )
            .into(),
            None => DirectoryTable::new(
                &self.device,
                self.bios_parameter_block.directory_table_base_address(),
                self.bios_parameter_block.directory_table_entry_count(),
            )
            .into(),
        }
    }

    fn directory_for(&'_ self, item: &DirectoryItem) -> Option<DirectoryFile<'_, S, N>> {
        if item.is_directory() {
            Some(DirectoryFile::new(
                &self.device,
                &self.allocation_table,
                self.bios_parameter_block.data_region_base_address(),
                self.bios_parameter_block.bytes_per_cluster(),
                item.first_cluster_number(),
            ))
        } else {
            None
        }
    }

    fn file_for(&'_ self, item: &DirectoryItem) -> Option<File<'_, S, N>> {
        if item.is_file() {
            Some(File::new(
                &self.device,
                &self.allocation_table,
                self.bios_parameter_block.data_region_base_address(),
                self.bios_parameter_block.bytes_per_cluster(),
                item.first_cluster_number(),
                item.file_size(),
            ))
        } else {
            None
        }
    }

    fn validate_boot_sector_signature<SE>(
        boot_sector_bytes: &[u8; 512],
    ) -> Result<(), FileSystemError<SE>>
    where
        SE: embedded_io::Error,
    {
        ensure!(
            boot_sector_bytes[510] == 0x55 && boot_sector_bytes[511] == 0xAA,
            FileSystemError::InvalidFatSignature
        );

        Ok(())
    }
}

#[cfg(feature = "sync")]
impl<S, const N: usize, CPE, IDE> FileSystem<S, N, CPE, IDE>
where
    S: Read + Seek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Loads an existing FAT filesystem from the provided `device`.
    pub fn new(
        mut device: Device<S, N>,
        code_page_encoder: CPE,
        on_invalid_directory_entry: IDE,
    ) -> Result<Self, FileSystemError<S::Error>> {
        let mut boot_sector_bytes = [0; 512];

        {
            let mut stream = device
                .acquire_stream()
                .map_err(|_| FileSystemError::DeviceBusy)?;

            stream.seek(SeekFrom::Start(0))?;
            stream.read_exact(&mut boot_sector_bytes)?;
        }

        Self::validate_boot_sector_signature(&boot_sector_bytes)?;

        let bios_parameter_block = BiosParameterBlock::from_boot_sector(&boot_sector_bytes)?;
        let allocation_table = AllocationTable::new(
            bios_parameter_block.allocation_table_kind(),
            bios_parameter_block.allocation_table_base_address(),
            bios_parameter_block.last_cluster_number(),
        );

        Ok(Self {
            device,
            code_page_encoder,

            allocation_table,
            bios_parameter_block,

            on_invalid_directory_entry,
        })
    }

    /// Searches for and opens an existing file.
    pub fn open(&self, file_path: &str) -> Option<File<'_, S, N>> {
        self.file_for(&self.find_item(file_path)?)
    }

    fn find_item(&self, file_path: &str) -> Option<DirectoryItem> {
        let mut current_directory = self.root_directory();
        let mut file_path_part_iterator = file_path.split("/");
        let mut file_path_part = file_path_part_iterator.next()?;

        loop {
            let iterator_directory = current_directory;
            let mut item_iterator = iterator_directory.items();

            loop {
                let item = match item_iterator.next() {
                    Ok(item) => item?,
                    Err(error) => {
                        (self.on_invalid_directory_entry)(error);
                        continue;
                    }
                };

                if item.is_match(&self.code_page_encoder, file_path_part) {
                    file_path_part = match file_path_part_iterator.next() {
                        Some(next_file_path_part) => next_file_path_part,
                        None => return Some(item),
                    };

                    current_directory = self.directory_for(&item)?.into();
                    break;
                }
            }
        }
    }
}

#[cfg(feature = "async")]
impl<S, const N: usize, CPE, IDE> FileSystem<S, N, CPE, IDE>
where
    S: AsyncRead + AsyncSeek,
    CPE: CodePageEncoder,
    IDE: Fn(DirectoryItemIterationError<S::Error>),
{
    /// Loads an existing FAT filesystem from the provided `device`.
    pub async fn new_async(
        device: Device<S, N>,
        code_page_encoder: CPE,
        on_invalid_directory_entry: IDE,
    ) -> Result<Self, FileSystemError<S::Error>> {
        let mut boot_sector_bytes = [0; 512];

        {
            let mut stream = device
                .acquire_stream_async()
                .await
                .map_err(|_| FileSystemError::DeviceBusy)?;

            stream.seek(SeekFrom::Start(0)).await?;
            stream.read_exact(&mut boot_sector_bytes).await?;
        }

        Self::validate_boot_sector_signature(&boot_sector_bytes)?;

        let bios_parameter_block = BiosParameterBlock::from_boot_sector(&boot_sector_bytes)?;
        let allocation_table = AllocationTable::new(
            bios_parameter_block.allocation_table_kind(),
            bios_parameter_block.allocation_table_base_address(),
            bios_parameter_block.last_cluster_number(),
        );

        Ok(Self {
            device,
            code_page_encoder,

            allocation_table,
            bios_parameter_block,

            on_invalid_directory_entry,
        })
    }

    /// Searches for and opens an existing file.
    pub async fn open_async(&self, file_path: &str) -> Option<File<'_, S, N>> {
        self.file_for(&self.find_item_async(file_path).await?)
    }

    async fn find_item_async(&self, file_path: &str) -> Option<DirectoryItem> {
        let mut current_directory = self.root_directory();
        let mut file_path_part_iterator = file_path.split("/");
        let mut file_path_part = file_path_part_iterator.next()?;

        loop {
            let iterator_directory = current_directory;
            let mut item_iterator = iterator_directory.items();

            loop {
                let item = match item_iterator.next_async().await {
                    Ok(item) => item?,
                    Err(error) => {
                        (self.on_invalid_directory_entry)(error);
                        continue;
                    }
                };

                if item.is_match(&self.code_page_encoder, file_path) {
                    file_path_part = match file_path_part_iterator.next() {
                        Some(next_file_path_part) => next_file_path_part,
                        None => return Some(item),
                    };

                    current_directory = self.directory_for(&item)?.into();
                    break;
                }
            }
        }
    }
}
