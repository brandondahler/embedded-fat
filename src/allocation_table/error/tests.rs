use super::*;
use crate::mock::IoError;
use alloc::string::ToString;
use core::fmt::Debug;

mod display {
    use super::*;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            AllocationTableError::StreamEndReached,
            AllocationTableError::StreamError(IoError::default()),
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
