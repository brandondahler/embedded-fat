use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::ReadExactError;

#[derive(Clone, Debug)]
pub enum AllocationTableError<E>
where
    E: embedded_io::Error,
{
    StreamError(E),
    StreamEndReached,
}

impl<E> Error for AllocationTableError<E> where E: embedded_io::Error {}

impl<E> Display for AllocationTableError<E>
where
    E: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AllocationTableError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            AllocationTableError::StreamError(e) => Display::fmt(&e, f),
        }
    }
}

impl<E> From<E> for AllocationTableError<E>
where
    E: embedded_io::Error,
{
    fn from(value: E) -> Self {
        AllocationTableError::StreamError(value)
    }
}

impl<E> From<ReadExactError<E>> for AllocationTableError<E>
where
    E: embedded_io::Error,
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
}
