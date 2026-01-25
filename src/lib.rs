#![cfg_attr(not(test), no_std)]
#![allow(dead_code, unused)]

#[cfg(test)]
extern crate alloc;

#[macro_use]
mod utils;

mod allocation_table;
mod boot_sector;
mod device;
mod directory;
mod directory_entry;
mod directory_item;
mod encoding;
mod file;
mod file_name;
mod file_system;

#[cfg(test)]
mod mock;

pub use allocation_table::AllocationTableKind;
pub use boot_sector::BiosParameterBlockError;
pub use device::{Device, SingleAccessDevice, SingleAccessDeviceError};
pub use directory_entry::{
    DirectoryEntryError, LongNameDirectoryEntryError, ShortNameDirectoryEntryError,
};
pub use directory_item::{DirectoryItemError, DirectoryItemIterationError};
pub use encoding::{AsciiOnlyEncoder, CodePageEncoder};
pub use file::{File, FileError};
pub use file_system::{FileSystem, FileSystemBuilder, FileSystemError};

#[cfg(feature = "sync")]
pub use device::{SyncDevice, SyncFlushableDevice};

#[cfg(feature = "async")]
pub use device::{AsyncDevice, AsyncFlushableDevice};
