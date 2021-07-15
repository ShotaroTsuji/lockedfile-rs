use std::io::{
    Read,
    Seek,
    SeekFrom,
    Write,
};
use std::path::PathBuf;
use lockedfile::std::{OwnedFile, SharedFile};
use async_trait::async_trait;

#[async_trait]
pub trait File: Sized {
    type Error: std::error::Error;

    async fn create_owned(path: PathBuf) -> Result<Self, Self::Error>;
    async fn open_shared(path: PathBuf) -> Result<Self, Self::Error>;
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
    async fn seek(&mut self, whence: SeekFrom) -> Result<u64, Self::Error>;
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
}

pub struct StdFile(std::fs::File);

impl StdFile {
    pub fn new(f: std::fs::File) -> Self {
        Self(f)
    }
}

#[async_trait]
impl File for StdFile {
    type Error = std::io::Error;

    async fn create_owned(path: PathBuf) -> Result<Self, Self::Error> {
        OwnedFile::create(path).map(Self::new)
    }

    async fn open_shared(path: PathBuf) -> Result<Self, Self::Error> {
        SharedFile::open(path).map(Self::new)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_exact(buf)
    }

    async fn seek(&mut self, whence: SeekFrom) -> Result<u64, Self::Error> {
        self.0.seek(whence)
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(buf)
    }
}
