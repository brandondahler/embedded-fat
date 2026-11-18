use super::*;
use alloc::string::ToString;

mod display {
    use super::*;
    use crate::mock::IoError;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            FilesystemError::FilesystemBusy,
            FilesystemError::InvalidFatSignature,
            FilesystemError::InvalidBiosParameterBlock(
                BiosParameterBlockError::AllocationTableCountInvalid,
            ),
            FilesystemError::StreamEndReached,
            FilesystemError::StreamError(IoError::default()),
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}
