use super::*;
use alloc::string::ToString;

mod display {
    use super::*;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            ShortFileNameParseError::CharacterNotAllowed {
                character: 'A',
                offset: 0,
            },
            ShortFileNameParseError::CharacterNotEncodable {
                character: 'A',
                offset: 0,
            },
            ShortFileNameParseError::EncodedCharacterByteNotAllowed {
                character: 'A',
                encoded_character: b'A',
                offset: 0,
            },
            ShortFileNameParseError::ExtensionTooLong,
            ShortFileNameParseError::InputEmpty,
            ShortFileNameParseError::NameEmpty,
            ShortFileNameParseError::NameStartsWithSpace,
            ShortFileNameParseError::NameTooLong,
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
