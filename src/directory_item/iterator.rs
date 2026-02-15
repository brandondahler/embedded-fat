use crate::Device;
use crate::directory_entry::{
    DirectoryEntry, DirectoryEntryIterator, FreeDirectoryEntry, LONG_NAME_CHARACTERS_PER_ENTRY,
};
use crate::directory_item::{
    DIRECTORY_ENTITY_LONG_NAME_MAX_LENGTH, DeviceDirectoryItemIterationError, DirectoryItem,
    DirectoryItemBuilder, DirectoryItemError,
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

const MAX_ENTRY_COUNT: usize =
    DIRECTORY_ENTITY_LONG_NAME_MAX_LENGTH.div_ceil(LONG_NAME_CHARACTERS_PER_ENTRY) + 1;

#[derive(Clone, Debug)]
pub struct DirectoryItemIterator<'a, D>
where
    D: Device,
{
    entry_iterator: DirectoryEntryIterator<'a, D>,
}

impl<'a, D> DirectoryItemIterator<'a, D>
where
    D: Device,
{
    pub fn new(entry_iterator: DirectoryEntryIterator<'a, D>) -> Self {
        Self { entry_iterator }
    }

    fn should_skip_advancing_iterator(&self, directory_item_error: &DirectoryItemError) -> bool {
        matches!(directory_item_error, DirectoryItemError::LongNameOrphaned)
    }
}

#[cfg(feature = "sync")]
impl<D, S> DirectoryItemIterator<'_, D>
where
    D: SyncDevice<Stream = S>,
    S: Read + Seek,
{
    pub fn next(&mut self) -> Option<Result<DirectoryItem, DeviceDirectoryItemIterationError<D>>> {
        let mut is_first_entry = true;
        let mut builder = DirectoryItemBuilder::new();

        loop {
            let entry = match self.entry_iterator.peek() {
                Some(result) => match result {
                    Ok(entry) => entry,
                    Err(error) => {
                        propagate_iteration_error!(self.entry_iterator.advance());
                        return Some(Err(error.into()));
                    }
                },
                None => {
                    return if !is_first_entry {
                        Some(Err(DirectoryItemError::LongNameOrphaned.into()))
                    } else {
                        None
                    };
                }
            };

            match entry {
                DirectoryEntry::Free(free_entry) => {
                    propagate_iteration_error!(self.entry_iterator.advance());

                    if !is_first_entry {
                        return Some(Err(DirectoryItemError::LongNameOrphaned.into()));
                    }

                    match free_entry {
                        FreeDirectoryEntry::CurrentOnly => continue,
                        FreeDirectoryEntry::AllFollowing => return None,
                    }
                }
                DirectoryEntry::LongName(long_name_entry) => {
                    builder = match builder.add_long_name_entry(long_name_entry) {
                        Ok(builder) => {
                            propagate_iteration_error!(self.entry_iterator.advance());
                            builder
                        }
                        Err(directory_item_error) => {
                            if !self.should_skip_advancing_iterator(&directory_item_error) {
                                propagate_iteration_error!(self.entry_iterator.advance());
                            }

                            return Some(Err(directory_item_error.into()));
                        }
                    };
                }
                DirectoryEntry::ShortName(short_name_entry) => {
                    let item = propagate_iteration_error!(builder.build(short_name_entry));
                    propagate_iteration_error!(self.entry_iterator.advance());

                    return Some(Ok(item));
                }
            }

            is_first_entry = false;
        }
    }
}

#[cfg(feature = "async")]
impl<D, S> DirectoryItemIterator<'_, D>
where
    D: AsyncDevice<Stream = S>,
    S: AsyncRead + AsyncSeek,
{
    pub async fn next_async(
        &mut self,
    ) -> Option<Result<DirectoryItem, DeviceDirectoryItemIterationError<D>>> {
        let mut is_first_entry = true;
        let mut builder = DirectoryItemBuilder::new();

        loop {
            let entry = match self.entry_iterator.peek_async().await {
                Some(result) => propagate_iteration_error!(result),
                None => {
                    return if !is_first_entry {
                        Some(Err(DirectoryItemError::LongNameOrphaned.into()))
                    } else {
                        None
                    };
                }
            };

            match entry {
                DirectoryEntry::Free(free_entry) => {
                    propagate_iteration_error!(self.entry_iterator.advance_async().await);

                    if !is_first_entry {
                        return Some(Err(DirectoryItemError::LongNameOrphaned.into()));
                    }

                    match free_entry {
                        FreeDirectoryEntry::CurrentOnly => continue,
                        FreeDirectoryEntry::AllFollowing => return None,
                    }
                }
                DirectoryEntry::LongName(long_name_entry) => {
                    builder = match builder.add_long_name_entry(long_name_entry) {
                        Ok(builder) => {
                            propagate_iteration_error!(self.entry_iterator.advance_async().await);
                            builder
                        }
                        Err(directory_item_error) => {
                            if !self.should_skip_advancing_iterator(&directory_item_error) {
                                propagate_iteration_error!(
                                    self.entry_iterator.advance_async().await
                                );
                            }

                            return Some(Err(directory_item_error.into()));
                        }
                    };
                }
                DirectoryEntry::ShortName(short_name_entry) => {
                    let item = propagate_iteration_error!(builder.build(short_name_entry));
                    propagate_iteration_error!(self.entry_iterator.advance_async().await);

                    return Some(Ok(item));
                }
            }

            is_first_entry = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory_entry::{DirectoryEntryAttributes, ShortNameDirectoryEntry};
    use crate::file_name::ShortFileName;
    use crate::mock::{ScriptedDirectoryEntryIterator, VoidStream};
    use crate::{AsciiOnlyEncoder, SingleAccessDevice};

    #[cfg(feature = "sync")]
    mod next {
        use super::*;

        #[test]
        fn short_entry_only_built_successfully() {
            let expected_short_directory_entry = ShortNameDirectoryEntry::builder()
                .name(ShortFileName::from_str(&AsciiOnlyEncoder, "foo.txt").unwrap())
                .attributes(DirectoryEntryAttributes::empty())
                .first_cluster_number(2)
                .file_size(1)
                .build();

            let scripted_entry_iterator =
                ScriptedDirectoryEntryIterator::<SingleAccessDevice<VoidStream>>::new()
                    .with_peek(|index| match index {
                        0 => Some(Ok(expected_short_directory_entry.clone().into())),
                        _ => panic!("Shouldn't be reached"),
                    })
                    .with_advance(|index| Ok(index == 0));

            let mut item_iterator = DirectoryItemIterator::new(scripted_entry_iterator.into());

            let result = item_iterator
                .next()
                .expect("Some should be returned")
                .expect("Ok should be returned");

            assert_eq!(result.short_directory_entry, expected_short_directory_entry);
            assert_eq!(result.long_name, None);
        }
    }
}
