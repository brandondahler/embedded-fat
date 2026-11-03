mod common;

use crate::common::delayed_std_file::DelayedStdFile;
use crate::common::std_file::StdFile;
use embedded_fat::FileSystemBuilder;
use embedded_io::Read;
use embedded_io_async::Read as AsyncRead;
use std::fs::File;
use std::time::Duration;

#[test]
fn multi_file_access_sync() {
    let file_system =
        FileSystemBuilder::from_stream(StdFile::new(File::open("disks/fat32.img").unwrap()))
            .build()
            .unwrap();

    let mut file_a = file_system.open("TEST.TXT").unwrap();
    let mut file_b = file_system.open("foo/bar.txt").unwrap();

    let mut bytes_a = [0; 5];
    let mut bytes_b = [0; 7];

    Read::read_exact(&mut file_a, &mut bytes_a).unwrap();
    Read::read_exact(&mut file_b, &mut bytes_b).unwrap();

    assert_eq!(bytes_a, "test\n".as_bytes());
    assert_eq!(bytes_b, "redrum\n".as_bytes());
}

#[tokio::test]
async fn multi_file_access_async() {
    let file_system = FileSystemBuilder::from_stream(DelayedStdFile::new(
        File::open("disks/fat32.img").unwrap(),
        Duration::from_millis(100),
        true,
    ))
    .build_async()
    .await
    .unwrap();

    let mut file_a = file_system.open_async("TEST.TXT").await.unwrap();
    let mut file_b = file_system.open_async("foo/bar.txt").await.unwrap();

    let mut bytes_a = [0; 5];
    let mut bytes_b = [0; 7];

    AsyncRead::read_exact(&mut file_a, &mut bytes_a)
        .await
        .unwrap();
    AsyncRead::read_exact(&mut file_b, &mut bytes_b)
        .await
        .unwrap();

    assert_eq!(bytes_a, "test\n".as_bytes());
    assert_eq!(bytes_b, "redrum\n".as_bytes());
}

#[tokio::test]
async fn multi_file_access_mixed() {
    let file_system = FileSystemBuilder::from_stream(DelayedStdFile::new(
        File::open("disks/fat32.img").unwrap(),
        Duration::from_millis(100),
        true,
    ))
    .build_async()
    .await
    .unwrap();

    let mut file_a = file_system.open_async("TEST.TXT").await.unwrap();
    let mut file_b = file_system.open("foo/bar.txt").unwrap();

    let mut bytes_a = [0; 5];
    let mut bytes_b = [0; 7];

    Read::read_exact(&mut file_a, &mut bytes_a).unwrap();
    AsyncRead::read_exact(&mut file_b, &mut bytes_b)
        .await
        .unwrap();

    assert_eq!(bytes_a, "test\n".as_bytes());
    assert_eq!(bytes_b, "redrum\n".as_bytes());
}
