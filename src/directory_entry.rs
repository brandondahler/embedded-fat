mod attributes;
mod error;
mod free;
mod iterator;
mod long_name;
mod short_name;

pub use attributes::*;
pub use error::*;
pub use free::*;
pub use iterator::*;
pub use long_name::*;
pub use short_name::*;

use embedded_io::{Seek, Write};
use embedded_io_async::{Seek as AsyncSeek, Write as AsyncWrite};

pub const DIRECTORY_ENTRY_SIZE: usize = 32;

pub enum DirectoryEntry {
    Free(FreeDirectoryEntry),
    ShortName(ShortNameDirectoryEntry),
    LongName(LongNameDirectoryEntry),
}

impl DirectoryEntry {
    pub fn from_bytes(
        entry_bytes: &[u8; DIRECTORY_ENTRY_SIZE],
    ) -> Result<DirectoryEntry, DirectoryEntryError> {
        if matches!(entry_bytes[0], 0x00) {
            Ok(FreeDirectoryEntry::AllFollowing.into())
        } else if matches!(entry_bytes[0], 0xE5) {
            Ok(FreeDirectoryEntry::CurrentOnly.into())
        } else if entry_bytes[11] & 0x0F > 0 {
            Ok(LongNameDirectoryEntry::from_bytes(entry_bytes)?.into())
        } else {
            Ok(ShortNameDirectoryEntry::from_bytes(entry_bytes)?.into())
        }
    }
}

impl From<FreeDirectoryEntry> for DirectoryEntry {
    fn from(value: FreeDirectoryEntry) -> Self {
        Self::Free(value)
    }
}

impl From<LongNameDirectoryEntry> for DirectoryEntry {
    fn from(value: LongNameDirectoryEntry) -> Self {
        Self::LongName(value)
    }
}

impl From<ShortNameDirectoryEntry> for DirectoryEntry {
    fn from(value: ShortNameDirectoryEntry) -> Self {
        Self::ShortName(value)
    }
}
