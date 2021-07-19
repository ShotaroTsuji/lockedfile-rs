use std::io::{Read, Write};
use lockedfile::std::OpenOptions;
use tempfile::TempPath;

mod common;

fn create_temp_file_with_content() -> TempPath {
    let path = common::create_temp_path();
    {
        let mut file = std::fs::File::create(&path).unwrap();
        write_block(&mut file);
    }

    path
}

fn write_block<W: Write>(mut w: W) {
    let buf = vec![0u8; 4096];
    let size = w.write(&buf).unwrap();
    assert_eq!(size, 4096);
}

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

open_and_read!(open_and_read_unlocked_file, std::fs::File);
open_and_read!(open_and_read_shared_file, lockedfile::std::SharedFile);
open_and_read!(open_and_read_owned_file, lockedfile::std::OwnedFile);

macro_rules! create_and_write {
    ($func_name:ident, $file_type:ty) => {
        #[test]
        fn $func_name() {
            let path = common::create_temp_path();
            let mut file = <$file_type>::create(&path).unwrap();
            write_block(&mut file);
        }
    };
}

create_and_write!(create_and_write_unlocked_file, std::fs::File);
create_and_write!(create_and_write_owned_file, lockedfile::std::OwnedFile);

#[test]
fn open_and_read_owned_file() {
    let path = create_temp_file_with_content();
    let mut file = lockedfile::std::OwnedFile::open(&path).unwrap();
    read_block(&mut file);
}

#[test]
fn create_and_append_shared_file() {
    let path = common::create_temp_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open_shared(&path)
        .unwrap();
    write_block(&mut file);
}

#[test]
fn create_and_append_owned_file() {
    let path = common::create_temp_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open_exclusive(&path)
        .unwrap();
    write_block(&mut file);
}
