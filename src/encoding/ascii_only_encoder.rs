use crate::CodePageEncoder;

#[derive(Clone, Copy, Debug, Default)]
pub struct AsciiOnlyEncoder;

impl CodePageEncoder for AsciiOnlyEncoder {
    fn encode(&self, character: char) -> Option<u8> {
        match character {
            '\0'..='\x7F' => Some(character as u8),
            _ => None,
        }
    }

    fn uppercase(&self, character: char) -> char {
        character.to_ascii_uppercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod encode {
        use super::*;

        #[test]
        fn ascii_characters_encodable() {
            let ascii_only_encoder = AsciiOnlyEncoder;

            for codepoint in 0x00..=0x7F {
                let character = char::from_u32(codepoint as u32).unwrap();
                let result = ascii_only_encoder
                    .encode(character)
                    .expect("Ok should be returned");

                assert_eq!(result, codepoint);
            }
        }

        #[test]
        fn non_ascii_characters_not_encodable() {
            let ascii_only_encoder = AsciiOnlyEncoder;
            let values = "귒攈ぷ뼧怡ꖟ珧⊤巬鉗ꏟ垉鋶寧";

            for value in values.chars() {
                let result = ascii_only_encoder.encode(value);

                assert!(result.is_none())
            }
        }
    }

    mod uppercase {
        use super::*;

        #[test]
        fn ascii_characters_uppercased() {
            let ascii_only_encoder = AsciiOnlyEncoder;

            assert_eq!(ascii_only_encoder.uppercase('a'), 'A');
        }

        #[test]
        fn non_ascii_characters_not_modified() {
            let ascii_only_encoder = AsciiOnlyEncoder;

            assert_eq!(ascii_only_encoder.uppercase('ā'), 'ā');
        }
    }
}
