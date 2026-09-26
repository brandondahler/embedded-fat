#[cfg(test)]
mod tests;

use core::error::Error;
use core::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhysicalAllocationTableEntryError {
    ValueInvalid(u32),
}

impl Display for PhysicalAllocationTableEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ValueInvalid(value) => {
                write!(
                    f,
                    "value is invalid for configured allocation table kind: {value}"
                )
            }
        }
    }
}

impl Error for PhysicalAllocationTableEntryError {}
