use super::*;
use crate::encoding::utf16_encode;

mod iterator_next {
    use super::*;

    #[test]
    fn single_code_unit_character_reads_successfully() {
        let mut code_units = [0];
        utf16_encode(&mut code_units, 'a');

        let mut utf16_codepoints = Utf16Chars::new(code_units.iter().copied());

        let result1 = utf16_codepoints.next();
        assert_eq!(result1, Some('a'));

        let result2 = utf16_codepoints.next();
        assert_eq!(result2, None);
    }

    #[test]
    fn multi_code_unit_character_reads_successfully() {
        let mut code_units = [0; 2];
        utf16_encode(&mut code_units, '\u{01_0000}');

        let mut utf16_codepoints = Utf16Chars::new(code_units.iter().copied());

        let result1 = utf16_codepoints.next();
        assert_eq!(result1, Some('\u{01_0000}'));

        let result2 = utf16_codepoints.next();
        assert_eq!(result2, None);
    }
}
