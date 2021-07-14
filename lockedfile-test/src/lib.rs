use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use lockedfile::std::{OwnedFile, SharedFile};

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(tag="Message")]
pub enum Message {
    CreateExclusive(CreateExclusive),
    FileLength(FileLength),
    IoError(IoError),
    OpenedFile(OpenedFile),
    OpenShared(OpenShared),
    ReadRange(ReadRange),
    WriteRange(WriteRange),
    Quit,
    /*
    OpenExclusive(PathBuf),
    */
}

impl Message {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self.to_json()).unwrap()
    }

    pub fn from_str(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }

    pub fn create_exclusive(file: PathBuf) -> Self {
        Message::CreateExclusive(CreateExclusive { file: file })
    }

    pub fn file_length(len: u64) -> Self {
        Message::FileLength(FileLength { len: len })
    }

    pub fn io_error(s: String) -> Self {
        Message::IoError(IoError { msg: s })
    }

    pub fn open_shared(file: PathBuf) -> Self {
        Message::OpenShared(OpenShared { file: file })
    }

    pub fn read_range(s: u64, e: u64) -> Self {
        Message::ReadRange(ReadRange { start: s, end: e })
    }

    pub fn write_range(s: u64, e: u64) -> Self {
        Message::WriteRange(WriteRange { start: s, end: e })
    }
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct CreateExclusive {
    pub file: PathBuf,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct FileLength {
    pub len: u64,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct OpenedFile {
    path: Option<PathBuf>,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct OpenShared {
    pub file: PathBuf,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct ReadRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct WriteRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct IoError {
    msg: String,
}

pub struct Executor {
    file: Option<(File, PathBuf)>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            file: None,
        }
    }

    pub fn exec_str(&mut self, s: &str) -> Result<Message, String> {
        let req = Message::from_str(s);

        Ok(self.execute(req))
    }

    pub fn execute(&mut self, req: Message) -> Message {
        match req {
            Message::CreateExclusive(c) => self.create_exclusive(c.file),
            Message::FileLength(_) => self.file_length(),
            Message::IoError(_) => panic!("Request error"),
            Message::OpenedFile(_) => self.opened_file(),
            Message::OpenShared(c) => self.open_shared(c.file),
            Message::ReadRange(c) => self.read_range(c.start, c.end),
            Message::WriteRange(c) => self.write_range(c.start, c.end),
            Message::Quit => std::process::exit(0),
        }
    }

    pub fn create_exclusive(&mut self, path: PathBuf) -> Message {
        let file = match OwnedFile::create(&path) {
            Ok(file) => file,
            Err(e) => {
                return Message::io_error(e.to_string());
            },
        };

        self.file.replace((file, path.clone()));

        Message::create_exclusive(path.clone())
    }

    pub fn file_length(&mut self) -> Message {
        if let Some((file, _)) = self.file.as_mut() {
            match file.seek(SeekFrom::End(0)) {
                Ok(size) => Message::file_length(size),
                Err(e) => Message::io_error(e.to_string()),
            }
        } else {
            Message::io_error("File is not opened".to_owned())
        }
    }

    pub fn opened_file(&mut self) -> Message {
        Message::OpenedFile(
            OpenedFile {
                path: self.file.as_ref().map(|x| x.1.clone()),
            })
    }

    pub fn open_shared(&mut self, path: PathBuf) -> Message {
        match SharedFile::open(&path) {
            Ok(file) => {
                self.file.replace((file, path.clone()));
                Message::open_shared(path.clone())
            },
            Err(e) => Message::io_error(e.to_string()),
        }
    }

    pub fn read_range(&mut self, start: u64, end: u64) -> Message {
        let size = match calc_and_check_size(start..end) {
            Ok(size) => size,
            Err(e) => return Message::io_error(e),
        };

        if let Some((file, _)) = self.file.as_mut() {
            match file.seek(SeekFrom::Start(start)) {
                Ok(_) => {},
                Err(e) => return Message::io_error(e.to_string()),
            }

            let mut buf = vec! [0; size as usize];
            match file.read(&mut buf) {
                Ok(size) => Message::read_range(start, start + size as u64),
                Err(e) => Message::io_error(e.to_string()),
            }
        } else {
            Message::io_error("File is not opened".to_owned())
        }
    }

    pub fn write_range(&mut self, start: u64, end: u64) -> Message {
        let size = match calc_and_check_size(start..end) {
            Ok(size) => size,
            Err(e) => return Message::io_error(e),
        };

        if let Some((file, _)) = self.file.as_mut() {
            match file.seek(SeekFrom::Start(start)) {
                Ok(_) => {},
                Err(e) => return Message::io_error(e.to_string()),
            }

            let buf = vec![0; size as usize];
            match file.write(&buf) {
                Ok(size) => Message::write_range(start, start + size as u64),
                Err(e) => Message::io_error(e.to_string()),
            }
        } else {
            Message::io_error("File is not opened".to_owned())
        }
    }
}

fn calc_and_check_size(range: std::ops::Range<u64>) -> Result<u64, String> {
    if range.start > range.end {
        Err("range.start > range.end".to_owned())
    } else if range.end > 4096 {
        Err("range.end > 4096".to_owned())
    } else {
        Ok(range.end - range.start)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize_to_json() {
        let cmd = Message::CreateExclusive(
            CreateExclusive {
                file: "file.txt".into(),
            });
        let s = serde_json::to_string_pretty(&cmd).unwrap();
        println!("{}", s);
    }
}
