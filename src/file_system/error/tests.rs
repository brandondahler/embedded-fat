use super::*;
use alloc::string::ToString;

mod display {
    use super::*;
    use crate::mock::IoError;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            FileSystemError::DeviceError(IoError::default()),
            FileSystemError::InvalidFatSignature,
            FileSystemError::InvalidBiosParameterBlock(
                BiosParameterBlockError::AllocationTableCountInvalid,
            ),
            FileSystemError::StreamEndReached,
            FileSystemError::StreamError(IoError::default()),
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
