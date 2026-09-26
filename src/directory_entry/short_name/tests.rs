use super::*;
use crate::AsciiOnlyEncoder;

mod from_bytes {
    use super::*;

    #[test]
    fn parses_entry_correctly() {
        let test_data = TestData::valid();

        let entry = ShortNameDirectoryEntry::from_bytes(&test_data.data).unwrap();

        assert_eq!(entry.name(), &test_data.name, "name should parse correctly");
        assert_eq!(
            entry.is_directory(),
            test_data.is_directory,
            "is_directory should be parsed correctly"
        );
        assert_eq!(
            entry.first_cluster_number(),
            test_data.first_cluster_number,
            "first_cluster_number should be parsed correctly"
        );
        assert_eq!(
            entry.file_size(),
            test_data.file_size,
            "file_size should be parsed correctly"
        );
    }

    #[test]
    fn initial_byte_05_parsed_correctly() {
        let mut data = TestData::valid().data;
        data[0] = 0x05;

        let entry = ShortNameDirectoryEntry::from_bytes(&data).unwrap();

        assert_eq!(
            entry.name().bytes()[0],
            0xE5,
            "First byte of name should be 0xE5"
        );
    }

    #[test]
    fn first_cluster_number_invalid_returns_err() {
        for first_cluster_number in [0, 1] {
            let test_data = TestData::valid()
                .with_first_cluster_number(first_cluster_number)
                .with_file_size(1);

            let error = ShortNameDirectoryEntry::from_bytes(&test_data.data).unwrap_err();

            assert_eq!(
                error,
                ShortNameDirectoryEntryError::FirstClusterNumberInvalid,
                "Expected error should be returned"
            );
        }
    }

    #[test]
    fn file_size_zero_allows_invalid_first_cluster_number() {
        for first_cluster_number in [0, 1] {
            let test_data = TestData::valid()
                .with_first_cluster_number(first_cluster_number)
                .with_file_size(0);

            let result = ShortNameDirectoryEntry::from_bytes(&test_data.data).unwrap();

            assert_eq!(result.file_size(), 0, "file_size should be 0");
        }
    }
}

mod write {
    use super::*;

    #[test]
    fn roundtrips_correctly() {
        let data = TestData::valid().data;
        let entry = ShortNameDirectoryEntry::from_bytes(&data).unwrap();

        let mut result = [0x00; DIRECTORY_ENTRY_SIZE];
        entry.write(&mut result);

        assert_eq!(result, data, "Input and output bytes should match exactly");
    }

    #[test]
    fn initial_byte_05_roundtrips_correctly() {
        let mut data = TestData::valid().data;
        data[0] = 0x05;

        let entry = ShortNameDirectoryEntry::from_bytes(&data).unwrap();

        let mut result = [0x00; DIRECTORY_ENTRY_SIZE];
        entry.write(&mut result);

        assert_eq!(result, data, "Input and output bytes should match exactly");
    }
}

struct TestData {
    data: [u8; DIRECTORY_ENTRY_SIZE],

    name: ShortFileName,
    is_directory: bool,
    first_cluster_number: u32,
    file_size: u32,
}

impl TestData {
    fn valid() -> Self {
        Self {
            #[rustfmt::skip]
            data: [
                // Name
                0x46, 0x4F, 0x4F, 0x42, 0x41, 0x52, 0x20, 0x20,
                0x54, 0x58, 0x54,

                // Attributes
                DirectoryEntryAttributes::Subdirectory.bits(),

                // Reserved
                0x00,

                // Unparsed timestamps
                0x00,
                0x00, 0x00,
                0x00, 0x00,
                0x00, 0x00,

                // First cluster high
                0x34, 0x12,

                // Unparsed timestamps
                0x00, 0x00,
                0x00, 0x00,

                // First cluster low
                0x78, 0x56,

                // File Size
                0xF1, 0xDE, 0xBC, 0x9A,
            ],

            name: ShortFileName::from_str(&AsciiOnlyEncoder, "foobar.txt").unwrap(),
            is_directory: true,
            first_cluster_number: 0x12345678,
            file_size: 0x9ABCDEF1,
        }
    }

    fn with_first_cluster_number(mut self, first_cluster_number: u32) -> Self {
        write_le_u16(&mut self.data, 20, (first_cluster_number >> 16) as u16);
        write_le_u16(&mut self.data, 26, first_cluster_number as u16);
        self.first_cluster_number = first_cluster_number;

        self
    }

    fn with_file_size(mut self, file_size: u32) -> Self {
        write_le_u32(&mut self.data, 28, file_size);
        self.file_size = file_size;

        self
    }
}
