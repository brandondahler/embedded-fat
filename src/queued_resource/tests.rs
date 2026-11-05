use super::*;
use core::assert_matches;
use core::sync::atomic::AtomicU32;
use core::time::Duration;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

mod try_acquire_resource {
    use super::*;

    #[test]
    fn resource_not_acquired_returns_lease() {
        let queued_resource = QueuedResource::<_, 1>::new(0);

        let result = queued_resource.try_acquire_resource();

        assert_matches!(result, Ok(_));
    }

    #[test]
    fn resource_already_acquired_returns_err() {
        let queued_resource = QueuedResource::<_, 1>::new(0);
        let _initial_lease = queued_resource.try_acquire_resource().unwrap();

        let result = queued_resource.try_acquire_resource();

        assert_matches!(result, Err(QueuedResourceBusyError));
    }
}

mod acquire_resource_async {
    use super::*;

    #[tokio::test]
    async fn resource_not_acquired_returns_lease() {
        let queued_resource = QueuedResource::<_, 1>::new(0);

        let result = queued_resource.acquire_resource_async().await;

        assert_matches!(result, Ok(_));
    }

    #[tokio::test]
    async fn resource_already_acquired_is_enqueued() {
        let queued_resource = QueuedResource::<_, 1>::new(AtomicU32::new(0));
        let initial_counter_lease = queued_resource.try_acquire_resource().unwrap();

        tokio::join!(
            async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                initial_counter_lease.fetch_add(1, Ordering::AcqRel);
            },
            async {
                assert_matches!(
                    queued_resource.try_acquire_resource(),
                    Err(QueuedResourceBusyError)
                );

                let second_counter_lease = queued_resource.acquire_resource_async().await.unwrap();

                assert_eq!(second_counter_lease.load(Ordering::Acquire), 1);
            }
        );
    }

    #[tokio::test]
    async fn too_many_enqueued_returns_busy() {
        let queued_resource = QueuedResource::<_, 1>::new(AtomicU32::new(0));
        let initial_counter_lease = queued_resource.try_acquire_resource().unwrap();

        tokio::join!(
            async move {
                tokio::time::sleep(Duration::from_millis(200)).await;
                initial_counter_lease.fetch_add(1, Ordering::AcqRel);
            },
            async {
                tokio::time::sleep(Duration::from_millis(100)).await;

                assert_matches!(
                    queued_resource.try_acquire_resource(),
                    Err(QueuedResourceBusyError)
                );

                let third_result = queued_resource.acquire_resource_async().await;

                assert_matches!(third_result, Err(QueuedResourceBusyError));
            },
            async {
                assert_matches!(
                    queued_resource.try_acquire_resource(),
                    Err(QueuedResourceBusyError)
                );

                let second_counter_lease = queued_resource.acquire_resource_async().await.unwrap();

                assert_eq!(second_counter_lease.load(Ordering::Acquire), 1);
            },
        );
    }

    #[tokio::test]
    async fn future_cancellation_does_not_block_future_acquisition() {
        let queued_resource = QueuedResource::<_, 1>::new(AtomicU32::new(0));
        let initial_counter_lease = queued_resource.try_acquire_resource().unwrap();

        // Takes up the single slot
        {
            let mut second_counter_lease_future = queued_resource.acquire_resource_async();

            assert_matches!(
                pin!(&mut second_counter_lease_future)
                    .poll(&mut Context::from_waker(Waker::noop())),
                Poll::Pending
            );

            drop(second_counter_lease_future);
        }

        // Frees the slot taken without trying to wait for the second future
        drop(initial_counter_lease);

        assert_matches!(queued_resource.try_acquire_resource(), Ok(_));
    }

    #[test]
    fn future_cancellation_before_initial_enqueue_does_nothing() {
        let queued_resource = QueuedResource::<_, 1>::new(AtomicU32::new(0));

        let initial_counter_lease_future = queued_resource.acquire_resource_async();
        drop(initial_counter_lease_future);

        assert_matches!(queued_resource.try_acquire_resource(), Ok(_));
    }

    #[test]
    fn future_cancellation_when_assigned_does_not_block_future_acquisition() {
        let queued_resource = QueuedResource::<_, 1>::new(AtomicU32::new(0));
        let initial_counter_lease = queued_resource.try_acquire_resource().unwrap();

        {
            let mut second_counter_lease_future = queued_resource.acquire_resource_async();
            let poll_result = pin!(&mut second_counter_lease_future)
                .poll(&mut Context::from_waker(Waker::noop()));

            assert_matches!(poll_result, Poll::Pending);

            // Assigns the resource to the second future, attempts to wake it
            drop(initial_counter_lease);

            // Cancels the future without actually processing it
            drop(second_counter_lease_future);
        }

        assert_matches!(queued_resource.try_acquire_resource(), Ok(_));
    }
}
