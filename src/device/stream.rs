use crate::device::Device;
use crate::device::stream_owner::DeviceStreamOwner;
use core::cell::{RefCell, RefMut, UnsafeCell};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use embedded_io::ErrorType;

pub struct DeviceStream<'a, S, const N: usize>
where
    S: ErrorType,
{
    device: &'a Device<S, N>,
}

impl<'a, S, const N: usize> DeviceStream<'a, S, N>
where
    S: ErrorType,
{
    pub fn new(device: &'a Device<S, N>) -> DeviceStream<'a, S, N> {
        Self { device }
    }
}

impl<'a, S, const N: usize> Deref for DeviceStream<'a, S, N>
where
    S: ErrorType,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        unsafe { self.device.get_stream() }
    }
}

impl<'a, S, const N: usize> DerefMut for DeviceStream<'a, S, N>
where
    S: ErrorType,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.device.get_stream_mut() }
    }
}

impl<'a, S, const N: usize> Drop for DeviceStream<'a, S, N>
where
    S: ErrorType,
{
    fn drop(&mut self) {
        self.device.device_stream_dropped();
    }
}
