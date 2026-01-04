mod builder;
mod error;
mod iteration_error;
mod iterator;

pub use builder::*;
pub use error::*;
pub use iteration_error::*;
pub use iterator::*;

use crate::CodePageEncoder;
use crate::directory_entry::ShortNameDirectoryEntry;
use crate::encoding::Ucs2Character;
use crate::file_name::{LongFileName, ShortFileName};

pub const DIRECTORY_ENTITY_LONG_NAME_MAX_LENGTH: usize = 255;

pub struct DirectoryItem {
    short_directory_entry: ShortNameDirectoryEntry,
    long_name: LongFileName,
}

impl DirectoryItem {
    pub fn new(short_directory_entry: ShortNameDirectoryEntry, long_name: LongFileName) -> Self {
        Self {
            short_directory_entry,
            long_name,
        }
    }

    pub fn is_directory(&self) -> bool {
        self.short_directory_entry.is_directory()
    }

    pub fn is_file(&self) -> bool {
        !self.is_directory()
    }

    pub fn first_cluster_number(&self) -> u32 {
        self.short_directory_entry.first_cluster_number()
    }

    pub fn file_size(&self) -> u32 {
        self.short_directory_entry.file_size()
    }

    pub fn is_match<CPE>(&self, code_page_encoder: &CPE, file_name: &str) -> bool
    where
        CPE: CodePageEncoder,
    {
        if !self.long_name.is_empty()
            && let Ok(long_name) = LongFileName::from_str(file_name)
            && self.long_name == long_name
        {
            return true;
        }

        if let Ok(short_name) = ShortFileName::from_str(code_page_encoder, file_name)
            && *self.short_directory_entry.name() == short_name
        {
            return true;
        }

        false
    }
}
