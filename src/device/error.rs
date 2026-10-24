use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum DeviceError {
    Busy,
}

impl Error for DeviceError {}

impl Display for DeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DeviceError::Busy => write!(f, "the device is currently busy"),
        }
    }
}
