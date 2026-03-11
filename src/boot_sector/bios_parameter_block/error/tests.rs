use super::*;
use alloc::string::ToString;
use strum::IntoEnumIterator;

mod display {
    use super::*;

    #[test]
    fn produces_non_empty_value() {
        for value in BiosParameterBlockError::iter() {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
