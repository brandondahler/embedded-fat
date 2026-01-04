use core::fmt::{Display, Formatter};
use embedded_io::{Error, ErrorKind, ReadExactError};

#[derive(Clone, Copy, Debug)]
pub enum AllocationTableError<E>
where
    E: Error,
{
    StreamError(E),
    StreamEndReached,
}

impl<E> core::error::Error for AllocationTableError<E> where E: Error {}

impl<E> Display for AllocationTableError<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AllocationTableError::StreamError(e) => Display::fmt(&e, f),
            AllocationTableError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
        }
    }
}

impl<E> Error for AllocationTableError<E>
where
    E: Error,
{
    fn kind(&self) -> ErrorKind {
        match self {
            AllocationTableError::StreamError(device_error) => device_error.kind(),
            AllocationTableError::StreamEndReached => ErrorKind::Other,
        }
    }
}

impl<E> From<E> for AllocationTableError<E>
where
    E: Error,
{
    fn from(value: E) -> Self {
        AllocationTableError::StreamError(value)
    }
}

impl<E> From<ReadExactError<E>> for AllocationTableError<E>
where
    E: Error,
{
    fn from(value: ReadExactError<E>) -> Self {
        match value {
            ReadExactError::Other(e) => e.into(),
            ReadExactError::UnexpectedEof => AllocationTableError::StreamEndReached,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Debug;

    mod display {
        use super::*;
        use alloc::string::ToString;

        #[test]
        fn display_produces_non_empty_value() {
            let values = [
                AllocationTableError::StreamEndReached,
                AllocationTableError::StreamError(MockError(ErrorKind::Other)),
            ];

            for value in values {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }

    mod error_kind {
        use super::*;

        #[test]
        fn stream_end_reached_is_other() {
            assert_eq!(
                AllocationTableError::<MockError>::StreamEndReached.kind(),
                ErrorKind::Other
            );
        }

        #[test]
        fn stream_error_inherits_value() {
            assert_eq!(
                AllocationTableError::StreamError(MockError(ErrorKind::AddrInUse)).kind(),
                ErrorKind::AddrInUse
            );
        }
    }

    #[derive(Debug)]
    struct MockError(ErrorKind);

    impl core::error::Error for MockError {}

    impl Display for MockError {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "MockError")
        }
    }

    impl Error for MockError {
        fn kind(&self) -> ErrorKind {
            self.0
        }
    }
}
