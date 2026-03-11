use super::*;

mod new {
    use super::*;

    #[test]
    fn trivially_valid_input_validates_successfully() {
        let code_units = ['f' as u16, 'o' as u16, 'o' as u16];

        let result = Utf16String::new(code_units).unwrap();

        assert_eq!(result.code_units, code_units);
    }

    #[test]
    fn valid_input_with_surrogates_validates_successfully() {
        let code_units = ['f' as u16, 0xD83D, 0xDE02];

        let result = Utf16String::new(code_units).unwrap();

        assert_eq!(result.code_units, code_units);
    }

    #[test]
    fn unpaired_high_surrogate_in_middle_returns_err() {
        let code_units = ['f' as u16, 0xD83D, 'o' as u16];

        let result = Utf16String::new(code_units).unwrap_err();

        assert_eq!(
            result,
            Utf16StringError::UnpairedSurrogateEncountered { index: 1 }
        );
    }

    #[test]
    fn unpaired_high_surrogate_at_end_returns_err() {
        let code_units = ['f' as u16, 'o' as u16, 0xD83D];

        let result = Utf16String::new(code_units).unwrap_err();

        assert_eq!(
            result,
            Utf16StringError::UnpairedSurrogateEncountered { index: 2 }
        );
    }

    #[test]
    fn unpaired_high_surrogate_before_null_returns_err() {
        let code_units = ['f' as u16, 'o' as u16, 0xD83D, 0];

        let result = Utf16String::new(code_units).unwrap_err();

        assert_eq!(
            result,
            Utf16StringError::UnpairedSurrogateEncountered { index: 2 }
        );
    }

    #[test]
    fn unpaired_low_surrogate_returns_err() {
        let code_units = ['f' as u16, 0xDE02, 'o' as u16];

        let result = Utf16String::new(code_units).unwrap_err();

        assert_eq!(
            result,
            Utf16StringError::UnpairedSurrogateEncountered { index: 1 }
        );
    }
}
