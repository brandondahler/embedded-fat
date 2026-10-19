use super::*;
use crate::AsciiOnlyEncoder;
use crate::directory_entry::{DirectoryEntryAttributes, ShortNameDirectoryEntry};
use crate::file_name::ShortFileName;
use crate::mock::{ScriptedDirectoryEntryIterator, VoidStream};

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

        let scripted_entry_iterator = ScriptedDirectoryEntryIterator::<VoidStream>::new()
            .with_peek(|index| match index {
                0 => Ok(Some(expected_short_directory_entry.clone().into())),
                _ => panic!("Shouldn't be reached"),
            })
            .with_advance(|index| Ok(index == 0));

        let mut item_iterator = DirectoryItemIterator::new(scripted_entry_iterator.into());

        let result = item_iterator.next().unwrap().unwrap();

        assert_eq!(result.short_directory_entry, expected_short_directory_entry);
        assert_eq!(result.long_name, None);
    }
}
