mod busy_error;
mod lease;
mod lease_future;

#[cfg(test)]
mod tests;

pub use busy_error::*;
pub use lease::*;
pub use lease_future::*;

use crate::waker_queue::{WakerQueue, WakerQueueLease};
use core::cell::{RefCell, UnsafeCell};
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;
use critical_section::Mutex;
use embedded_io::ErrorType;

// TODO: Consider pulling out to a dedicated library. This implementation is useful to achieve the
//   goals of this library, but it's equally valuable for other embedded use-cases where async
//   resource sharing is needed but synchronous locking isn't desirable.

/// A resource wrapper which manages the safe sharing of a provided resource across multiple
/// asynchronous consumers.
///
/// Asynchronous acquisitions are returned a future which will be blocked until the resource is
/// assigned to that acquisition's future.
///
/// Synchronous acquisitions succeed only if the resource isn't currently held. Asynchronous
/// acquisitions are still able to queue up behind a synchronous acquisition.
///
/// Consumers are returned a [QueuedResourceLease], which represents ownership of the underlying
/// resource until the lease is dropped. Once the lease is dropped, the next acquiring future will
/// be assigned the resource and woken.
///
/// Up to [N] asynchronous requests can be queued up for acquisition. Asynchronous acquisition
/// futures will be assigned the resource in a strict FIFO manner. Asynchronous acquisition can be
/// safely canceled by dropping the future; however, cancellation will not immediately release the
/// allocated slot in the queue -- it will be released when the original future would have been
/// assigned the resource.
#[derive(Debug)]
pub struct QueuedResource<T, const N: usize> {
    /// The resource whose access can be queued up asynchronously.
    ///
    /// # Safety
    /// Must only be accessed by the single [QueuedResourceLease] currently allocated for this
    /// instance.
    resource: UnsafeCell<T>,

    /// An atomic flag for whether a [QueuedResourceLease] is currently outstanding.
    resource_leased: AtomicBool,

    /// The queue of [Waker]s wrapped in a [Mutex]
    waker_queue: Mutex<RefCell<WakerQueue<N>>>,
}

/// [QueuedResource] is [Sync] if and only if [T] is [Send].
///
/// [T] must be [Send] because [QueuedResource] uses [UnsafeCell] to share [T] based on its internal
/// locking and queueing implementation.
///
/// [QueuedResource] is allowed to be [Sync] because we utilize internal locking and queueing to
/// provide exclusive access to [T]. All other fields in [QueuedResource] **MUST** also be [Sync].
unsafe impl<T, const N: usize> Sync for QueuedResource<T, N> where T: Send {}

impl<T, const N: usize> QueuedResource<T, N> {
    /// Creates a new instance which owns and manages access to the provided resource.
    pub fn new(resource: T) -> Self {
        Self {
            resource: UnsafeCell::new(resource),
            resource_leased: AtomicBool::new(false),

            waker_queue: Mutex::new(RefCell::new(WakerQueue::new())),
        }
    }

    /// Attempts to synchronously acquire the resource.
    ///
    /// If the resource is already allocated out to another consumer, a [QueuedResourceBusyError]
    /// will be returned immediately.
    pub fn try_acquire_resource(
        &self,
    ) -> Result<QueuedResourceLease<'_, T, N>, QueuedResourceBusyError> {
        if self.resource_leased.swap(true, Ordering::Acquire) {
            return Err(QueuedResourceBusyError);
        }

        Ok(QueuedResourceLease::new(self))
    }

    /// Attempts to asynchronously acquire the resource.
    ///
    /// If the resource is already allocated out to another consumer, the returned future will be
    /// enqueued for acquisition. If [N] futures have already been enqueued, the future will
    /// immediately return a [QueuedResourceBusyError].
    pub fn acquire_resource_async(
        &self,
    ) -> impl Future<Output = Result<QueuedResourceLease<'_, T, N>, QueuedResourceBusyError>> {
        QueuedResourceLeaseFuture::new(self)
    }

    /// Unsafely accesses the underlying resource
    ///
    /// # Safety
    /// This **MUST** only be accessed by the allocated [QueuedResourceLease], which should only be
    /// created if the `resource_leased` field is `true`.
    unsafe fn get_resource(&self) -> &T {
        debug_assert!(
            self.resource_leased.load(Ordering::Acquire),
            "get_resource called despite not being leased"
        );

        unsafe { &*self.resource.get() }
    }

    /// Unsafely accesses the underlying resource
    ///
    /// # Safety
    /// This **MUST** only be accessed by the allocated [QueuedResourceLease], which should only be
    /// created if the `resource_leased` field is `true`.
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_resource_mut(&self) -> &mut T {
        debug_assert!(
            self.resource_leased.load(Ordering::Acquire),
            "get_resource_mut called despite not being leased"
        );

        unsafe { &mut *self.resource.get() }
    }

    /// Enqueues the waker into the [WakerQueue] associated with this instance.
    ///
    /// If the resource is immediately available, the caller is assigned the resource and [None] is
    /// returned. If the queue is full, a [QueuedResourceBusyError] is returned. In either of these
    /// cases, the provided [Waker] will therefore never be called and the caller **MUST** return
    /// immediately to prevent being deadlocked.
    fn enqueue_waker(
        &self,
        waker: Waker,
    ) -> Result<Option<WakerQueueLease>, QueuedResourceBusyError> {
        if !self.resource_leased.swap(true, Ordering::Acquire) {
            return Ok(None);
        }

        let enqueue_result = critical_section::with(|cs| {
            let mut waker_queue = self.waker_queue.borrow_ref_mut(cs);

            waker_queue.push(waker)
        });

        match enqueue_result {
            Some(waker_queue_lease) => Ok(Some(waker_queue_lease)),
            None => Err(QueuedResourceBusyError),
        }
    }

    /// Attempts to update the [WakerQueueLease]'s slot with the provided [Waker].
    ///
    /// If the caller has been assigned the resource, [None] will be returned. If this happens, the
    /// provided [Waker] will therefore never be called and the caller **MUST** return immediately
    /// to prevent being deadlocked.
    ///
    /// If the update is successful, the originally passed [WakerQueueLease] will be returned back.
    fn try_update_waker(
        &self,
        waker_queue_lease: WakerQueueLease,
        waker: &Waker,
    ) -> Option<WakerQueueLease> {
        critical_section::with(|cs| {
            let mut waker_queue = self.waker_queue.borrow_ref_mut(cs);

            waker_queue.try_update(waker_queue_lease, waker)
        })
    }

    /// Attempts to cancel the [WakerQueueLease]'s slot.
    ///
    /// If the caller has been assigned the resource, `false` will be returned.  If this happens,
    /// the caller has already been assigned the resource and therefore the caller **MUST** allocate
    /// a [QueuedResourceLease]. The caller then **MAY** immediately drop that[QueuedResourceLease]
    /// instance.
    fn try_cancel_waker(&self, waker_queue_lease: WakerQueueLease) -> bool {
        critical_section::with(|cs| {
            let mut waker_queue = self.waker_queue.borrow_ref_mut(cs);

            waker_queue.try_cancel(waker_queue_lease)
        })
    }

    /// Releases the resource from the currently assigned consumer.
    ///
    /// The next asynchronous consumer will be immediately assigned the resource and woken.
    fn lease_dropped(&self) {
        debug_assert!(
            self.resource_leased.load(Ordering::Acquire),
            "lease_dropped called on an unleased QueuedResource"
        );

        let waker = critical_section::with(|cs| self.waker_queue.borrow_ref_mut(cs).next_waker());

        if let Some(waker) = waker {
            waker.wake();
        } else {
            let resource_was_leased = self.resource_leased.swap(false, Ordering::Release);

            debug_assert!(
                resource_was_leased,
                "lease_dropped called twice concurrently resulting in double-releasing of the resource"
            );
        }
    }
}
