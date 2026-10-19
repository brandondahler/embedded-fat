//! # Embedded FAT
//!
//! `embedded-fat` is a pure rust implementation of the FAT12, FAT16, and FAT32 filesystems
//! primarily focused on consumption by embedded systems.
//!
//! This implementation specifically prioritizes:
//! * Stack-only memory allocation
//!   * Implemented as `no_std` and does not require an `alloc` implementation.
//! * Minimal binary output size
//!   * Features which are not strictly required, such as Unicode case insensitivity, can be opted
//!     out of in order to reduce the final binary size.
//! * First-class asynchronous support
//!   * Unlike programs running on traditional operating systems, embedded systems often do not have
//!     threading to provide native context switching.
//!   * Instead, embedded systems often utilize asynchronous runtimes to context switch between
//!     the active program loop and offloading work to dedicated peripherals.

#![cfg_attr(not(test), no_std)]
#![allow(dead_code, unused)]

#[cfg(test)]
extern crate alloc;

#[macro_use]
mod utils;

mod allocation_table;
mod boot_sector;
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
pub use directory_entry::{
    DirectoryEntryError, LongNameDirectoryEntryError, ShortNameDirectoryEntryError,
};
pub use directory_item::{DirectoryItemError, DirectoryItemIterationError};
pub use encoding::{AsciiOnlyEncoder, CodePageEncoder};
pub use file::{File, FileError};
pub use file_system::{FileSystem, FileSystemBuilder, FileSystemError};
