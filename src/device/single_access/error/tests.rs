use super::*;
use crate::mock::IoError;
use alloc::string::ToString;

mod display {
    use super::*;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            SingleAccessDeviceError::StreamInUse,
            SingleAccessDeviceError::FlushFailed(IoError::default()),
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
