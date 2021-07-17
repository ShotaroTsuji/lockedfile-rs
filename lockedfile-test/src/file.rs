use std::io::{
    Read,
    Seek,
    SeekFrom,
    Write,
};
use tokio::io::{
    AsyncReadExt,
    AsyncSeekExt,
    AsyncWriteExt,
};
use std::path::PathBuf;
use lockedfile::{
    std::OwnedFile as StdOwnedFile,
    std::SharedFile as StdSharedFile,
    tokio::OwnedFile as TokioOwnedFile,
    tokio::SharedFile as TokioSharedFile,
};
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
        StdOwnedFile::create(path).map(Self::new)
    }

    async fn open_shared(path: PathBuf) -> Result<Self, Self::Error> {
        StdSharedFile::open(path).map(Self::new)
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

pub struct TokioFile(tokio::fs::File);

impl TokioFile {
    pub fn new(f: tokio::fs::File) -> Self {
        Self(f)
    }
}

#[async_trait]
impl File for TokioFile {
    type Error = std::io::Error;

    async fn create_owned(path: PathBuf) -> Result<Self, Self::Error> {
        TokioOwnedFile::create(path)
            .await
            .map(Self::new)
    }

    async fn open_shared(path: PathBuf) -> Result<Self, Self::Error> {
        TokioSharedFile::open(path)
            .await
            .map(Self::new)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_exact(buf).await
            .map(|_| ())
    }

    async fn seek(&mut self, whence: SeekFrom) -> Result<u64, Self::Error> {
        self.0.seek(whence).await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(buf).await
    }
}
