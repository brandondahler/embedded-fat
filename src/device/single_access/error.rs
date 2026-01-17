use core::cell::BorrowMutError;
use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind};

#[derive(Clone, Debug)]
pub enum SingleAccessDeviceError<E>
where
    E: Error,
{
    /// The stream is already in use by another process
    StreamInUse,

    /// Attempting to flush the underlying stream failed
    FlushFailed(E),
}

impl<E> core::error::Error for SingleAccessDeviceError<E> where E: Error {}

impl<E> Display for SingleAccessDeviceError<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SingleAccessDeviceError::StreamInUse => {
                write!(f, "some other process is already using the device's stream")
            }
            SingleAccessDeviceError::FlushFailed(e) => write!(
                f,
                "an error occurred while flushing the underlying stream: {}",
                e
            ),
        }
    }
}

impl<E> Error for SingleAccessDeviceError<E>
where
    E: Error,
{
    fn kind(&self) -> ErrorKind {
        match self {
            SingleAccessDeviceError::StreamInUse => ErrorKind::Other,
            SingleAccessDeviceError::FlushFailed(e) => e.kind(),
        }
    }
}

impl<E> From<BorrowMutError> for SingleAccessDeviceError<E>
where
    E: Error,
{
    fn from(value: BorrowMutError) -> Self {
        Self::StreamInUse
    }
}

#[cfg(test)]
mod tests {
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

    mod kind {
        use super::*;

        #[test]
        fn stream_in_use_is_other() {
            assert_eq!(
                SingleAccessDeviceError::<IoError>::StreamInUse.kind(),
                ErrorKind::Other
            );
        }

        #[test]
        fn flush_failed_inherits_value() {
            assert_eq!(
                SingleAccessDeviceError::<IoError>::FlushFailed(IoError(ErrorKind::AddrInUse))
                    .kind(),
                ErrorKind::AddrInUse
            );
        }
    }
}
