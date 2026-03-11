use super::*;
use crate::AsciiOnlyEncoder;
use crate::file_name::ShortFileName;

mod from_bytes {
    use super::*;

    #[test]
    fn free_all_following_parsed_correctly() {
        let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
        data[0] = 0x00;

        let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

        assert!(
            matches!(
                entry,
                DirectoryEntry::Free(FreeDirectoryEntry::AllFollowing)
            ),
            "AllFollowing free entry should be returned"
        );
    }

    #[test]
    fn free_current_only_parsed_correctly() {
        let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
        data[0] = 0xE5;

        let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

        assert!(
            matches!(entry, DirectoryEntry::Free(FreeDirectoryEntry::CurrentOnly)),
            "CurrentOnly free entry should be returned"
        );
    }

    #[test]
    fn short_name_parsed_correctly() {
        let short_name_entry = ShortNameDirectoryEntry::builder()
            .name(ShortFileName::from_str(&AsciiOnlyEncoder, "A").unwrap())
            .attributes(DirectoryEntryAttributes::empty())
            .first_cluster_number(2)
            .file_size(0)
            .build();

        let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
        short_name_entry.write(&mut data);

        let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

        assert!(
            matches!(entry, DirectoryEntry::ShortName(_)),
            "ShortName entry should be returned"
        );
    }

    #[test]
    fn short_name_error_propagated() {
        let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
        data[0] = 0x01;

        let error = DirectoryEntry::from_bytes(&data).expect_err("Err should be returned");

        assert!(
            matches!(error, DirectoryEntryError::ShortNameEntryInvalid(_)),
            "ShortNameEntryInvalid should be returned"
        );
    }

    #[test]
    fn long_name_parsed_correctly() {
        let mut utf16_code_units = [0xFFFF; LONG_NAME_CHARACTERS_PER_ENTRY];
        utf16_code_units[0] = 'A' as u16;
        utf16_code_units[1] = 0;

        let long_name_entry = LongNameDirectoryEntry::builder()
            .utf16_code_units(utf16_code_units)
            .order_byte(0x01)
            .short_name_checksum(0x00)
            .build();

        let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
        long_name_entry.write(&mut data);

        let entry = DirectoryEntry::from_bytes(&data).expect("Ok should be returned");

        assert!(
            matches!(entry, DirectoryEntry::LongName(_)),
            "LongName entry should be returned"
        );
    }

    #[test]
    fn long_name_error_propagated() {
        let mut data = [0x00; DIRECTORY_ENTRY_SIZE];
        data[0] = 0x3F;
        data[11] = 0x0F;

        let error = DirectoryEntry::from_bytes(&data).expect_err("Err should be returned");

        assert!(
            matches!(error, DirectoryEntryError::LongNameEntryInvalid(_)),
            "LongNameEntryInvalid should be returned"
        );
    }
}
