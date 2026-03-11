use super::*;
use crate::mock::CoreError;
use crate::mock::IoError;
use alloc::string::ToString;
use embedded_io::Error;
use strum::IntoEnumIterator;

mod display {
    use super::*;
    use crate::mock::IoError;

    #[test]
    fn produces_non_empty_value() {
        let values = [
            FileError::DeviceError(IoError::default()),
            FileError::SeekPositionBeyondLimits(0),
            FileError::SeekPositionImpossible(0),
            FileError::StreamEndReached,
            FileError::StreamError(IoError::default()),
            FileError::UnexpectedAllocationTableEntryEncountered,
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }
}

mod kind {
    use super::*;

    #[test]
    fn stream_error_delegates_to_contained_error() {
        assert_eq!(
            FileError::<CoreError, IoError>::StreamError(IoError(ErrorKind::AddrInUse)).kind(),
            ErrorKind::AddrInUse
        );
    }

    #[test]
    fn non_stream_error_returns_other() {
        assert_eq!(
            FileError::<CoreError, IoError>::DeviceError(CoreError).kind(),
            ErrorKind::Other
        );
    }
}
