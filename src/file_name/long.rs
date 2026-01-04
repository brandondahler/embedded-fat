use crate::CharacterEncodingError;
use crate::encoding::Ucs2Character;

pub const LONG_NAME_MAX_LENGTH: usize = 255;

#[derive(Debug, Eq)]
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
        let mut index = 0;

        for character in name.chars() {
            ensure!(
                index < LONG_NAME_MAX_LENGTH,
                LongFileNameError::InputTooLong
            );
            ensure!(
                Self::is_valid_character(character),
                LongFileNameError::CharacterInvalid(character)
            );

            ucs2_characters[index] = character.try_into()?;
            index += 1;
        }

        if index < 256 {
            ucs2_characters[index] = Ucs2Character::null();
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
        let mut left_chars = self
            .ucs2_characters
            .iter()
            .flat_map(|ucs2_character| ucs2_character.to_char().to_uppercase());

        let mut right_chars = other
            .ucs2_characters
            .iter()
            .flat_map(|ucs2_character| ucs2_character.to_char().to_uppercase());

        loop {
            let left_char = left_chars.next();
            let right_char = right_chars.next();

            match (left_char, right_char) {
                (Some(l), Some(r)) => {
                    if l != r {
                        return false;
                    }

                    if l == '\0' {
                        return true;
                    }
                }
                _ => return false,
                (None, None) => return true,
            }
        }
    }
}

impl From<[Ucs2Character; LONG_NAME_MAX_LENGTH]> for LongFileName {
    fn from(value: [Ucs2Character; LONG_NAME_MAX_LENGTH]) -> Self {
        Self::new(value)
    }
}

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
