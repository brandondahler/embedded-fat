#[cfg(test)]
mod tests;

use crate::DirectoryItemIterationError;
use crate::directory_entry::{
    DirectoryEntry, DirectoryEntryIterator, FreeDirectoryEntry, LONG_NAME_CHARACTERS_PER_ENTRY,
};
use crate::directory_item::{
    DIRECTORY_ENTITY_LONG_NAME_MAX_LENGTH, DirectoryItem, DirectoryItemBuilder, DirectoryItemError,
};
use embedded_io::{ErrorType, SeekFrom};

#[cfg(feature = "sync")]
use embedded_io::{Read, Seek};

#[cfg(feature = "async")]
use embedded_io_async::{Read as AsyncRead, Seek as AsyncSeek};

const MAX_ENTRY_COUNT: usize =
    DIRECTORY_ENTITY_LONG_NAME_MAX_LENGTH.div_ceil(LONG_NAME_CHARACTERS_PER_ENTRY) + 1;

#[derive(Clone, Debug)]
pub struct DirectoryItemIterator<'a, S>
where
    S: ErrorType,
{
    entry_iterator: DirectoryEntryIterator<'a, S>,
}

impl<'a, S> DirectoryItemIterator<'a, S>
where
    S: ErrorType,
{
    pub fn new(entry_iterator: DirectoryEntryIterator<'a, S>) -> Self {
        Self { entry_iterator }
    }

    fn should_skip_advancing_iterator(&self, directory_item_error: &DirectoryItemError) -> bool {
        matches!(directory_item_error, DirectoryItemError::LongNameOrphaned)
    }
}

#[cfg(feature = "sync")]
impl<S> DirectoryItemIterator<'_, S>
where
    S: Read + Seek,
{
    pub fn next(&mut self) -> Result<Option<DirectoryItem>, DirectoryItemIterationError<S::Error>> {
        let mut is_first_entry = true;
        let mut builder = DirectoryItemBuilder::new();

        loop {
            let entry = match self.entry_iterator.peek() {
                Ok(Some(entry)) => entry,
                Ok(None) => {
                    return if !is_first_entry {
                        Err(DirectoryItemError::LongNameOrphaned.into())
                    } else {
                        Ok(None)
                    };
                }
                Err(error) => {
                    self.entry_iterator.advance()?;
                    return Err(error.into());
                }
            };

            match entry {
                DirectoryEntry::Free(free_entry) => {
                    self.entry_iterator.advance()?;

                    if !is_first_entry {
                        return Err(DirectoryItemError::LongNameOrphaned.into());
                    }

                    match free_entry {
                        FreeDirectoryEntry::CurrentOnly => continue,
                        FreeDirectoryEntry::AllFollowing => return Ok(None),
                    }
                }
                DirectoryEntry::LongName(long_name_entry) => {
                    builder = match builder.add_long_name_entry(long_name_entry) {
                        Ok(builder) => {
                            self.entry_iterator.advance()?;
                            builder
                        }
                        Err(directory_item_error) => {
                            if !self.should_skip_advancing_iterator(&directory_item_error) {
                                self.entry_iterator.advance()?;
                            }

                            return Err(directory_item_error.into());
                        }
                    };
                }
                DirectoryEntry::ShortName(short_name_entry) => {
                    let item = builder.build(short_name_entry)?;
                    self.entry_iterator.advance()?;

                    return Ok(Some(item));
                }
            }

            is_first_entry = false;
        }
    }
}

#[cfg(feature = "async")]
impl<S> DirectoryItemIterator<'_, S>
where
    S: AsyncRead + AsyncSeek,
{
    pub async fn next_async(
        &mut self,
    ) -> Result<Option<DirectoryItem>, DirectoryItemIterationError<S::Error>> {
        let mut is_first_entry = true;
        let mut builder = DirectoryItemBuilder::new();

        loop {
            let entry = match self.entry_iterator.peek_async().await {
                Ok(Some(result)) => result,
                Ok(None) => {
                    return if !is_first_entry {
                        Err(DirectoryItemError::LongNameOrphaned.into())
                    } else {
                        Ok(None)
                    };
                }
                Err(error) => {
                    self.entry_iterator.advance_async().await?;
                    return Err(error.into());
                }
            };

            match entry {
                DirectoryEntry::Free(free_entry) => {
                    self.entry_iterator.advance_async().await;

                    if !is_first_entry {
                        return Err(DirectoryItemError::LongNameOrphaned.into());
                    }

                    match free_entry {
                        FreeDirectoryEntry::CurrentOnly => continue,
                        FreeDirectoryEntry::AllFollowing => return Ok(None),
                    }
                }
                DirectoryEntry::LongName(long_name_entry) => {
                    builder = match builder.add_long_name_entry(long_name_entry) {
                        Ok(builder) => {
                            self.entry_iterator.advance_async().await?;
                            builder
                        }
                        Err(directory_item_error) => {
                            if !self.should_skip_advancing_iterator(&directory_item_error) {
                                self.entry_iterator.advance_async().await?;
                            }

                            return Err(directory_item_error.into());
                        }
                    };
                }
                DirectoryEntry::ShortName(short_name_entry) => {
                    let item = builder.build(short_name_entry)?;
                    self.entry_iterator.advance_async().await?;

                    return Ok(Some(item));
                }
            }

            is_first_entry = false;
        }
    }
}
