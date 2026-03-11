use super::*;
use core::array::from_fn;

mod from_str {
    use super::*;

    #[test]
    fn basic_input_parsed_correctly() {
        let mut expected_characters = [0; LONG_NAME_MAX_LENGTH];
        expected_characters[0] = 'f' as u16;
        expected_characters[1] = 'o' as u16;
        expected_characters[2] = 'o' as u16;

        let long_file_name = LongFileName::from_str("foo").expect("Name should parse successfully");

        assert_eq!(
            long_file_name.utf16_string,
            Utf16String::new(expected_characters).unwrap(),
            "Characters should match expected result"
        );
    }

    #[test]
    fn empty_input_returns_error() {
        let result = LongFileName::from_str("").expect_err("Err should be returned");

        assert!(
            matches!(result, LongFileNameStringError::InputEmpty),
            "Returned error should be InputEmpty"
        );
    }

    #[test]
    fn too_long_input_returns_error() {
        let result = LongFileName::from_str(&"a".repeat(256)).expect_err("Err should be returned");

        assert_eq!(
            result,
            LongFileNameStringError::InputTooLong,
            "Returned error should be InputTooLong"
        );
    }

    #[test]
    fn invalid_filename_character_returns_error() {
        let invalid_characters = "\
                \x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\
                \x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\
                \"*/:<>?\\|\u{FFFF}";

        for invalid_character in invalid_characters.chars() {
            let mut invalid_character_buffer = [0; 4];
            let input = invalid_character.encode_utf8(&mut invalid_character_buffer);

            let result = LongFileName::from_str(&input).expect_err("Err should be returned");

            assert!(
                matches!(
                    result,
                    LongFileNameStringError::CharacterInvalid {
                        character: invalid_character,
                        offset: 0
                    }
                ),
                "Returned error should be CharacterInvalid(0x{:04X})",
                invalid_character as u16
            );
        }
    }
}

mod eq {
    use super::*;

    #[test]
    fn same_case_returns_true() {
        let name_1 = LongFileName::from_str("foobar").expect("Provided string should be valid");

        assert_eq!(name_1, name_1, "Values should be equal");
    }

    #[test]
    fn max_length_returns_true() {
        let name_1 = LongFileName::from_str(&"a".repeat(LONG_NAME_MAX_LENGTH))
            .expect("Provided string should be valid");

        assert_eq!(name_1, name_1, "Values should be equal");
    }

    #[test]
    fn different_case_returns_true() {
        let name_1 = LongFileName::from_str("fooBar").expect("Provided string should be valid");
        let name_2 = LongFileName::from_str("fOobAr").expect("Provided string should be valid");

        assert_eq!(name_1, name_2, "Values should be equal");
        assert_eq!(name_2, name_1, "Values should be equal");
    }

    #[cfg(feature = "unicode-case-folding")]
    #[test]
    fn different_case_simple_folding_returns_true() {
        // Both values are folded to ß when using the simple case folding mapping
        let name_1 = LongFileName::from_str("ß").expect("Provided string should be valid");
        let name_2 = LongFileName::from_str("ẞ").expect("Provided string should be valid");

        assert_eq!(name_1, name_2, "Values should be equal");
        assert_eq!(name_2, name_1, "Values should be equal");
    }

    #[test]
    fn different_values_returns_false() {
        let name_1 = LongFileName::from_str("a").expect("Provided string should be valid");
        let name_2 = LongFileName::from_str("b").expect("Provided string should be valid");

        assert_ne!(name_1, name_2, "Values should not be equal");
        assert_ne!(name_2, name_1, "Values should not be equal");
    }

    #[test]
    fn different_lengths_returns_false() {
        let name_1 = LongFileName::from_str("foo").expect("Provided string should be valid");
        let name_2 = LongFileName::from_str("foobar").expect("Provided string should be valid");

        assert_ne!(name_1, name_2, "Values should not be equal");
        assert_ne!(name_2, name_1, "Values should not be equal");
    }

    #[test]
    fn different_complex_full_folding_returns_false() {
        // ẞ would be folded to SS when using the full case folding mapping
        let name_1 = LongFileName::from_str("ẞ").expect("Provided string should be valid");
        let name_2 = LongFileName::from_str("SS").expect("Provided string should be valid");

        assert_ne!(name_1, name_2, "Values should not be equal");
        assert_ne!(name_2, name_1, "Values should not be equal");
    }
}
