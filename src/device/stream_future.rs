use crate::device::Device;
use crate::device::error::DeviceError;
use crate::device::stream::DeviceStream;
use crate::device::stream_owner::DeviceStreamOwner;
use crate::device::waker_queue::DeviceWakerQueueLease;
use core::cell::{RefCell, RefMut};
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use embedded_io::ErrorType;

pub struct DeviceStreamFuture<'a, S, const N: usize>
where
    S: ErrorType,
{
    device: &'a Device<S, N>,

    queue_lease: Option<DeviceWakerQueueLease>,
}

impl<'a, S, const N: usize> DeviceStreamFuture<'a, S, N>
where
    S: ErrorType,
{
    pub fn new(device: &'a Device<S, N>) -> Self {
        Self {
            device,
            queue_lease: None,
        }
    }
}

impl<'a, S, const N: usize> Future for DeviceStreamFuture<'a, S, N>
where
    S: ErrorType,
{
    type Output = Result<DeviceStream<'a, S, N>, DeviceError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &self.queue_lease {
            // Not yet enqueued
            None => {
                let enqueue_result = self.device.enqueue_waker(cx.waker().clone());

                match enqueue_result {
                    Ok(None) => Poll::Ready(Ok(DeviceStream::new(self.device))),
                    Ok(Some(queue_lease)) => {
                        self.queue_lease = Some(queue_lease);
                        Poll::Pending
                    }
                    Err(device_error) => Poll::Ready(Err(device_error)),
                }
            }

            // Already enqueued
            Some(queue_lease) => {
                if !self.device.update_waker(queue_lease, cx.waker().clone()) {
                    // Our future has been assigned the DeviceStream.

                    // Ensure DeviceStream is only created/dropped once
                    self.queue_lease = None;

                    Poll::Ready(Ok(DeviceStream::new(self.device)))
                } else {
                    // Still waiting
                    Poll::Pending
                }
            }
        }
    }
}

impl<'a, S, const N: usize> Drop for DeviceStreamFuture<'a, S, N>
where
    S: ErrorType,
{
    fn drop(&mut self) {
        if let Some(queue_lease) = self.queue_lease.take() {
            // Attempt to cancel the queue lease
            if !self.device.cancel_waker(queue_lease) {
                // Cancellation failing means that our future was assigned the DeviceStream;
                // however, we never got a chance to create (and therefore drop it).

                // To ensure the device is freed up correctly, we need to create one and immediately
                // drop it.
                drop(DeviceStream::new(self.device));
            }
        }
    }
}
