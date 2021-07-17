use std::path::Path;

pub struct SharedFile;

impl SharedFile {
    pub async fn open<P: AsRef<Path>>(path: P) -> std::io::Result<tokio::fs::File> {
        OpenOptions::new()
            .read(true)
            .open_shared(path)
            .await
    }
}

pub struct OwnedFile;

impl OwnedFile {
    pub async fn open<P: AsRef<Path>>(path: P) -> std::io::Result<tokio::fs::File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open_exclusive(path)
            .await
    }

    pub async fn create<P: AsRef<Path>>(path: P) -> std::io::Result<tokio::fs::File> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .open_exclusive(path)
            .await
    }
}

pub struct OpenOptions {
    sys: tokio::fs::OpenOptions,
    #[cfg(unix)]
    flags: i32,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            sys: tokio::fs::OpenOptions::new(),
            #[cfg(unix)]
            flags: 0,
        }
    }

    crate::impl_open_options!();

    pub async fn open_exclusive<P: AsRef<Path>>(&self, path: P) -> std::io::Result<tokio::fs::File> {
        self.sys.clone()
            .custom_flags(self.flags | libc::O_EXLOCK)
            .open(path)
            .await
    }

    pub async fn open_shared<P: AsRef<Path>>(&self, path: P) -> std::io::Result<tokio::fs::File> {
        self.sys.clone()
            .custom_flags(self.flags | libc::O_SHLOCK)
            .open(path)
            .await
    }

    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.sys.mode(mode);

        self
    }

    pub fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.flags = flags;

        self
    }
}
