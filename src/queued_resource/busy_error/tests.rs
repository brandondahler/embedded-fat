use super::*;
use alloc::string::ToString;

mod display {
    use super::*;

    #[test]
    fn produces_non_empty_value() {
        assert!(
            !QueuedResourceBusyError.to_string().is_empty(),
            "Display implementation should be non-empty"
        );
    }
}
