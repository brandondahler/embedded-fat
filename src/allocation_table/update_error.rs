#[cfg(test)]
mod tests;

use crate::allocation_table::{PhysicalAllocationTableEntry, PhysicalAllocationTableEntryError};
use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_io::ReadExactError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AllocationTableUpdateError<E>
where
    E: embedded_io::Error,
{
    EntryInvalid(PhysicalAllocationTableEntryError),
    StreamError(E),
    StreamEndReached,
}

impl<E> Error for AllocationTableUpdateError<E> where E: embedded_io::Error {}

impl<E> Display for AllocationTableUpdateError<E>
where
    E: embedded_io::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AllocationTableUpdateError::EntryInvalid(e) => {
                write!(f, "entry invalid: {e}")
            }
            AllocationTableUpdateError::StreamEndReached => {
                write!(f, "stream end was reached when not expected")
            }
            AllocationTableUpdateError::StreamError(e) => write!(f, "stream error: {e}"),
        }
    }
}

impl<E> From<E> for AllocationTableUpdateError<E>
where
    E: embedded_io::Error,
{
    fn from(value: E) -> Self {
        AllocationTableUpdateError::StreamError(value)
    }
}

impl<E> From<PhysicalAllocationTableEntryError> for AllocationTableUpdateError<E>
where
    E: embedded_io::Error,
{
    fn from(value: PhysicalAllocationTableEntryError) -> Self {
        AllocationTableUpdateError::EntryInvalid(value)
    }
}

impl<E> From<ReadExactError<E>> for AllocationTableUpdateError<E>
where
    E: embedded_io::Error,
{
    fn from(value: ReadExactError<E>) -> Self {
        match value {
            ReadExactError::Other(e) => e.into(),
            ReadExactError::UnexpectedEof => AllocationTableUpdateError::StreamEndReached,
        }
    }
}
