#[cfg(test)]
mod tests;

use core::error::Error;
use core::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct QueuedResourceBusyError;

impl Display for QueuedResourceBusyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "The queued resource is busy")
    }
}

impl Error for QueuedResourceBusyError {}
