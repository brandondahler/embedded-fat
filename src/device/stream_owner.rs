use crate::device::Device;
use crate::device::waker_queue::DeviceWakerQueueLease;
use core::cell::{BorrowMutError, RefCell, RefMut};
use core::task::Waker;
use embedded_io::ErrorType;

pub(crate) trait DeviceStreamOwner<S>
where
    S: ErrorType,
{
    /// Gets an immutable reference to the stream for the single [DeviceStream] instance allocated
    /// for the associated [Device].
    unsafe fn get_stream(&self) -> &S;

    /// Gets a mutable reference to the stream for the single [DeviceStream] instance allocated
    /// for the associated [Device].
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_stream_mut(&self) -> &mut S;

    /// Notifies the device that the [DeviceStream] has been dropped and triggers the next queued
    /// waker to acquire the stream, if applicable.
    fn device_stream_dropped(&self);

    /// Attempts to enqueue the provided waker into the waker queue
    fn enqueue_waker(&self, waker: Waker) -> Option<DeviceWakerQueueLease>;

    /// Attempts to update the waker with the provided lease.
    ///
    /// Returns true if the replacement was a success; otherwise, it has been assigned the
    /// DeviceStream and therefore is not allowed to update its waker.
    fn update_waker(&self, lease: &DeviceWakerQueueLease, waker: Waker) -> bool;

    /// Requests the cancellation for the provided lease.
    ///
    /// Returns true if the replacement was a success; otherwise, it has been assigned the
    /// DeviceStream and therefore is not allowed to be canceled.
    fn cancel_waker(&self, lease: DeviceWakerQueueLease) -> bool;
}
