use std::io::SeekFrom;
use async_trait::async_trait;

#[async_trait]
pub trait File {
    type Error;

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
    async fn seek(&mut self, whence: SeekFrom) -> Result<u64, Self::Error>;
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
}
