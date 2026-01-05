use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct CharacterEncodingError(pub char);

impl Display for CharacterEncodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "character `{}` (\\u{:08X}) does not have a valid encoding",
            self.0, self.0 as u32
        )
    }
}

impl Error for CharacterEncodingError {}
