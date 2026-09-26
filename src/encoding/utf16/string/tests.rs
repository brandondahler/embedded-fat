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

mod from_str {
    use super::*;

    #[test]
    fn trivially_valid_input_validates_successfully() {
        let code_units = ['f' as u16, 'o' as u16, 'o' as u16];

        let result = Utf16String::from_str("foo").unwrap();

        assert_eq!(result.code_units, code_units);
    }

    #[test]
    fn valid_input_with_surrogates_validates_successfully() {
        let code_units = ['f' as u16, 0xD83D, 0xDE02, 'o' as u16];

        let result = Utf16String::from_str("f😂o").unwrap();

        assert_eq!(result.code_units, code_units);
    }

    #[test]
    fn extra_code_units_initialized_to_zero() {
        let expected_code_units = ['f' as u16, 0, 0];

        let result = Utf16String::from_str("f").unwrap();

        assert_eq!(result.code_units, expected_code_units);
    }

    #[test]
    fn null_characters_allowed() {
        let code_units = ['f' as u16, 0, 'o' as u16, 0];

        let result = Utf16String::from_str("f\0o").unwrap();

        assert_eq!(result.code_units, code_units);
    }
}

mod eq_ignore_case {
    use super::*;

    #[test]
    fn self_returns_true() {
        let value = Utf16String::<3>::from_str("f").unwrap();

        assert!(value.eq_ignore_case(&value));
    }

    #[test]
    fn same_value_returns_true() {
        let left = Utf16String::<3>::from_str("foo").unwrap();
        let right = Utf16String::<3>::from_str("foo").unwrap();

        assert!(left.eq_ignore_case(&right));
        assert!(right.eq_ignore_case(&left));
    }

    #[test]
    fn different_case_returns_true() {
        let left = Utf16String::<3>::from_str("f").unwrap();
        let right = Utf16String::<3>::from_str("F").unwrap();

        assert!(left.eq_ignore_case(&right));
        assert!(right.eq_ignore_case(&left));
    }

    #[test]
    fn non_equal_values_returns_false() {
        let left = Utf16String::<3>::from_str("foo").unwrap();
        let right = Utf16String::from_str("bar").unwrap();

        assert!(!left.eq_ignore_case(&right));
        assert!(!right.eq_ignore_case(&left));
    }

    #[test]
    fn different_codepoint_count_returns_false() {
        let left = Utf16String::<2>::from_str("f").unwrap();
        let right = Utf16String::<2>::from_str("🌷").unwrap();

        assert!(!left.eq_ignore_case(&right));
        assert!(!right.eq_ignore_case(&left));
    }
}

mod eq {
    use super::*;

    #[test]
    fn self_returns_true() {
        let value = Utf16String::<3>::from_str("f").unwrap();

        assert_eq!(value, value);
    }

    #[test]
    fn same_value_returns_true() {
        let left = Utf16String::<3>::from_str("foo").unwrap();
        let right = Utf16String::from_str("foo").unwrap();

        assert_eq!(left, right);
        assert_eq!(right, left);
    }

    #[test]
    fn empty_values_returns_true() {
        let left = Utf16String::<1>::from_str("").unwrap();
        let right = Utf16String::from_str("").unwrap();

        assert_eq!(left, right);
        assert_eq!(right, left);
    }

    #[test]
    fn non_equal_values_returns_false() {
        let left = Utf16String::<3>::from_str("foo").unwrap();
        let right = Utf16String::from_str("bar").unwrap();

        assert_ne!(left, right);
        assert_ne!(right, left);
    }

    #[test]
    fn different_case_returns_false() {
        let left = Utf16String::<3>::from_str("foo").unwrap();
        let right = Utf16String::from_str("Foo").unwrap();

        assert_ne!(left, right);
        assert_ne!(right, left);
    }

    #[test]
    fn different_codepoint_count_returns_false() {
        let left = Utf16String::<2>::from_str("f").unwrap();
        let right = Utf16String::<2>::from_str("🌷").unwrap();

        assert_ne!(left, right);
        assert_ne!(right, left);
    }
}
