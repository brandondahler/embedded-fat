mod common;

use crate::common::std_file::StdFile;
use embedded_fat::FileSystemBuilder;
use embedded_io::Read;
use std::fs::File;

#[test]
fn sync() {
    let file_system =
        FileSystemBuilder::from_stream(StdFile::new(File::open("disks/fat32.img").unwrap()))
            .build()
            .unwrap();

    let mut file_a = file_system.open("TEST.TXT").unwrap();
    let mut file_b = file_system.open("foo/bar.txt").unwrap();

    let mut bytes_a = [0; 5];
    let mut bytes_b = [0; 7];

    file_a.read_exact(&mut bytes_a).unwrap();
    file_b.read_exact(&mut bytes_b).unwrap();

    assert_eq!(bytes_a, "test\n".as_bytes());
    assert_eq!(bytes_b, "redrum\n".as_bytes());
}
