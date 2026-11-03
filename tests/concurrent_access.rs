use crate::common::delayed_std_file::DelayedStdFile;
use core::assert_matches;
use embedded_fat::{FileError, FileSystemBuilder};
use embedded_io::ReadExactError;
use embedded_io_async::Read as AsyncRead;
use std::fs::File;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use tokio::sync::Barrier;
use tokio::task::JoinSet;

mod common;

#[test]
fn concurrent_access_sync() {
    let file_system = FileSystemBuilder::from_stream(DelayedStdFile::new(
        File::open("disks/fat32.img").unwrap(),
        Duration::from_millis(100),
        false,
    ))
    .with_device_queue_length::<2>()
    .build()
    .unwrap();

    let mut file_1 = file_system.open("TEST.TXT").unwrap();
    let mut file_2 = file_system.open("TEST.TXT").unwrap();

    let mut data_1 = [0; 5];
    let mut data_2 = [0; 5];

    {
        let mut read_future_1 = Some(AsyncRead::read_exact(&mut file_1, &mut data_1));
        let mut read_future_2 = Some(AsyncRead::read_exact(&mut file_2, &mut data_2));

        fn poll_future(
            future_container: &mut Option<
                impl Future<Output = Result<(), ReadExactError<FileError<std::io::Error>>>>,
            >,
        ) {
            if let Some(future) = future_container {
                let poll_result = {
                    let pinned_future = unsafe { Pin::new_unchecked(future) };
                    pinned_future.poll(&mut Context::from_waker(Waker::noop()))
                };

                match poll_result {
                    Poll::Pending => {}
                    Poll::Ready(result) => {
                        assert_matches!(result, Ok(()));
                        *future_container = None;
                    }
                }
            }
        }

        while read_future_1.is_some() || read_future_2.is_some() {
            poll_future(&mut read_future_1);
            poll_future(&mut read_future_2);
        }
    }

    assert_eq!(data_1, "test\n".as_bytes());
    assert_eq!(data_2, "test\n".as_bytes());
}

#[tokio::test]
async fn concurrent_access_async() {
    let file_system = Arc::new(
        FileSystemBuilder::from_stream(DelayedStdFile::new(
            File::open("disks/fat32.img").unwrap(),
            Duration::from_millis(100),
            true,
        ))
        .with_device_queue_length::<2>()
        .build()
        .unwrap(),
    );
    let barrier = Arc::new(Barrier::new(2));

    let mut join_set = JoinSet::new();

    for _ in 0..2 {
        let task_file_system = file_system.clone();
        let task_barrier = barrier.clone();

        join_set.spawn(async move {
            task_barrier.wait().await;
            let mut file = task_file_system.open_async("TEST.TXT").await.unwrap();

            let mut data = [0; 5];
            task_barrier.wait().await;
            AsyncRead::read_exact(&mut file, &mut data).await.unwrap();

            assert_eq!(data, "test\n".as_bytes());
        });
    }

    join_set.join_all().await;
}
