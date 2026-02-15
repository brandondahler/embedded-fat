mod builder;
mod error;
mod iteration_error;
mod iterator;

pub use builder::*;
pub use error::*;
pub use iteration_error::*;
pub use iterator::*;

use crate::directory_entry::ShortNameDirectoryEntry;
use crate::file_name::{LongFileName, ShortFileName};
use crate::{AllocationTableKind, CodePageEncoder};

pub const DIRECTORY_ENTITY_LONG_NAME_MAX_LENGTH: usize = 255;

#[derive(Clone, Debug)]
pub struct DirectoryItem {
    short_directory_entry: ShortNameDirectoryEntry,
    long_name: Option<LongFileName>,
}

impl DirectoryItem {
    pub fn new(
        short_directory_entry: ShortNameDirectoryEntry,
        long_name: Option<LongFileName>,
    ) -> Self {
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
        if let Some(item_long_name) = self.long_name.as_ref()
            && let Ok(input_long_name) = LongFileName::from_str(file_name)
            && item_long_name == &input_long_name
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
