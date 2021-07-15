use std::io::{Read, Write};
use lockedfile::std::OpenOptions;
use tempfile::TempPath;

mod common;

fn create_temp_file_with_content() -> TempPath {
    use std::fs::File;

    let path = common::create_temp_path();

    {
        let mut file = File::create(&path).unwrap();

        let size = write_block(&mut file).unwrap();

        assert_eq!(size, 4096);
    }

    path
}

fn write_block<W: Write>(mut w: W) -> std::io::Result<usize> {
    let buf = vec![0u8; 4096];
    w.write(&buf)
}

// SharedFile::open みたいにした方が良さそう

fn read_block(file: &mut std::fs::File) {
    let mut buf = vec![0u8; 4096];
    let size = file.read(&mut buf).unwrap();
    assert_eq!(size, 4096);
}

macro_rules! open_and_read {
    ($func_name:ident, $file_type:ty) => {
        #[test]
        fn $func_name() {
            let path = create_temp_file_with_content();

            let mut file = <$file_type>::open(&path).unwrap();

            read_block(&mut file);
        }
    };
}

open_and_read!(open_and_read_unlocked, std::fs::File);
open_and_read!(open_and_read_shared, lockedfile::std::SharedFile);
open_and_read!(open_and_read_owned, lockedfile::std::OwnedFile);

macro_rules! create_and_write {
    ($func_name:ident, $file_type:ty) => {
        #[test]
        fn $func_name() {
            let path = common::create_temp_path();

            let mut file = <$file_type>::create(&path).unwrap();

            let size = write_block(&mut file).unwrap();
            assert_eq!(size, 4096);
        }
    };
}

create_and_write!(create_and_write_unlocked, std::fs::File);
create_and_write!(create_and_write_owned, lockedfile::std::OwnedFile);

#[test]
fn open_and_write_owned() {
    let path = create_temp_file_with_content();

    let mut file = lockedfile::std::OwnedFile::open(&path).unwrap();

    let data = vec![1; 2048];

    let size = file.write(&data).unwrap();
    assert_eq!(size, 2048);
}

#[test]
fn create_and_append_with_shared_lock() {
    let path = common::create_temp_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open_shared(&path)
        .unwrap();

    let size = write_block(&mut file).unwrap();
    assert_eq!(size, 4096);
}

#[test]
fn create_and_append_with_exclusive_lock() {
    let path = common::create_temp_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open_exclusive(&path)
        .unwrap();

    let size = write_block(&mut file).unwrap();
    assert_eq!(size, 4096);
}

#[test]
fn read_file_with_shared_lock() {
    let path = create_temp_file_with_content();
    let mut file = OpenOptions::new()
        .read(true)
        .open_shared(&path)
        .unwrap();

    let mut buf = vec![0u8; 4096];
    let size = file.read(&mut buf).unwrap();
    assert_eq!(size, 4096);
}

#[test]
fn read_file_with_exclusive_lock() {
    let path = create_temp_file_with_content();
    let mut file = OpenOptions::new()
        .read(true)
        .open_exclusive(&path)
        .unwrap();

    let mut buf = vec![0u8; 4096];
    let size = file.read(&mut buf).unwrap();
    assert_eq!(size, 4096);
}
