use crate::queued_resource::{QueuedResource, QueuedResourceBusyError, QueuedResourceLease};
use crate::waker_queue::WakerQueueLease;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct QueuedResourceLeaseFuture<'a, T, const N: usize> {
    queued_resource: &'a QueuedResource<T, N>,
    waker_queue_lease: Option<WakerQueueLease>,
}

impl<'a, T, const N: usize> QueuedResourceLeaseFuture<'a, T, N> {
    pub(super) fn new(queued_resource: &'a QueuedResource<T, N>) -> Self {
        Self {
            queued_resource,
            waker_queue_lease: None,
        }
    }
}

impl<'a, T, const N: usize> Future for QueuedResourceLeaseFuture<'a, T, N> {
    type Output = Result<QueuedResourceLease<'a, T, N>, QueuedResourceBusyError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.waker_queue_lease.take() {
            // Not yet enqueued
            None => {
                let enqueue_result = self.queued_resource.enqueue_waker(cx.waker().clone());

                match enqueue_result {
                    // Our future has been assigned the QueuedResourceLease
                    Ok(None) => Poll::Ready(Ok(QueuedResourceLease::new(self.queued_resource))),

                    // Our waker was enqueued successfully
                    Ok(Some(new_waker_queue_lease)) => {
                        self.waker_queue_lease = Some(new_waker_queue_lease);
                        Poll::Pending
                    }

                    // The queue does not have any space to enqueue our waker into
                    Err(error) => Poll::Ready(Err(error)),
                }
            }

            // Already enqueued
            Some(waker_queue_lease) => {
                match self
                    .queued_resource
                    .try_update_waker(waker_queue_lease, cx.waker())
                {
                    // Still waiting
                    Some(new_waker_queue_lease) => {
                        self.waker_queue_lease = Some(new_waker_queue_lease);
                        Poll::Pending
                    }

                    // Our future has been assigned the QueuedResourceLease
                    None => Poll::Ready(Ok(QueuedResourceLease::new(self.queued_resource))),
                }
            }
        }
    }
}

impl<'a, T, const N: usize> Drop for QueuedResourceLeaseFuture<'a, T, N> {
    fn drop(&mut self) {
        if let Some(waker_queue_lease) = self.waker_queue_lease.take() {
            // Attempt to cancel the queue lease
            if !self.queued_resource.try_cancel_waker(waker_queue_lease) {
                // Cancellation failing means that our future was assigned the QueuedResourceLease;
                // however, we never got a chance to create (and therefore drop it).

                // To ensure the resource is freed up correctly, we need to create one and
                // immediately drop it.
                drop(QueuedResourceLease::new(self.queued_resource));
            }
        }
    }
}
