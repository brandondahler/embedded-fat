mod common;

use crate::common::std_file::StdFile;
use embedded_fat::FileSystemBuilder;
use embedded_io::Read;
use std::fs::File;

#[test]
fn fat12() {
    verify_disk("fat12.img");
}

#[test]
fn fat16() {
    verify_disk("fat16.img");
}

#[test]
fn fat32() {
    verify_disk("fat32.img");
}

fn verify_disk(file_name: &str) {
    let file_system = FileSystemBuilder::from_stream(StdFile::new(
        File::open(String::from("disks/") + file_name).unwrap(),
    ))
    .build()
    .expect("Opening disk works");

    {
        let mut file = file_system
            .open("TEST.TXT")
            .expect("Opening a file with a basic short name works");
        let mut bytes = [0; 5];

        file.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, "test\n".as_bytes());
    }

    {
        let mut file = file_system
            .open("long-File.name.txt")
            .expect("Opening a file with a long name with matching casing works");
        let mut bytes = [0; 9];

        file.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, "much wow\n".as_bytes());
    }

    {
        let mut file = file_system
            .open("long-file.name.txt")
            .expect("Opening a file with a long name with wrong casing works");
        let mut bytes = [0; 9];

        file.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, "much wow\n".as_bytes());
    }

    {
        let mut file = file_system
            .open("foo/bar.txt")
            .expect("Opening a file in a subfolder works");
        let mut bytes = [0; 7];

        file.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, "redrum\n".as_bytes());
    }
}
