mod builder;
mod error;

pub use builder::*;
use core::error::Error;
pub use error::*;

use crate::Device;
use crate::allocation_table::AllocationTable;
use crate::boot_sector::BiosParameterBlock;
use crate::directory::{Directory, DirectoryFile, DirectoryTable};
use crate::directory_item::{DeviceDirectoryItemIterationError, DirectoryItem};
use crate::{AllocationTableKind, CodePageEncoder, File};
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use {
    crate::SyncDevice,
    embedded_io::{Read, Seek, Write},
};

#[cfg(feature = "async")]
use {
    crate::AsyncDevice,
    embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek, Write as AsyncWrite},
};

#[derive(Clone, Debug)]
pub struct FileSystem<D, CPE, IDE>
where
    D: Device,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    device: D,
    code_page_encoder: CPE,

    allocation_table: AllocationTable,
    bios_parameter_block: BiosParameterBlock,

    on_invalid_directory_entry: IDE,
}

impl<D, CPE, IDE> FileSystem<D, CPE, IDE>
where
    D: Device,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    /// The type of FAT filesystem the loaded instance is
    pub fn allocation_table_kind(&self) -> AllocationTableKind {
        self.allocation_table.kind()
    }

    fn root_directory(&self) -> Directory<'_, D> {
        let directory_table_entry_count = self.bios_parameter_block.directory_table_entry_count();

        if directory_table_entry_count > 0 {
            DirectoryTable::new(
                &self.device,
                self.bios_parameter_block.directory_table_base_address(),
                directory_table_entry_count,
            )
            .into()
        } else {
            DirectoryFile::new(
                &self.device,
                &self.allocation_table,
                self.bios_parameter_block.data_region_base_address(),
                self.bios_parameter_block.bytes_per_cluster(),
                self.bios_parameter_block
                    .root_directory_file_cluster_number(),
            )
            .into()
        }
    }

    fn directory_for(&'_ self, item: &DirectoryItem) -> Option<DirectoryFile<'_, D>> {
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

    fn file_for(&'_ self, item: &DirectoryItem) -> Option<File<'_, D>> {
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

    fn validate_boot_sector_signature<DE, SE>(
        boot_sector_bytes: &[u8; 512],
    ) -> Result<(), FileSystemError<DE, SE>>
    where
        DE: Error,
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
impl<D, S, CPE, IDE> FileSystem<D, CPE, IDE>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    pub fn new(
        mut device: D,
        code_page_encoder: CPE,
        on_invalid_directory_entry: IDE,
    ) -> Result<Self, FileSystemError<D::Error, S::Error>> {
        let mut boot_sector_bytes = [0; 512];

        device
            .with_stream(
                |stream| -> Result<(), FileSystemError<D::Error, S::Error>> {
                    stream.seek(SeekFrom::Start(0))?;

                    stream.read_exact(&mut boot_sector_bytes)?;

                    Ok(())
                },
            )
            .map_err(FileSystemError::DeviceError)?;

        Self::validate_boot_sector_signature(&boot_sector_bytes)?;

        let bios_parameter_block = BiosParameterBlock::from_boot_sector(&boot_sector_bytes)?;
        let allocation_table = AllocationTable::new(
            bios_parameter_block.allocation_table_kind(),
            bios_parameter_block.allocation_table_base_address(),
        );

        Ok(Self {
            device,
            code_page_encoder,

            allocation_table,
            bios_parameter_block,

            on_invalid_directory_entry,
        })
    }

    pub fn open(&self, file_path: &str) -> Option<File<'_, D>> {
        self.file_for(&self.find_item(file_path)?)
    }

    fn find_item(&self, file_path: &str) -> Option<DirectoryItem> {
        let mut current_directory = self.root_directory();
        let mut file_path_part_iterator = file_path.split("/");
        let mut file_path_part = file_path_part_iterator.next()?;

        loop {
            let mut item_iterator = current_directory.items();

            loop {
                let item = match item_iterator.next()? {
                    Ok(item) => item,
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
impl<D, S, CPE, IDE> FileSystem<D, CPE, IDE>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
    CPE: CodePageEncoder,
    IDE: Fn(DeviceDirectoryItemIterationError<D>),
{
    pub async fn new_async(
        mut device: D,
        code_page_encoder: CPE,
        on_invalid_directory_entry: IDE,
    ) -> Result<Self, FileSystemError<D::Error, S::Error>> {
        let mut boot_sector_bytes = [0; 512];

        device
            .with_stream(
                async |stream| -> Result<(), FileSystemError<D::Error, S::Error>> {
                    stream.seek(SeekFrom::Start(0)).await?;

                    stream.read_exact(&mut boot_sector_bytes).await?;

                    Ok(())
                },
            )
            .await
            .map_err(FileSystemError::DeviceError)?;

        Self::validate_boot_sector_signature(&boot_sector_bytes)?;

        let bios_parameter_block = BiosParameterBlock::from_boot_sector(&boot_sector_bytes)?;
        let allocation_table = AllocationTable::new(
            bios_parameter_block.allocation_table_kind(),
            bios_parameter_block.allocation_table_base_address(),
        );

        Ok(Self {
            device,
            code_page_encoder,

            allocation_table,
            bios_parameter_block,

            on_invalid_directory_entry,
        })
    }

    pub async fn open_async(&self, file_path: &str) -> Option<File<'_, D>> {
        self.file_for(&self.find_item_async(file_path).await?)
    }

    async fn find_item_async(&self, file_path: &str) -> Option<DirectoryItem> {
        let mut current_directory = self.root_directory();
        let mut file_path_part_iterator = file_path.split("/");
        let mut file_path_part = file_path_part_iterator.next()?;

        loop {
            let mut item_iterator = current_directory.items();

            loop {
                let item = match item_iterator.next_async().await? {
                    Ok(item) => item,
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
