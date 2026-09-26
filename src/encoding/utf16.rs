mod chars;
mod decode_error;
mod encode_error;
mod string;

pub use chars::*;
pub use decode_error::*;
pub use encode_error::*;
pub use string::*;

pub type Utf16CodeUnit = u16;

pub fn utf16_encode(
    output: &mut [Utf16CodeUnit],
    character: char,
) -> Result<usize, Utf16EncodeError> {
    let codepoint = character as u32;

    if codepoint <= 0xFFFF {
        ensure!(!output.is_empty(), Utf16EncodeError::OutputTooSmall);

        output[0] = codepoint as u16;
        Ok(1)
    } else {
        ensure!(output.len() >= 2, Utf16EncodeError::OutputTooSmall);

        let temp_value = codepoint - 0x01_0000;

        output[0] = 0xD800 | ((temp_value >> 10) as u16 & 0x3FF);
        output[1] = 0xDC00 | (temp_value as u16 & 0x3FF);

        Ok(2)
    }
}

pub fn utf16_decode<T>(input: &mut T) -> Result<Option<char>, Utf16DecodeError>
where
    T: Iterator<Item = Utf16CodeUnit>,
{
    let code_unit = match input.next() {
        Some(code_unit) => code_unit,
        None => return Ok(None),
    };

    if !matches!(code_unit, 0xD800..=0xDBFF) {
        let character =
            char::from_u32(code_unit as u32).ok_or(Utf16DecodeError::LowSurrogateUnexpected)?;

        return Ok(Some(character));
    }

    let low_surrogate_code_unit = input
        .next()
        .ok_or(Utf16DecodeError::HighSurrogateOrphaned)?;

    ensure!(
        matches!(low_surrogate_code_unit, 0xDC00..=0xDFFF),
        Utf16DecodeError::LowSurrogateMissing
    );

    let high_surrogate = code_unit & 0x3FF;
    let low_surrogate = low_surrogate_code_unit & 0x3FF;

    // SAFETY: Mathematically impossible to be invalid
    let character = unsafe {
        char::from_u32_unchecked(
            0x01_0000 + (((high_surrogate as u32) << 10) | low_surrogate as u32),
        )
    };

    Ok(Some(character))
}
