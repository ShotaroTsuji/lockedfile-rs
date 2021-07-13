use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use lockedfile::std::{OwnedFile, SharedFile};

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(tag="Message")]
pub enum Message {
    CreateExclusive(CreateExclusive),
    IoError(IoError),
    OpenedFile(OpenedFile),
    OpenShared(OpenShared),
    ReadRange(ReadRange),
    WriteZeros(WriteZeros),
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

    pub fn io_error(s: String) -> Self {
        Message::IoError(IoError { msg: s })
    }

    pub fn read_range(s: u64, e: u64) -> Self {
        Message::ReadRange(ReadRange { start: s, end: e })
    }
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct CreateExclusive {
    pub file: PathBuf,
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
pub struct WriteZeros {
    pub size: usize,
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
            Message::IoError(_) => panic!("Request error"),
            Message::OpenedFile(_) => self.opened_file(),
            Message::OpenShared(c) => self.open_shared(c.file),
            Message::ReadRange(c) => self.read_range(c.start, c.end),
            Message::WriteZeros(c) => self.write_zeros(c.size),
            Message::Quit => std::process::exit(0),
        }
    }

    pub fn create_exclusive(&mut self, path: PathBuf) -> Message {
        let file = match OwnedFile::create(&path) {
            Ok(file) => file,
            Err(e) => {
                return Message::IoError(IoError { msg: e.to_string() });
            },
        };

        self.file.replace((file, path.clone()));

        Message::CreateExclusive(CreateExclusive { file: path.clone() })
    }

    pub fn write_zeros(&mut self, size: usize) -> Message {
        assert!(size <= 4096);

        let (file, _) = self.file.as_mut().unwrap();
        let buf = vec![0; size];

        let written_size = file.write(&buf).unwrap();

        Message::WriteZeros(WriteZeros { size: written_size })
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
                Message::OpenShared(OpenShared { file: path.clone() })
            },
            Err(e) => {
                Message::IoError(IoError { msg: e.to_string(), })
            },
        }
    }

    pub fn read_range(&mut self, start: u64, end: u64) -> Message {
        let size = end - start;
        if size > 4096 {
            return Message::io_error("Size is too large".to_owned());
        }

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
