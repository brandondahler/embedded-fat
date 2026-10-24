mod error;
mod stream;
mod stream_future;
mod stream_owner;
mod waker_queue;

pub use error::*;
pub use stream::*;
pub use stream_future::*;
pub use stream_owner::*;
pub use waker_queue::*;

use core::cell::{RefCell, UnsafeCell};
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;
use critical_section::Mutex;
use embedded_io::ErrorType;

/// A device which can queue up to N futures.
#[derive(Debug)]
pub struct Device<S, const N: usize>
where
    S: ErrorType,
{
    /// The stream implementation of the device
    ///
    /// # Safety
    /// Must only be accessed by the single [DeviceStream] currently allocated for this
    /// [Device].
    stream: UnsafeCell<S>,

    /// An atomic flag for whether a [DeviceStream] has been allocated.
    stream_allocated: AtomicBool,

    /// The queue of wakers wrapped in a [Mutex]
    waker_queue: Mutex<RefCell<DeviceWakerQueue<N>>>,
}

unsafe impl<S, const N: usize> Sync for Device<S, N> where S: ErrorType + Send {}

impl<S, const N: usize> Device<S, N>
where
    S: ErrorType,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream: UnsafeCell::new(stream),
            stream_allocated: AtomicBool::new(false),
            waker_queue: Mutex::new(RefCell::new(DeviceWakerQueue::new())),
        }
    }

    pub fn acquire_stream(&self) -> Result<DeviceStream<'_, S, N>, DeviceError> {
        if self.stream_allocated.swap(true, Ordering::Acquire) {
            return Err(DeviceError::Busy);
        }

        Ok(DeviceStream::new(self))
    }

    pub fn acquire_stream_async(&self) -> DeviceStreamFuture<'_, S, N> {
        DeviceStreamFuture::new(self)
    }

    unsafe fn get_stream(&self) -> &S {
        unsafe { &*self.stream.get() }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn get_stream_mut(&self) -> &mut S {
        unsafe { &mut *self.stream.get() }
    }

    fn device_stream_dropped(&self) {
        assert!(
            self.stream_allocated.load(Ordering::Acquire),
            "device_stream_dropped called on a non-busy Device"
        );

        let waker = critical_section::with(|cs| self.waker_queue.borrow_ref_mut(cs).pop());

        if let Some(waker) = waker {
            waker.wake();
        } else {
            assert!(
                self.stream_allocated.swap(false, Ordering::Release),
                "device_stream_dropped called twice concurrently"
            );
        }
    }

    fn enqueue_waker(&self, waker: Waker) -> Result<Option<DeviceWakerQueueLease>, DeviceError> {
        if !self.stream_allocated.swap(true, Ordering::Acquire) {
            return Ok(None);
        }

        let enqueue_result = critical_section::with(|cs| {
            let mut waker_queue = self.waker_queue.borrow_ref_mut(cs);

            waker_queue.push(waker)
        });

        match enqueue_result {
            Some(queue_lease) => Ok(Some(queue_lease)),
            None => Err(DeviceError::Busy),
        }
    }

    fn update_waker(&self, lease: &DeviceWakerQueueLease, waker: Waker) -> bool {
        critical_section::with(|cs| {
            let mut waker_queue = self.waker_queue.borrow_ref_mut(cs);

            waker_queue.try_update(lease, waker)
        })
    }

    fn cancel_waker(&self, lease: DeviceWakerQueueLease) -> bool {
        critical_section::with(|cs| {
            let mut waker_queue = self.waker_queue.borrow_ref_mut(cs);

            waker_queue.try_cancel(lease)
        })
    }
}
