use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct CoreError;

impl Display for CoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "CoreError")
    }
}

impl Error for CoreError {}

#[derive(Clone, Copy, Debug)]
pub struct IntoCoreError;

impl From<IntoCoreError> for CoreError {
    fn from(value: IntoCoreError) -> Self {
        Self
    }
}
