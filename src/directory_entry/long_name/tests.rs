use super::*;
use crate::directory_entry::DirectoryEntryAttributes;

mod from_bytes {
    use super::*;

    #[test]
    fn parses_entry_correctly() {
        let mut test_data = TestData::valid();

        let entry = LongNameDirectoryEntry::from_bytes(&mut test_data.bytes)
            .expect("Ok should be returned");

        assert_eq!(
            entry.is_last_entry(),
            test_data.is_last_entry,
            "is_last_entry should parse correctly"
        );
        assert_eq!(
            entry.entry_number(),
            test_data.entry_number,
            "entry_number should be parsed correctly"
        );
        assert_eq!(
            entry.short_name_checksum(),
            test_data.short_name_checksum,
            "short_name_checksum should be parsed correctly"
        );
        assert_eq!(
            entry.utf16_code_units(),
            &test_data.name_utf16_code_units,
            "utf16_code_units should be parsed correctly"
        );
    }

    #[test]
    fn entry_number_zero_returns_err() {
        let mut data = TestData::valid().bytes;
        data[0] = 0x00;

        let entry =
            LongNameDirectoryEntry::from_bytes(&mut data).expect_err("Err should be returned");

        assert!(
            matches!(entry, LongNameDirectoryEntryError::EntryNumberInvalid),
            "EntryNumberInvalid should be returned"
        );
    }

    #[test]
    fn entry_number_too_large_returns_err() {
        let mut data = TestData::valid().bytes;
        data[0] = 0x3F;

        let error =
            LongNameDirectoryEntry::from_bytes(&mut data).expect_err("Err should be returned");

        assert!(
            matches!(error, LongNameDirectoryEntryError::EntryNumberInvalid),
            "EntryNumberInvalid should be returned"
        );
    }
}

mod write {
    use super::*;

    #[test]
    fn roundtrips_correctly() {
        let data = TestData::valid().bytes;
        let entry = LongNameDirectoryEntry::from_bytes(&data).expect("Ok should be returned");

        let mut result = [0x00; DIRECTORY_ENTRY_SIZE];
        entry.write(&mut result);

        assert_eq!(result, data, "Input and output bytes should match exactly");
    }
}

struct TestData {
    bytes: [u8; DIRECTORY_ENTRY_SIZE],

    is_last_entry: bool,
    entry_number: u8,
    short_name_checksum: u8,
    name_utf16_code_units: [Utf16CodeUnit; LONG_NAME_CHARACTERS_PER_ENTRY],
}

impl TestData {
    fn valid() -> Self {
        Self {
            #[rustfmt::skip]
                bytes: [
                    // Order byte
                    0x41,

                    // Name stride 1
                    0x66, 0x00,
                    0x6F, 0x00,
                    0x6F, 0x00,
                    0x6B, 0x71,
                    0x36, 0x21,

                    // Attributes
                    DirectoryEntryAttributes::LongName.bits(),

                    // Reserved
                    0x00,

                    // Short name checksum
                    0x12,

                    // Name stride 2
                    0xCC, 0x18,
                    0x92, 0x5F,
                    0x99, 0xB2,
                    0xB3, 0xD4,
                    0x33, 0x60,
                    0x0C, 0xC3,

                    // Reserved
                    0x00,
                    0x00,

                    // Name stride 3
                    0x00, 0x00,
                    0xFF, 0xFF,
                ],

            is_last_entry: true,
            entry_number: 1,
            short_name_checksum: 0x12,
            name_utf16_code_units: [
                0x0066, 0x006F, 0x006F, 0x716B, 0x2136, 0x18CC, 0x5F92, 0xB299, 0xD4B3, 0x6033,
                0xC30C, 0x0000, 0xFFFF,
            ],
        }
    }
}
