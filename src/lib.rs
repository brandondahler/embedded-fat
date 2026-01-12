#![cfg_attr(not(test), no_std)]
#![allow(dead_code, unused)]

#[cfg(test)]
extern crate alloc;

#[macro_use]
mod utils;

mod allocation_table;
mod bios_parameter_block;
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
pub use bios_parameter_block::BiosParameterBlockError;
pub use device::{
    AsyncDevice, AsyncFlushableDevice, Device, SingleAccessDevice, SingleAccessDeviceError,
    SyncFlushableDevice,
};
pub use directory_entry::{
    DirectoryEntryError, LongNameDirectoryEntryError, LongNameDirectoryEntryNameError,
    ShortNameDirectoryEntryError, ShortNameDirectoryEntryNameError,
};
pub use directory_item::{DirectoryItemError, DirectoryItemIterationError};
pub use encoding::{AsciiOnlyEncoder, CharacterEncodingError, CodePageEncoder};
pub use file::{File, FileError};
pub use file_system::{FileSystem, FileSystemBuilder, FileSystemError};
