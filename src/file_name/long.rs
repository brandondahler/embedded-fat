use crate::CharacterEncodingError;
use crate::encoding::Ucs2Character;

pub const LONG_NAME_MAX_LENGTH: usize = 255;

#[derive(Eq)]
#[cfg_attr(test, derive(Debug))]
pub struct LongFileName {
    ucs2_characters: [Ucs2Character; LONG_NAME_MAX_LENGTH],
}

impl LongFileName {
    pub fn new(ucs2_characters: [Ucs2Character; LONG_NAME_MAX_LENGTH]) -> Self {
        LongFileName { ucs2_characters }
    }

    pub fn from_str(name: &str) -> Result<LongFileName, LongFileNameError> {
        ensure!(!name.is_empty(), LongFileNameError::InputEmpty);

        let mut ucs2_characters = [Ucs2Character::null(); LONG_NAME_MAX_LENGTH];
        for (character_index, character) in name.chars().enumerate() {
            ensure!(
                character_index < LONG_NAME_MAX_LENGTH,
                LongFileNameError::InputTooLong
            );
            ensure!(
                Self::is_valid_character(character),
                LongFileNameError::CharacterInvalid(character)
            );

            ucs2_characters[character_index] = character.try_into()?;
        }

        Ok(Self::new(ucs2_characters))
    }

    pub fn is_empty(&self) -> bool {
        self.ucs2_characters[0] == Ucs2Character::null()
    }

    fn is_valid_character(character: char) -> bool {
        !matches!(
            character,
            '\0'..='\x1F' | '"' | '*' | '/' | ':' | '<' | '>' | '?' | '\\' | '|' | '\u{FFFF}'
        )
    }
}

impl PartialEq for LongFileName {
    fn eq(&self, other: &Self) -> bool {
        let mut left_chars = self.ucs2_characters.iter();
        let mut right_chars = other.ucs2_characters.iter();

        loop {
            let left_char = left_chars.next();
            let right_char = right_chars.next();

            match (left_char, right_char) {
                (Some(_), None) | (None, Some(_)) => return false,
                (None, None) => return true,
                (Some(l), Some(r)) => {
                    if !l.eq_ignore_case(r) {
                        return false;
                    }

                    if *l == Ucs2Character::null() {
                        return true;
                    }
                }
            }
        }
    }
}

impl From<[Ucs2Character; LONG_NAME_MAX_LENGTH]> for LongFileName {
    fn from(value: [Ucs2Character; LONG_NAME_MAX_LENGTH]) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LongFileNameError {
    CharacterInvalid(char),
    EncoderError(CharacterEncodingError),
    InputEmpty,
    InputTooLong,
}

impl From<CharacterEncodingError> for LongFileNameError {
    fn from(value: CharacterEncodingError) -> Self {
        LongFileNameError::EncoderError(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_str {
        use super::*;

        #[test]
        fn basic_input_parsed_correctly() {
            let mut expected_characters = [Ucs2Character::null(); LONG_NAME_MAX_LENGTH];
            expected_characters[0] = 'f'.try_into().unwrap();
            expected_characters[1] = 'o'.try_into().unwrap();
            expected_characters[2] = 'o'.try_into().unwrap();

            let long_file_name =
                LongFileName::from_str("foo").expect("Name should parse successfully");

            assert_eq!(
                long_file_name.ucs2_characters, expected_characters,
                "Characters should match expected result"
            );
        }

        #[test]
        fn advanced_input_parsed_successfully() {
            // NOTE: Test input is the maximum number of random but valid Unicode characters in the
            //   \u{0000} to \u{FFFF} range.
            let long_file_name =
                LongFileName::from_str("\u{2B00}\u{152C}\u{A657}\u{F4BA}\u{5CE1}\u{5176}\u{98AD}\u{3007}\u{9ABA}\u{3D0B}\u{7FD8}\u{25AE}\u{7D16}\u{D6ED}\u{185A}\u{01CC}\u{8D73}\u{7B87}\u{6278}\u{BD97}\u{8B1C}\u{FCE6}\u{1DC1}\u{EA56}\u{8069}\u{27FE}\u{9A4A}\u{F822}\u{2020}\u{214C}\u{4D36}\u{7B98}\u{EFD4}\u{0056}\u{B0E7}\u{D183}\u{4CD2}\u{EC8B}\u{6C1D}\u{0B5A}\u{E207}\u{B171}\u{FC7B}\u{0627}\u{54B1}\u{CFEC}\u{CC93}\u{180E}\u{688E}\u{94EA}\u{45BD}\u{F01E}\u{627C}\u{46A4}\u{FC42}\u{4ADA}\u{87B8}\u{971D}\u{8F04}\u{5F15}\u{B66A}\u{730A}\u{0DEC}\u{7D83}\u{ABB6}\u{D749}\u{775E}\u{D307}\u{FD0F}\u{1542}\u{9277}\u{5F4A}\u{2B53}\u{1DE2}\u{3BD1}\u{0265}\u{B1E2}\u{F270}\u{2E75}\u{822E}\u{9DC8}\u{FF27}\u{E054}\u{EBFF}\u{EF5E}\u{C17A}\u{F321}\u{9B3A}\u{E26E}\u{7807}\u{E017}\u{C91A}\u{5640}\u{9352}\u{3494}\u{A645}\u{0258}\u{5CE0}\u{412E}\u{B5ED}\u{6CCE}\u{375E}\u{9F9E}\u{572F}\u{34B4}\u{2C2E}\u{4518}\u{6477}\u{804E}\u{1C61}\u{BA79}\u{3048}\u{E0F7}\u{0EA8}\u{B2CD}\u{5300}\u{68BB}\u{C34D}\u{458B}\u{8D84}\u{C18A}\u{221D}\u{F6FB}\u{2192}\u{3724}\u{4209}\u{7E2D}\u{4A78}\u{CD93}\u{F733}\u{3001}\u{25DE}\u{860C}\u{9346}\u{8BEF}\u{80C0}\u{BFFA}\u{BBD6}\u{5A56}\u{1B56}\u{7784}\u{D3D0}\u{0E46}\u{9C11}\u{920B}\u{B7D5}\u{4AD9}\u{83C9}\u{742D}\u{1F58}\u{4DA1}\u{8630}\u{B984}\u{221F}\u{5802}\u{F3FA}\u{1420}\u{44AC}\u{951A}\u{1627}\u{27DD}\u{5E80}\u{FD87}\u{2706}\u{2579}\u{3F3A}\u{2B4A}\u{BDDB}\u{D7E7}\u{17D9}\u{1530}\u{90BC}\u{9838}\u{9874}\u{9E1A}\u{376F}\u{E43C}\u{3B81}\u{9A67}\u{CB2A}\u{7849}\u{3270}\u{7B73}\u{1AB7}\u{61D7}\u{68E1}\u{E922}\u{B422}\u{F178}\u{33FC}\u{3400}\u{23EE}\u{CCE0}\u{F2CD}\u{B967}\u{3328}\u{66CF}\u{13E3}\u{C20F}\u{1FB7}\u{D371}\u{2068}\u{5D7A}\u{2D65}\u{1A3F}\u{F12A}\u{C4D1}\u{D025}\u{6CB4}\u{1CAA}\u{FD7E}\u{3A95}\u{3544}\u{3589}\u{FF9F}\u{274A}\u{EF4D}\u{F182}\u{A386}\u{89FD}\u{47D4}\u{9D2E}\u{24F5}\u{7ACF}\u{8D8A}\u{EB6C}\u{7441}\u{B9F8}\u{0378}\u{E34F}\u{A038}\u{E6E6}\u{F1DF}\u{403A}\u{8A96}\u{9745}\u{8CA8}\u{EAE1}\u{A808}\u{20B0}\u{77E6}\u{0CA7}\u{61EC}\u{5416}\u{F5B6}\u{3E9E}\u{F63D}\u{ED25}\u{4304}\u{3485}\u{5CAB}\u{A9D5}\u{FDC9}\u{3BA6}\u{32DA}")
                    .expect("Name should parse successfully");

            for (character_index, character) in long_file_name.ucs2_characters.iter().enumerate() {
                assert_ne!(
                    *character,
                    Ucs2Character::null(),
                    "Character {character_index} should not be null"
                );
            }
        }

        #[test]
        fn empty_input_returns_error() {
            let result = LongFileName::from_str("").expect_err("Err should be returned");

            assert!(
                matches!(result, LongFileNameError::InputEmpty),
                "Returned error should be InputEmpty"
            );
        }

        #[test]
        fn too_long_input_returns_error() {
            let result =
                LongFileName::from_str(&"a".repeat(256)).expect_err("Err should be returned");

            assert!(
                matches!(result, LongFileNameError::InputTooLong),
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
                        LongFileNameError::CharacterInvalid(c) if c == invalid_character
                    ),
                    "Returned error should be CharacterInvalid(0x{:04X})",
                    invalid_character as u16
                );
            }
        }

        #[test]
        fn invalid_encoding_character_returns_error() {
            let input = "\u{10000}";

            let result = LongFileName::from_str(&input).expect_err("Err should be returned");

            assert!(
                matches!(
                    result,
                    LongFileNameError::EncoderError(CharacterEncodingError(c)) if c == '\u{10000}'
                ),
                "Returned error should be CharacterInvalid(\\u{{10000}})",
            );
        }
    }

    mod is_empty {
        use super::*;

        #[test]
        fn empty_input_returns_true() {
            let long_file_name = LongFileName::new([Ucs2Character::null(); LONG_NAME_MAX_LENGTH]);

            assert!(
                long_file_name.is_empty(),
                "Value should be considered empty"
            );
        }

        #[test]
        fn non_empty_input_returns_false() {
            let long_file_name =
                LongFileName::from_str("a").expect("Provided string should be valid");

            assert!(
                !long_file_name.is_empty(),
                "Value should not be considered empty"
            );
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

    mod from_ucs2_characters {
        use super::*;
        use core::array::from_fn;

        #[test]
        fn creates_instance_with_values() {
            let input: [Ucs2Character; LONG_NAME_MAX_LENGTH] = from_fn(|index| {
                Ucs2Character::from_u16((index as u16) + 1).expect("Value should be valid")
            });
            let long_file_name: LongFileName = input.into();

            assert_eq!(
                long_file_name.ucs2_characters, input,
                "Instance should contain same input characters"
            );
        }
    }
}
